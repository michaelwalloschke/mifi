//! PayPal Aktivitäten-Export CSV importer (SPEC.md §6, asset 17 §1).
//!
//! UTF-8 with BOM, comma-separated, all fields quoted, German dates/decimals. Row
//! filter: `Auswirkung auf Guthaben ∈ {Soll, Haben}` ∧ `Status = Abgeschlossen`, holds
//! and FX-conversion rows excluded (the latter folded into their parent's EUR leg).

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDate;

use super::{parse_german_decimal_cents, NormalizedRow, ParsedCsv, RowNote};
use crate::normalize::strip_processor_prefix;

const REQUIRED_ALIASES: &[&[&str]] = &[&["Datum"], &["Status"], &["Währung"], &["Brutto"], &["Transaktionscode"]];

struct Columns {
    datum: usize,
    name: Option<usize>,
    typ: Option<usize>,
    status: usize,
    waehrung: usize,
    brutto: usize,
    netto: Option<usize>,
    transaktionscode: usize,
    zugehoeriger_transaktionscode: Option<usize>,
    auswirkung: Option<usize>,
    betreff: Option<usize>,
    hinweis: Option<usize>,
    artikelbezeichnung: Option<usize>,
    rechnungsnummer: Option<usize>,
    absender_email: Option<usize>,
    empfaenger_email: Option<usize>,
}

fn find_column(headers: &csv::StringRecord, names: &[&str]) -> Option<usize> {
    names.iter().find_map(|name| headers.iter().position(|h| h == *name))
}

fn resolve_columns(headers: &csv::StringRecord) -> Result<Columns, String> {
    for aliases in REQUIRED_ALIASES {
        if find_column(headers, aliases).is_none() {
            return Err(format!("missing required column: {}", aliases[0]));
        }
    }
    let typ = find_column(headers, &["Typ", "Beschreibung"]);
    if typ.is_none() {
        return Err("missing required column: Typ (or Beschreibung)".to_string());
    }

    Ok(Columns {
        datum: find_column(headers, &["Datum"]).unwrap(),
        name: find_column(headers, &["Name"]),
        typ,
        status: find_column(headers, &["Status"]).unwrap(),
        waehrung: find_column(headers, &["Währung"]).unwrap(),
        brutto: find_column(headers, &["Brutto"]).unwrap(),
        netto: find_column(headers, &["Netto"]),
        transaktionscode: find_column(headers, &["Transaktionscode"]).unwrap(),
        zugehoeriger_transaktionscode: find_column(headers, &["Zugehöriger Transaktionscode"]),
        auswirkung: find_column(headers, &["Auswirkung auf Guthaben"]),
        betreff: find_column(headers, &["Betreff"]),
        hinweis: find_column(headers, &["Hinweis"]),
        artikelbezeichnung: find_column(headers, &["Artikelbezeichnung"]),
        rechnungsnummer: find_column(headers, &["Rechnungsnummer"]),
        absender_email: find_column(headers, &["Absender E-Mail-Adresse"]),
        empfaenger_email: find_column(headers, &["Empfänger E-Mail-Adresse"]),
    })
}

fn get(record: &csv::StringRecord, idx: Option<usize>) -> &str {
    idx.and_then(|i| record.get(i)).unwrap_or("").trim()
}

fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)
}

