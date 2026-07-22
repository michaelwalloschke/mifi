//! Finanzguru XLSX seed importer (SPEC.md §7, §11).
//!
//! Seeds Transaction/Split history, the Category taxonomy (verbatim from the export),
//! merchant memory, and NB token counts. Transfer-flagged rows are imported as
//! Transactions but left uncategorized — Transfer *link* creation is a separate
//! domain-engine step, not part of seeding.

use std::collections::HashMap;
use std::path::Path;

use calamine::{open_workbook_auto, DataType, Reader};
use chrono::NaiveDate;
use rusqlite::Connection;

use crate::import_hash::{self, OccurrenceCounter};
use crate::normalize::{normalize_merchant, tokenize};

/// A single parsed row from the Finanzguru export, independent of the XLSX reader so the
/// seeding logic below can be unit-tested without a real spreadsheet.
#[derive(Debug, Clone)]
pub struct RawRow {
    pub booking_date: NaiveDate,
    pub account_ref: String,
    pub amount_cents: i64,
    pub kontostand_cents: i64,
    pub counterparty_raw: String,
    pub purpose_raw: String,
    pub hauptkategorie: String,
    pub unterkategorie: String,
    pub is_transfer: bool,
    pub external_ref: Option<String>,
}

/// Maps the export's `Referenzkonto` values (IBANs / PayPal account ref) to mifi's fixed
/// account ids. These identifiers are personal — callers supply them at runtime (env vars,
/// a local untracked config file); they must never be hardcoded into the repo.
pub struct AccountRefs {
    pub giro: String,
    pub tagesgeld: String,
    pub paypal: String,
}

