//! Übersicht stat tiles (SPEC.md §13): Einnahmen, Ausgaben, Sparquote, Puffer for a
//! month, plus the trailing-months series each tile sparklines.
//!
//! Puffer here is a deliberate simplification — Einnahmen minus Ausgaben, i.e. leftover
//! cash flow before any explicit savings transfer. The prototype's Sankey additionally
//! nets out transfers to a "Sparziele" bucket; that needs a savings-goal model this
//! build doesn't have yet, so the stat tile stays at the simpler, honest definition.

use chrono::{Months, NaiveDate};
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq)]
pub struct MonthlyOverview {
    pub month: String,
    pub einnahmen_cents: i64,
    pub ausgaben_cents: i64,
    pub sparquote_percent: f64,
    pub puffer_cents: i64,
}

fn parse_month(month: &str) -> NaiveDate {
    NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .unwrap_or_else(|_| panic!("invalid month string: {month:?}, expected YYYY-MM"))
}

fn month_string(date: NaiveDate) -> String {
    date.format("%Y-%m").to_string()
}

/// The month `n` calendar months before `month` ("YYYY-MM").
pub fn months_before(month: &str, n: u32) -> String {
    let date = parse_month(month);
    month_string(date.checked_sub_months(Months::new(n)).expect("month arithmetic should not overflow"))
}

fn category_kind_sum_cents(conn: &Connection, month: &str, kind: &str) -> rusqlite::Result<i64> {
    let net: i64 = conn.query_row(
        "SELECT COALESCE(SUM(s.amount_cents), 0)
         FROM split s
         JOIN \"transaction\" t ON t.id = s.transaction_id
         JOIN category c ON c.id = s.category_id
         WHERE c.kind = ?2 AND strftime('%Y-%m', t.booking_date) = ?1",
        (month, kind),
        |row| row.get(0),
    )?;
    Ok(net.abs())
}

/// Computes the four stat-tile figures for one month.
pub fn compute_month(conn: &Connection, month: &str) -> rusqlite::Result<MonthlyOverview> {
    let einnahmen_cents = category_kind_sum_cents(conn, month, "income")?;
    let ausgaben_cents = category_kind_sum_cents(conn, month, "expense")?;
    let sparquote_percent = if einnahmen_cents == 0 {
        0.0
    } else {
        (einnahmen_cents - ausgaben_cents) as f64 / einnahmen_cents as f64 * 100.0
    };
    let puffer_cents = einnahmen_cents - ausgaben_cents;

    Ok(MonthlyOverview {
        month: month.to_string(),
        einnahmen_cents,
        ausgaben_cents,
        sparquote_percent,
        puffer_cents,
    })
}

/// The trailing `months` MonthlyOverviews ending at (and including) `end_month`, oldest
/// first — feeds each stat tile's 12-month sparkline.
pub fn compute_series(conn: &Connection, end_month: &str, months: u32) -> rusqlite::Result<Vec<MonthlyOverview>> {
    (0..months)
        .rev()
        .map(|i| compute_month(conn, &months_before(end_month, i)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_categories(conn: &Connection) {
        conn.execute("INSERT INTO category (id, parent_id, name, kind) VALUES (1, NULL, 'Einnahmen', 'income')", [])
            .unwrap();
        conn.execute("INSERT INTO category (id, parent_id, name, kind) VALUES (2, NULL, 'Wohnen', 'expense')", [])
            .unwrap();
    }

    fn insert_split(conn: &Connection, category_id: i64, amount_cents: i64, date: &str) {
        conn.execute(
            "INSERT INTO \"transaction\" (
                account_id, booking_date, amount_cents, counterparty_raw, counterparty_normalized,
                purpose_raw, purpose_normalized, import_hash, occurrence_index, source
            ) VALUES (1, ?1, ?2, 'X', 'x', 'Y', 'y', ?3, 0, 'csv-paypal')",
            (date, amount_cents, format!("h-{date}-{amount_cents}-{category_id}")),
        )
        .unwrap();
        let txn_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO split (transaction_id, amount_cents, category_id, category_source) VALUES (?1, ?2, ?3, 'auto')",
            (txn_id, amount_cents, category_id),
        )
        .unwrap();
    }

    #[test]
    fn months_before_crosses_year_boundary() {
        assert_eq!(months_before("2024-01", 1), "2023-12");
        assert_eq!(months_before("2024-03", 3), "2023-12");
        assert_eq!(months_before("2024-05", 0), "2024-05");
    }

    #[test]
    fn computes_einnahmen_ausgaben_sparquote_and_puffer() {
        let conn = db::open(":memory:").unwrap();
        setup_categories(&conn);
        insert_split(&conn, 1, 420000, "2024-05-01"); // Einnahmen 4200
        insert_split(&conn, 2, -294000, "2024-05-15"); // Ausgaben 2940 (abs)

        let overview = compute_month(&conn, "2024-05").unwrap();
        assert_eq!(overview.einnahmen_cents, 420000);
        assert_eq!(overview.ausgaben_cents, 294000);
        assert_eq!(overview.puffer_cents, 126000);
        assert!((overview.sparquote_percent - 30.0).abs() < 0.01);
    }

    #[test]
    fn zero_einnahmen_gives_zero_sparquote_not_a_crash() {
        let conn = db::open(":memory:").unwrap();
        setup_categories(&conn);
        insert_split(&conn, 2, -1000, "2024-05-01");

        let overview = compute_month(&conn, "2024-05").unwrap();
        assert_eq!(overview.sparquote_percent, 0.0);
    }

    #[test]
    fn compute_series_returns_oldest_first_over_the_requested_span() {
        let conn = db::open(":memory:").unwrap();
        setup_categories(&conn);
        insert_split(&conn, 1, 100000, "2024-03-01");
        insert_split(&conn, 1, 200000, "2024-05-01");

        let series = compute_series(&conn, "2024-05", 3).unwrap();
        assert_eq!(series.len(), 3);
        assert_eq!(series[0].month, "2024-03");
        assert_eq!(series[0].einnahmen_cents, 100000);
        assert_eq!(series[1].month, "2024-04");
        assert_eq!(series[1].einnahmen_cents, 0);
        assert_eq!(series[2].month, "2024-05");
        assert_eq!(series[2].einnahmen_cents, 200000);
    }
}
