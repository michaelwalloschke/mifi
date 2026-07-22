//! Scalable Capital broker transaction export CSV importer (SPEC.md §6, asset 17 §2).
//!
//! Semicolon-separated, English headers, German decimals, ISO dates. `assetType`
//! routes each row to depot (`Security`) or Verrechnungskonto (`Cash`) — both import
//! as ordinary Transactions for flows/transfer-healing; valuation stays scalable-cli's
//! job (SPEC.md §11.5).

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDate;
use rusqlite::Connection;

use super::{parse_german_decimal_cents, ImportSummary, NormalizedRow, RowNote};

pub const DEPOT_ACCOUNT_ID: i64 = 3;
pub const VERRECHNUNGSKONTO_ACCOUNT_ID: i64 = 4;

struct Columns {
    date: usize,
    status: usize,
    reference: usize,
    description: usize,
    asset_type: usize,
    r#type: usize,
    isin: usize,
    shares: usize,
    price: usize,
    amount: usize,
    currency: usize,
}

fn find_column(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

fn resolve_columns(headers: &csv::StringRecord) -> Result<Columns, String> {
    let get = |name: &str| find_column(headers, name).ok_or_else(|| format!("missing required column: {name}"));
    Ok(Columns {
        date: get("date")?,
        status: get("status")?,
        reference: get("reference")?,
        description: get("description")?,
        asset_type: get("assetType")?,
        r#type: get("type")?,
        isin: get("isin")?,
        shares: get("shares")?,
        price: get("price")?,
        amount: get("amount")?,
        currency: get("currency")?,
    })
}

fn get(record: &csv::StringRecord, idx: usize) -> &str {
    record.get(idx).unwrap_or("").trim()
}

/// One imported row plus which account it belongs to (`assetType`-routed).
pub struct ScalableRow {
    pub account_id: i64,
    pub row: NormalizedRow,
}

pub struct ParsedScalableCsv {
    pub rows: Vec<ScalableRow>,
    pub skipped: Vec<RowNote>,
    /// Rows imported but worth a human glance: amounts with >2 decimal places were
    /// rounded half-away-from-zero (asset 17 §2.3 — rounding rule not yet verified
    /// against a real settlement).
    pub flagged_for_review: Vec<RowNote>,
}

pub fn parse_csv(path: impl AsRef<Path>) -> Result<ParsedScalableCsv, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(path.as_ref())
        .map_err(|e| format!("failed to open file: {e}"))?;
    let headers = reader
        .headers()
        .map_err(|e| format!("failed to read header row: {e}"))?
        .clone();
    let columns = resolve_columns(&headers)?;

    let records: Vec<csv::StringRecord> = reader
        .records()
        .collect::<Result<_, _>>()
        .map_err(|e| format!("failed to read CSV rows: {e}"))?;

    let mut result = ParsedScalableCsv { rows: Vec::new(), skipped: Vec::new(), flagged_for_review: Vec::new() };

    for (i, record) in records.iter().enumerate() {
        let row_number = i + 2;

        if get(record, columns.status) != "Executed" {
            continue;
        }

        let currency = get(record, columns.currency);
        if currency != "EUR" {
            result.skipped.push(RowNote { row_number, reason: format!("non-EUR currency not supported: {currency}") });
            continue;
        }

        let booking_date = match NaiveDate::parse_from_str(get(record, columns.date), "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => {
                result.skipped.push(RowNote { row_number, reason: format!("invalid date: {e}") });
                continue;
            }
        };

        let raw_amount = get(record, columns.amount);
        let amount_cents = match parse_german_decimal_cents(raw_amount) {
            Ok(cents) => cents,
            Err(reason) => {
                result.skipped.push(RowNote { row_number, reason });
                continue;
            }
        };
        if raw_amount.split(',').nth(1).is_some_and(|frac| frac.len() > 2) {
            result.flagged_for_review.push(RowNote {
                row_number,
                reason: "amount had >2 decimal places, rounded half-away-from-zero — verify against settlement".to_string(),
            });
        }

        let asset_type = get(record, columns.asset_type);
        let r#type = get(record, columns.r#type);
        let description = get(record, columns.description);
        let isin = get(record, columns.isin);
        let shares = get(record, columns.shares);
        let price = get(record, columns.price);

        let account_id = match asset_type {
            "Security" => DEPOT_ACCOUNT_ID,
            "Cash" => VERRECHNUNGSKONTO_ACCOUNT_ID,
            other => {
                result.skipped.push(RowNote { row_number, reason: format!("unknown assetType: {other}") });
                continue;
            }
        };

        let counterparty = if asset_type == "Security" {
            description.to_string()
        } else if !description.is_empty() {
            format!("{type}: {description}")
        } else {
            r#type.to_string()
        };

        let mut purpose = r#type.to_string();
        if !description.is_empty() {
            purpose.push(' ');
            purpose.push_str(description);
        }
        if !isin.is_empty() {
            purpose.push(' ');
            purpose.push_str(isin);
        }
        if !shares.is_empty() && !price.is_empty() {
            purpose.push(' ');
            purpose.push_str(shares);
            purpose.push('@');
            purpose.push_str(price);
        }

        result.rows.push(ScalableRow {
            account_id,
            row: NormalizedRow {
                booking_date,
                amount_cents,
                counterparty_raw: counterparty,
                purpose_raw: purpose,
                external_ref: Some(get(record, columns.reference).to_string()),
                fx_metadata: None,
            },
        });
    }

    Ok(result)
}