impl AccountRefs {
    fn resolve(&self, account_ref: &str) -> Option<i64> {
        if account_ref == self.giro {
            Some(1)
        } else if account_ref == self.tagesgeld {
            Some(2)
        } else if account_ref == self.paypal {
            Some(5)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SeedSummary {
    pub transactions_imported: usize,
    pub categories_created: usize,
    pub merchant_rules_created: usize,
    pub balance_snapshots_created: usize,
}

#[derive(Debug)]
pub enum SeedError {
    Xlsx(String),
    UnknownAccountRef(String),
    Sqlite(rusqlite::Error),
}

impl From<rusqlite::Error> for SeedError {
    fn from(e: rusqlite::Error) -> Self {
        SeedError::Sqlite(e)
    }
}

/// Parses the Finanzguru XLSX export at `path` into row order as they appear in the file.
pub fn parse_xlsx(path: impl AsRef<Path>) -> Result<Vec<RawRow>, SeedError> {
    let mut workbook: calamine::Sheets<_> =
        open_workbook_auto(path.as_ref()).map_err(|e| SeedError::Xlsx(e.to_string()))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| SeedError::Xlsx("workbook has no sheets".to_string()))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| SeedError::Xlsx(e.to_string()))?;

    let mut rows = Vec::new();
    for row in range.rows().skip(1) {
        let cell = |i: usize| row.get(i).cloned().unwrap_or_default();
        let text = |i: usize| cell(i).as_string().unwrap_or_default();

        let booking_date = cell(0)
            .as_date()
            .ok_or_else(|| SeedError::Xlsx("row missing Buchungstag".to_string()))?;
        let amount_cents = (cell(3).as_f64().unwrap_or(0.0) * 100.0).round() as i64;
        let kontostand_cents = (cell(4).as_f64().unwrap_or(0.0) * 100.0).round() as i64;
        let external_ref = text(25);

        rows.push(RawRow {
            booking_date,
            account_ref: text(1),
            amount_cents,
            kontostand_cents,
            counterparty_raw: text(6),
            purpose_raw: text(8),
            hauptkategorie: text(12),
            unterkategorie: text(13),
            is_transfer: text(17) == "ja",
            external_ref: if external_ref.is_empty() { None } else { Some(external_ref) },
        });
    }
    Ok(rows)
}

/// Imports parsed rows into `conn` inside a single transaction: taxonomy, transactions,
/// splits, merchant memory, NB token counts, and month-end balance Snapshots.
pub fn seed(conn: &mut Connection, rows: &[RawRow], account_refs: &AccountRefs) -> Result<SeedSummary, SeedError> {
    let tx = conn.transaction()?;
    let mut summary = SeedSummary::default();

    // 1. Taxonomy: every unique (Hauptkategorie, Unterkategorie) pair, verbatim.
    let mut parent_ids: HashMap<String, i64> = HashMap::new();
    let mut category_ids: HashMap<(String, String), i64> = HashMap::new();
    for row in rows {
        if !parent_ids.contains_key(&row.hauptkategorie) {
            let kind = if row.hauptkategorie == "Einnahmen" { "income" } else { "expense" };
            tx.execute(
                "INSERT INTO category (parent_id, name, kind) VALUES (NULL, ?1, ?2)",
                (&row.hauptkategorie, kind),
            )?;
            parent_ids.insert(row.hauptkategorie.clone(), tx.last_insert_rowid());
            summary.categories_created += 1;
        }
        let key = (row.hauptkategorie.clone(), row.unterkategorie.clone());
        if !category_ids.contains_key(&key) {
            let parent_id = parent_ids[&row.hauptkategorie];
            let kind = if row.hauptkategorie == "Einnahmen" { "income" } else { "expense" };
            tx.execute(
                "INSERT INTO category (parent_id, name, kind) VALUES (?1, ?2, ?3)",
                (parent_id, &row.unterkategorie, kind),
            )?;
            category_ids.insert(key.clone(), tx.last_insert_rowid());
            summary.categories_created += 1;
        }
    }

    // 2. Transactions + Splits, in file order. Transfer-flagged rows import uncategorized.
    let mut occurrences = OccurrenceCounter::new();
    let mut merchant_categories: HashMap<String, HashMap<i64, usize>> = HashMap::new();
    let mut token_counts: HashMap<(String, i64), i64> = HashMap::new();
    let mut month_end_balance: HashMap<(i64, String), (NaiveDate, i64)> = HashMap::new();

    for row in rows {
        let account_id = account_refs
            .resolve(&row.account_ref)
            .ok_or_else(|| SeedError::UnknownAccountRef(row.account_ref.clone()))?;

        let booking_date_str = row.booking_date.format("%Y-%m-%d").to_string();
        let import_hash = import_hash::compute(
            &booking_date_str,
            row.amount_cents,
            &row.counterparty_raw,
            &row.purpose_raw,
        );
        let occurrence_index = occurrences.next_index(account_id, &import_hash);
        let normalized_counterparty = normalize_merchant(&row.counterparty_raw);

        tx.execute(
            "INSERT INTO \"transaction\" (
                account_id, booking_date, amount_cents,
                counterparty_raw, counterparty_normalized,
                purpose_raw, purpose_normalized,
                import_hash, occurrence_index, source, external_ref
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'finanzguru-seed', ?10)",
            rusqlite::params![
                account_id,
                booking_date_str,
                row.amount_cents,
                row.counterparty_raw,
                normalized_counterparty,
                row.purpose_raw,
                crate::normalize::normalize_purpose(&row.purpose_raw),
                import_hash,
                occurrence_index,
                row.external_ref,
            ],
        )?;
        let transaction_id = tx.last_insert_rowid();
        summary.transactions_imported += 1;

        let category_id = if row.is_transfer {
            None
        } else {
            category_ids
                .get(&(row.hauptkategorie.clone(), row.unterkategorie.clone()))
                .copied()
        };

        tx.execute(
            "INSERT INTO split (transaction_id, amount_cents, category_id, category_source)
             VALUES (?1, ?2, ?3, 'auto')",
            (transaction_id, row.amount_cents, category_id),
        )?;

        if let Some(category_id) = category_id {
            *merchant_categories
                .entry(normalized_counterparty.clone())
                .or_default()
                .entry(category_id)
                .or_insert(0) += 1;

            for token in tokenize(&row.counterparty_raw)
                .into_iter()
                .chain(tokenize(&row.purpose_raw))
            {
                *token_counts.entry((token, category_id)).or_insert(0) += 1;
            }
        }

        let month_key = (account_id, row.booking_date.format("%Y-%m").to_string());
        month_end_balance
            .entry(month_key)
            .and_modify(|(date, balance)| {
                if row.booking_date > *date {
                    *date = row.booking_date;
                    *balance = row.kontostand_cents;
                }
            })
            .or_insert((row.booking_date, row.kontostand_cents));
    }

