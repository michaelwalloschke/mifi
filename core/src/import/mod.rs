//! Shared CSV import scaffolding (SPEC.md §6): German decimal parsing and the
//! commit path every format-specific importer (PayPal, Scalable, …) feeds into.

pub mod paypal;
pub mod scalable;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::import_hash::{self, OccurrenceCounter};
use crate::normalize::normalize_purpose;

/// One row already decoded from its source format, ready to become a Transaction.
#[derive(Debug, Clone)]
pub struct NormalizedRow {
    pub booking_date: NaiveDate,
    pub amount_cents: i64,
    pub counterparty_raw: String,
    pub purpose_raw: String,
    pub external_ref: Option<String>,
    /// Inert display metadata for foreign-currency originals (SPEC.md §4, CONTEXT.md
    /// Amount) — e.g. `{"original_currency":"USD","original_amount_cents":-1099}`. Never
    /// computed with; EUR is the only currency mifi does arithmetic in.
    pub fx_metadata: Option<String>,
}

/// A row that couldn't be parsed, or that parsed but needs human review — never a
/// silent drop (SPEC.md §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowNote {
    pub row_number: usize,
    pub reason: String,
}

/// Output of parsing one CSV file: valid rows plus every skipped/flagged row with why.
#[derive(Debug, Default)]
pub struct ParsedCsv {
    pub rows: Vec<NormalizedRow>,
    pub skipped: Vec<RowNote>,
    pub flagged_for_review: Vec<RowNote>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub imported: usize,
    pub duplicate_external_ref_skipped: usize,
}

/// Parses a German-formatted decimal amount (`"1.234,56"`, `"-74,989475"`) into integer
/// cents, rounding half-away-from-zero when more than 2 decimal places are present.
pub fn parse_german_decimal_cents(raw: &str) -> Result<i64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("empty amount".to_string());
    }
    let normalized = trimmed.replace('.', "").replace(',', ".");
    let decimal: Decimal = normalized
        .parse()
        .map_err(|_| format!("invalid amount: {raw:?}"))?;
    let cents = (decimal * Decimal::from(100)).round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    cents
        .to_i64()
        .ok_or_else(|| format!("amount out of range: {raw:?}"))
}

/// Commits already-parsed rows for one account/source pair inside a single DB transaction:
/// idempotent on `(source, external_ref)` first, then Import Hash + occurrence index.
/// CSV-imported Splits land uncategorized — categorization runs as a separate post-commit
/// sweep (SPEC.md §7), never as part of the import itself.
pub fn commit(conn: &mut Connection, account_id: i64, source: &str, rows: &[NormalizedRow]) -> rusqlite::Result<ImportSummary> {
    let tx = conn.transaction()?;
    let mut occurrences = OccurrenceCounter::seeded_from_db(&tx, account_id)?;
    let mut summary = ImportSummary::default();

    for row in rows {
        if let Some(external_ref) = &row.external_ref {
            let already_imported: Option<i64> = tx
                .query_row(
                    "SELECT id FROM \"transaction\" WHERE source = ?1 AND external_ref = ?2",
                    (source, external_ref),
                    |r| r.get(0),
                )
                .optional()?;
            if already_imported.is_some() {
                summary.duplicate_external_ref_skipped += 1;
                continue;
            }
        }

        let booking_date_str = row.booking_date.format("%Y-%m-%d").to_string();
        let import_hash = import_hash::compute(
            &booking_date_str,
            row.amount_cents,
            &row.counterparty_raw,
            &row.purpose_raw,
        );
        let occurrence_index = occurrences.next_index(account_id, &import_hash);

        tx.execute(
            "INSERT INTO \"transaction\" (
                account_id, booking_date, amount_cents,
                counterparty_raw, counterparty_normalized,
                purpose_raw, purpose_normalized,
                import_hash, occurrence_index, source, external_ref, fx_metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                account_id,
                booking_date_str,
                row.amount_cents,
                row.counterparty_raw,
                crate::normalize::normalize_merchant(&row.counterparty_raw),
                row.purpose_raw,
                normalize_purpose(&row.purpose_raw),
                import_hash,
                occurrence_index,
                source,
                row.external_ref,
                row.fx_metadata,
            ],
        )?;
        let transaction_id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO split (transaction_id, amount_cents, category_id, category_source)
             VALUES (?1, ?2, NULL, 'auto')",
            (transaction_id, row.amount_cents),
        )?;
        summary.imported += 1;
    }

    tx.commit()?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_german_thousands_and_decimal_comma() {
        assert_eq!(parse_german_decimal_cents("1.234,56"), Ok(123456));
        assert_eq!(parse_german_decimal_cents("-41,00"), Ok(-4100));
        assert_eq!(parse_german_decimal_cents("7,34"), Ok(734));
    }

    #[test]
    fn rounds_more_than_two_decimals_half_away_from_zero() {
        assert_eq!(parse_german_decimal_cents("-74,989475"), Ok(-7499));
        assert_eq!(parse_german_decimal_cents("0,005"), Ok(1));
        assert_eq!(parse_german_decimal_cents("-0,005"), Ok(-1));
    }

    #[test]
    fn rejects_empty_or_invalid_amount() {
        assert!(parse_german_decimal_cents("").is_err());
        assert!(parse_german_decimal_cents("abc").is_err());
    }

    #[test]
    fn commit_is_idempotent_on_external_ref() {
        let mut conn = crate::db::open(":memory:").unwrap();
        let rows = vec![NormalizedRow {
            booking_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            amount_cents: -500,
            counterparty_raw: "Steam".to_string(),
            purpose_raw: "game".to_string(),
            external_ref: Some("TXN123".to_string()),
            fx_metadata: None,
        }];

        let first = commit(&mut conn, 5, "csv-paypal", &rows).unwrap();
        assert_eq!(first.imported, 1);
        let second = commit(&mut conn, 5, "csv-paypal", &rows).unwrap();
        assert_eq!(second.imported, 0);
        assert_eq!(second.duplicate_external_ref_skipped, 1);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM \"transaction\"", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn commit_seeds_occurrence_index_from_prior_runs() {
        let mut conn = crate::db::open(":memory:").unwrap();
        let row = |external_ref: &str| NormalizedRow {
            booking_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            amount_cents: -500,
            counterparty_raw: "Bakery".to_string(),
            purpose_raw: "bread".to_string(),
            external_ref: Some(external_ref.to_string()),
            fx_metadata: None,
        };

        commit(&mut conn, 5, "csv-paypal", &[row("A")]).unwrap();
        commit(&mut conn, 5, "csv-paypal", &[row("B")]).unwrap();

        let mut stmt = conn
            .prepare("SELECT occurrence_index FROM \"transaction\" ORDER BY external_ref")
            .unwrap();
        let indices: Vec<i64> = stmt.query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(indices, vec![0, 1]);
    }
}
