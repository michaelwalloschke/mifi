//! Net Worth (SPEC.md §10, CONTEXT.md Net Worth): derived, never stored — sum of latest
//! balance Snapshots plus depot Position valuations at a date.

use rusqlite::Connection;

/// Sum of the latest balance Snapshot per account, plus the latest Position valuation
/// per (account, isin), each at or before `date` ("YYYY-MM-DD").
pub fn net_worth_cents(conn: &Connection, date: &str) -> rusqlite::Result<i64> {
    let cash: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_cents), 0) FROM (
            SELECT balance_cents,
                   ROW_NUMBER() OVER (PARTITION BY account_id ORDER BY date DESC) AS rn
            FROM balance_snapshot WHERE date <= ?1
         ) WHERE rn = 1",
        [date],
        |row| row.get(0),
    )?;

    let positions: i64 = conn.query_row(
        "SELECT COALESCE(SUM(valuation_cents), 0) FROM (
            SELECT valuation_cents,
                   ROW_NUMBER() OVER (PARTITION BY account_id, isin ORDER BY date DESC) AS rn
            FROM position_snapshot WHERE date <= ?1
         ) WHERE rn = 1",
        [date],
        |row| row.get(0),
    )?;

    Ok(cash + positions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn sums_latest_balance_per_account_at_or_before_date() {
        let conn = db::open(":memory:").unwrap();
        conn.execute("INSERT INTO balance_snapshot (account_id, date, balance_cents) VALUES (1, '2024-05-01', 10000)", [])
            .unwrap();
        conn.execute("INSERT INTO balance_snapshot (account_id, date, balance_cents) VALUES (1, '2024-05-31', 12000)", [])
            .unwrap();
        conn.execute("INSERT INTO balance_snapshot (account_id, date, balance_cents) VALUES (2, '2024-05-15', 5000)", [])
            .unwrap();

        assert_eq!(net_worth_cents(&conn, "2024-05-20").unwrap(), 10000 + 5000);
        assert_eq!(net_worth_cents(&conn, "2024-06-01").unwrap(), 12000 + 5000);
        assert_eq!(net_worth_cents(&conn, "2024-04-30").unwrap(), 0);
    }

    #[test]
    fn adds_latest_position_valuation_per_isin() {
        let conn = db::open(":memory:").unwrap();
        conn.execute("INSERT INTO balance_snapshot (account_id, date, balance_cents) VALUES (4, '2024-05-01', 1000)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO position_snapshot (account_id, isin, date, quantity, price, valuation_cents) VALUES (3, 'US1', '2024-05-01', 10, 100, 100000)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO position_snapshot (account_id, isin, date, quantity, price, valuation_cents) VALUES (3, 'US1', '2024-05-15', 10, 110, 110000)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO position_snapshot (account_id, isin, date, quantity, price, valuation_cents) VALUES (3, 'US2', '2024-05-10', 5, 50, 25000)",
            [],
        )
        .unwrap();

        assert_eq!(net_worth_cents(&conn, "2024-05-20").unwrap(), 1000 + 110000 + 25000);
    }
}