/// Commits parsed Scalable rows, grouped by their routed account (depot vs.
/// Verrechnungskonto), each in its own atomic commit per SPEC.md §5.
pub fn commit(conn: &mut Connection, rows: &[ScalableRow]) -> rusqlite::Result<ImportSummary> {
    let mut by_account: HashMap<i64, Vec<NormalizedRow>> = HashMap::new();
    for r in rows {
        by_account.entry(r.account_id).or_default().push(r.row.clone());
    }

    let mut total = ImportSummary::default();
    for (account_id, account_rows) in by_account {
        let summary = super::commit(conn, account_id, "csv-scalable", &account_rows)?;
        total.imported += summary.imported;
        total.duplicate_external_ref_skipped += summary.duplicate_external_ref_skipped;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(contents: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        file
    }

    const HEADER: &str = "date;time;status;reference;description;assetType;type;isin;shares;price;amount;fee;tax;currency\n";

    #[test]
    fn routes_security_rows_to_depot_and_cash_rows_to_verrechnungskonto() {
        let mut contents = HEADER.to_string();
        contents.push_str("2024-10-23;13:10:35;Executed;\"SCALHGJwmX8Bo9W\";\"Uranium Energy Co\";Security;Buy;US9168961038;80;7,34;-587,20;0,00;0,00;EUR\n");
        contents.push_str("2025-09-25;02:00:00;Executed;\"WWEK 51597383\";\"iShares Core FTSE 100 (Dist)\";Cash;Distribution;IE0005042456;;;56,78;0,00;0,00;EUR\n");
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert_eq!(parsed.rows.len(), 2);
        assert_eq!(parsed.rows[0].account_id, DEPOT_ACCOUNT_ID);
        assert_eq!(parsed.rows[0].row.amount_cents, -58720);
        assert_eq!(parsed.rows[1].account_id, VERRECHNUNGSKONTO_ACCOUNT_ID);
        assert_eq!(parsed.rows[1].row.amount_cents, 5678);
    }

    #[test]
    fn skips_non_executed_rows() {
        let mut contents = HEADER.to_string();
        contents.push_str("2024-10-23;13:10:35;Cancelled;\"SCAL1\";\"Foo\";Security;Buy;US1;1;1,00;-1,00;0,00;0,00;EUR\n");
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert!(parsed.rows.is_empty());
        assert!(parsed.skipped.is_empty());
    }

    #[test]
    fn flags_fractional_savings_plan_amounts_for_review() {
        let mut contents = HEADER.to_string();
        contents.push_str("2025-08-01;14:11:01;Executed;\"SCALT68r1eJqD4Q\";\"iShares BIC 50 (Dist)\";Security;Savings plan;IE00B1W57M07;3,505;21,395;-74,989475;0,00;0,00;EUR\n");
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(parsed.rows[0].row.amount_cents, -7499);
        assert_eq!(parsed.flagged_for_review.len(), 1);
    }

    #[test]
    fn non_eur_currency_is_skipped_with_reason() {
        let mut contents = HEADER.to_string();
        contents.push_str("2024-10-23;13:10:35;Executed;\"SCAL1\";\"Foo\";Security;Buy;US1;1;1,00;-1,00;0,00;0,00;USD\n");
        let file = write_fixture(&contents);

        let parsed = parse_csv(file.path()).unwrap();
        assert!(parsed.rows.is_empty());
        assert_eq!(parsed.skipped.len(), 1);
        assert!(parsed.skipped[0].reason.contains("non-EUR"));
    }
}