    // 3. Merchant memory: purity-gated (>=2 rows, >=80% one category), tunable at import.
    for (merchant, categories) in &merchant_categories {
        let total: usize = categories.values().sum();
        if total < 2 {
            continue;
        }
        let (&best_category, &best_count) = categories
            .iter()
            .max_by_key(|(_, count)| **count)
            .expect("merchant_categories entries are never empty");
        if best_count as f64 / total as f64 >= 0.8 {
            tx.execute(
                "INSERT INTO merchant_rule (normalized_merchant, category_id) VALUES (?1, ?2)",
                (merchant, best_category),
            )?;
            summary.merchant_rules_created += 1;
        }
    }

    // 4. NB token counts: every non-transfer row contributes, not just purity-gated merchants.
    for ((token, category_id), count) in &token_counts {
        tx.execute(
            "INSERT INTO nb_token_count (token, category_id, count) VALUES (?1, ?2, ?3)
             ON CONFLICT (token, category_id) DO UPDATE SET count = count + excluded.count",
            (token, category_id, count),
        )?;
    }

    // 5. Month-end balance Snapshots from Kontostand — seeds the cash net-worth curve (§11.4).
    for ((account_id, _month), (date, balance_cents)) in &month_end_balance {
        tx.execute(
            "INSERT INTO balance_snapshot (account_id, date, balance_cents) VALUES (?1, ?2, ?3)
             ON CONFLICT (account_id, date) DO UPDATE SET balance_cents = excluded.balance_cents",
            (account_id, date.format("%Y-%m-%d").to_string(), balance_cents),
        )?;
        summary.balance_snapshots_created += 1;
    }

    tx.commit()?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[allow(clippy::too_many_arguments)]
    fn row(
        date: (i32, u32, u32),
        account_ref: &str,
        amount_cents: i64,
        kontostand_cents: i64,
        counterparty: &str,
        purpose: &str,
        main: &str,
        sub: &str,
        is_transfer: bool,
    ) -> RawRow {
        RawRow {
            booking_date: NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap(),
            account_ref: account_ref.to_string(),
            amount_cents,
            kontostand_cents,
            counterparty_raw: counterparty.to_string(),
            purpose_raw: purpose.to_string(),
            hauptkategorie: main.to_string(),
            unterkategorie: sub.to_string(),
            is_transfer,
            external_ref: None,
        }
    }

    fn refs() -> AccountRefs {
        AccountRefs {
            giro: "GIRO-IBAN".to_string(),
            tagesgeld: "TAGESGELD-IBAN".to_string(),
            paypal: "paypal@example.com".to_string(),
        }
    }

    fn open_test_db() -> Connection {
        db::open(":memory:").unwrap()
    }