/// Parses a PayPal Aktivitäten-Export CSV file. Unknown/undecodable header is a hard
/// error (whole file rejected, SPEC.md §6); individual malformed rows or unmatched FX
/// legs are collected in `ParsedCsv::skipped` instead of aborting the import.
pub fn parse_csv(path: impl AsRef<Path>) -> Result<ParsedCsv, String> {
    let bytes = std::fs::read(path.as_ref()).map_err(|e| format!("failed to read file: {e}"))?;
    let bytes = strip_bom(&bytes);

    let mut reader = csv::ReaderBuilder::new().delimiter(b',').from_reader(bytes);
    let headers = reader
        .headers()
        .map_err(|e| format!("failed to read header row: {e}"))?
        .clone();
    let columns = resolve_columns(&headers)?;

    let records: Vec<csv::StringRecord> = reader
        .records()
        .collect::<Result<_, _>>()
        .map_err(|e| format!("failed to read CSV rows: {e}"))?;

    // Pass 1: harvest the EUR-debit leg of every FX conversion triple, keyed by the
    // parent payment's Transaktionscode (asset 17 §1.5).
    let mut fx_eur_legs: HashMap<String, i64> = HashMap::new();
    for record in &records {
        if get(record, columns.typ) == "Allgemeine Währungsumrechnung" && get(record, Some(columns.waehrung)) == "EUR" {
            if let Some(parent_code) = columns.zugehoeriger_transaktionscode.and_then(|i| record.get(i)) {
                let netto = get(record, columns.netto);
                let amount = if !netto.is_empty() { netto } else { get(record, Some(columns.brutto)) };
                if let Ok(cents) = parse_german_decimal_cents(amount) {
                    fx_eur_legs.insert(parent_code.trim().to_string(), cents);
                }
            }
        }
    }

    let mut parsed = ParsedCsv::default();

    for (i, record) in records.iter().enumerate() {
        let row_number = i + 2; // 1-based, header is row 1
        let typ = get(record, columns.typ);
        let status = get(record, Some(columns.status));
        let auswirkung = columns.auswirkung.map(|idx| get(record, Some(idx)));

        if status != "Abgeschlossen" {
            continue;
        }
        if let Some(auswirkung) = &auswirkung {
            if auswirkung != &"Soll" && auswirkung != &"Haben" {
                continue; // Memo rows never moved the balance.
            }
        }
        if typ.contains("Einbehaltung") {
            continue; // Hold/release pairs net to zero.
        }
        if typ == "Allgemeine Währungsumrechnung" {
            continue; // Folded into the parent payment row below.
        }

        let booking_date = match NaiveDate::parse_from_str(get(record, Some(columns.datum)), "%d.%m.%Y") {
            Ok(date) => date,
            Err(e) => {
                parsed.skipped.push(RowNote { row_number, reason: format!("invalid Datum: {e}") });
                continue;
            }
        };

        let waehrung = get(record, Some(columns.waehrung));
        let own_code = get(record, Some(columns.transaktionscode)).to_string();

        let netto = get(record, columns.netto);
        let raw_amount = if !netto.is_empty() { netto } else { get(record, Some(columns.brutto)) };
        let payment_amount_cents = match parse_german_decimal_cents(raw_amount) {
            Ok(cents) => cents,
            Err(reason) => {
                parsed.skipped.push(RowNote { row_number, reason });
                continue;
            }
        };

        let (amount_cents, fx_metadata) = if waehrung == "EUR" {
            (payment_amount_cents, None)
        } else {
            match fx_eur_legs.get(&own_code) {
                Some(&eur_cents) => (
                    eur_cents,
                    Some(format!(
                        r#"{{"original_currency":"{waehrung}","original_amount_cents":{payment_amount_cents}}}"#
                    )),
                ),
                None => {
                    parsed.skipped.push(RowNote {
                        row_number,
                        reason: format!("non-EUR row ({waehrung}) has no matching EUR conversion leg"),
                    });
                    continue;
                }
            }
        };

        let counterparty = {
            let name = get(record, columns.name);
            if !name.is_empty() {
                name.to_string()
            } else {
                let fallback = match auswirkung {
                    Some("Haben") => get(record, columns.absender_email),
                    Some("Soll") => get(record, columns.empfaenger_email),
                    _ => "",
                };
                let fallback = if fallback.is_empty() {
                    let a = get(record, columns.absender_email);
                    if !a.is_empty() { a } else { get(record, columns.empfaenger_email) }
                } else {
                    fallback
                };
                if fallback.is_empty() { "PayPal".to_string() } else { strip_processor_prefix(fallback).to_string() }
            }
        };

        let mut purpose_parts: Vec<&str> = [columns.betreff, columns.hinweis, columns.artikelbezeichnung]
            .into_iter()
            .map(|idx| get(record, idx))
            .filter(|s| !s.is_empty())
            .collect();
        let rechnungsnummer = get(record, columns.rechnungsnummer);
        if !rechnungsnummer.is_empty() {
            purpose_parts.push(rechnungsnummer);
        }
        let purpose = purpose_parts.join(", ");

        parsed.rows.push(NormalizedRow {
            booking_date,
            amount_cents,
            counterparty_raw: counterparty,
            purpose_raw: purpose,
            external_ref: Some(own_code),
            fx_metadata,
        });
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(contents: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        file
    }

    const HEADER: &str = "\"Datum\",\"Name\",\"Typ\",\"Status\",\"Währung\",\"Brutto\",\"Gebühr\",\"Netto\",\"Absender E-Mail-Adresse\",\"Empfänger E-Mail-Adresse\",\"Transaktionscode\",\"Zugehöriger Transaktionscode\",\"Betreff\",\"Hinweis\",\"Artikelbezeichnung\",\"Rechnungsnummer\",\"Auswirkung auf Guthaben\"\n";

    #[allow(clippy::too_many_arguments)]
    fn row(
        datum: &str,
        name: &str,
        typ: &str,
        status: &str,
        waehrung: &str,
        brutto: &str,
        code: &str,
        parent_code: &str,
        auswirkung: &str,
    ) -> String {
        format!(
            "\"{datum}\",\"{name}\",\"{typ}\",\"{status}\",\"{waehrung}\",\"{brutto}\",\"0,00\",\"{brutto}\",\"\",\"\",\"{code}\",\"{parent_code}\",\"Kauf\",\"\",\"\",\"\",\"{auswirkung}\"\n"
        )
    }

    #[test]
    fn imports_a_simple_eur_payment() {
        let mut contents = HEADER.to_string();
        contents.push_str(&row("18.06.2025", "Steam", "Website-Zahlung", "Abgeschlossen", "EUR", "-9,99", "TXN1", "", "Soll"));
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(parsed.rows[0].amount_cents, -999);
        assert_eq!(parsed.rows[0].counterparty_raw, "Steam");
        assert_eq!(parsed.rows[0].external_ref.as_deref(), Some("TXN1"));
    }

    #[test]
    fn drops_memo_and_pending_and_hold_rows() {
        let mut contents = HEADER.to_string();
        contents.push_str(&row("18.06.2025", "A", "Zahlungsanforderung", "Abgeschlossen", "EUR", "-1,00", "T1", "", "Memo"));
        contents.push_str(&row("18.06.2025", "B", "Website-Zahlung", "Ausstehend", "EUR", "-1,00", "T2", "", "Soll"));
        contents.push_str(&row("18.06.2025", "C", "Einbehaltung für offene Autorisierung", "Abgeschlossen", "EUR", "-1,00", "T3", "", "Soll"));
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert!(parsed.rows.is_empty());
    }

    #[test]
    fn folds_fx_triple_into_eur_leg() {
        let mut contents = HEADER.to_string();
        // Foreign-currency payment leg.
        contents.push_str(&row("18.06.2025", "Steam", "Website-Zahlung", "Abgeschlossen", "USD", "-10,99", "PAY1", "", "Soll"));
        // EUR debit leg of the conversion (this is what should be booked).
        contents.push_str(&row("18.06.2025", "", "Allgemeine Währungsumrechnung", "Abgeschlossen", "EUR", "-9,50", "FX1", "PAY1", "Soll"));
        // Foreign-currency credit leg offsetting the payment (ignored).
        contents.push_str(&row("18.06.2025", "", "Allgemeine Währungsumrechnung", "Abgeschlossen", "USD", "10,99", "FX2", "PAY1", "Haben"));
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(parsed.rows[0].amount_cents, -950);
        assert!(parsed.rows[0].fx_metadata.as_ref().unwrap().contains("USD"));
    }

    #[test]
    fn non_eur_row_without_matching_leg_is_skipped_not_fatal() {
        let mut contents = HEADER.to_string();
        contents.push_str(&row("18.06.2025", "Steam", "Website-Zahlung", "Abgeschlossen", "USD", "-10,99", "PAY1", "", "Soll"));
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert!(parsed.rows.is_empty());
        assert_eq!(parsed.skipped.len(), 1);
        assert!(parsed.skipped[0].reason.contains("no matching EUR"));
    }

    #[test]
    fn missing_required_column_is_a_hard_error() {
        let contents = "\"Foo\",\"Bar\"\n\"1\",\"2\"\n".to_string();
        let file = write_fixture(&contents);
        assert!(parse_csv(file.path()).is_err());
    }
}