    #[test]
    fn builds_taxonomy_from_unique_pairs_with_correct_kind() {
        let mut conn = open_test_db();
        let rows = vec![
            row((2022, 5, 30), "GIRO-IBAN", -840, -10000, "Lotto24", "x", "Freizeit", "Sonstige Freizeitausgaben", false),
            row((2022, 5, 31), "GIRO-IBAN", 276533, -5000, "Employer", "salary", "Einnahmen", "Lohn / Gehalt", false),
        ];
        let summary = seed(&mut conn, &rows, &refs()).unwrap();
        assert_eq!(summary.categories_created, 4); // 2 parents + 2 children

        let kind: String = conn
            .query_row("SELECT kind FROM category WHERE name = 'Einnahmen'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(kind, "income");
        let kind: String = conn
            .query_row("SELECT kind FROM category WHERE name = 'Freizeit'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(kind, "expense");
    }

    #[test]
    fn transfer_rows_import_uncategorized() {
        let mut conn = open_test_db();
        let rows = vec![row(
            (2022, 5, 30),
            "GIRO-IBAN",
            -5000,
            -10000,
            "Own Account",
            "sparplan",
            "Sparen",
            "ETF-Sparplan",
            true,
        )];
        seed(&mut conn, &rows, &refs()).unwrap();

        let category_id: Option<i64> = conn
            .query_row("SELECT category_id FROM split", [], |r| r.get(0))
            .unwrap();
        assert_eq!(category_id, None);
    }

    #[test]
    fn duplicate_same_day_rows_get_incrementing_occurrence_index() {
        let mut conn = open_test_db();
        let rows = vec![
            row((2022, 5, 30), "GIRO-IBAN", -500, -10000, "Bakery", "bread", "Essen & Trinken", "Bäcker", false),
            row((2022, 5, 30), "GIRO-IBAN", -500, -10050, "Bakery", "bread", "Essen & Trinken", "Bäcker", false),
        ];
        seed(&mut conn, &rows, &refs()).unwrap();

        let mut stmt = conn
            .prepare("SELECT occurrence_index FROM \"transaction\" ORDER BY occurrence_index")
            .unwrap();
        let indices: Vec<i64> = stmt.query_map([], |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn merchant_rule_seeded_only_when_purity_gate_passes() {
        let mut conn = open_test_db();
        let rows = vec![
            row((2022, 1, 1), "GIRO-IBAN", -1000, -1000, "HelloFre sh", "p", "Essen & Trinken", "Lieferservice", false),
            row((2022, 1, 8), "GIRO-IBAN", -1000, -2000, "HelloFresh", "p", "Essen & Trinken", "Lieferservice", false),
            row((2022, 1, 15), "GIRO-IBAN", -1000, -3000, "HelloFresh", "p", "Essen & Trinken", "Lieferservice", false),
            row((2022, 1, 22), "GIRO-IBAN", -1000, -4000, "HelloFresh", "p", "Essen & Trinken", "Lieferservice", false),
            row((2022, 1, 29), "GIRO-IBAN", -1000, -5000, "HelloFresh", "p", "Sonstiges", "Sonstige Ausgaben", false),
            // A merchant seen only once never qualifies (< 2 rows).
            row((2022, 1, 1), "GIRO-IBAN", -200, -1200, "OneOff Shop", "p", "Lifestyle", "Shopping", false),
        ];
        let summary = seed(&mut conn, &rows, &refs()).unwrap();
        assert_eq!(summary.merchant_rules_created, 1);

        let merchant: String = conn
            .query_row("SELECT normalized_merchant FROM merchant_rule", [], |r| r.get(0))
            .unwrap();
        assert_eq!(merchant, "hellofresh");
    }

    #[test]
    fn nb_token_counts_include_all_non_transfer_rows() {
        let mut conn = open_test_db();
        let rows = vec![
            row((2022, 1, 1), "GIRO-IBAN", -1000, -1000, "Rewe Markt", "einkauf", "Essen & Trinken", "Supermarkt", false),
            row((2022, 1, 2), "GIRO-IBAN", -1000, -2000, "Rewe Markt", "einkauf", "Essen & Trinken", "Supermarkt", false),
        ];
        seed(&mut conn, &rows, &refs()).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT count FROM nb_token_count WHERE token = 'einkauf'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn balance_snapshot_keeps_latest_row_per_account_month() {
        let mut conn = open_test_db();
        let rows = vec![
            row((2022, 5, 30), "GIRO-IBAN", -840, -10000, "A", "p", "Freizeit", "Sub", false),
            row((2022, 5, 31), "GIRO-IBAN", -100, -15000, "B", "p", "Freizeit", "Sub", false),
            row((2022, 6, 1), "GIRO-IBAN", -100, -20000, "C", "p", "Freizeit", "Sub", false),
        ];
        let summary = seed(&mut conn, &rows, &refs()).unwrap();
        assert_eq!(summary.balance_snapshots_created, 2);

        let balance: i64 = conn
            .query_row(
                "SELECT balance_cents FROM balance_snapshot WHERE date = '2022-05-31'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(balance, -15000);
    }

    #[test]
    fn unknown_account_ref_is_a_hard_error() {
        let mut conn = open_test_db();
        let rows = vec![row((2022, 1, 1), "SOME-OTHER-REF", -100, -100, "A", "p", "Freizeit", "Sub", false)];
        let result = seed(&mut conn, &rows, &refs());
        assert!(matches!(result, Err(SeedError::UnknownAccountRef(_))));
    }
}
