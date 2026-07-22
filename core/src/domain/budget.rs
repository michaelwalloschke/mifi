//! Budgeting (SPEC.md §9): per-category monthly targets, spent, and state.

use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetState {
    OnTrack,
    Warning,
    Over,
}

/// The effective target for `category_id` as of `month` ("YYYY-MM"): the most recent
/// `budget_target` row with `effective_from_month <= month`. A `NULL` amount on that
/// row means the budget has ended as of that month — also `None` here.
pub fn effective_target_cents(conn: &Connection, category_id: i64, month: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT amount_cents FROM budget_target
         WHERE category_id = ?1 AND effective_from_month <= ?2
         ORDER BY effective_from_month DESC LIMIT 1",
        (category_id, month),
        |row| row.get(0),
    )
    .optional()
    .map(Option::flatten)
}

/// Spent = |net sum of the month's Splits| in the Category. A parent Category rolls up
/// its subcategories' spending; a subcategory counts only itself (SPEC.md §9).
pub fn spent_cents(conn: &Connection, category_id: i64, month: &str) -> rusqlite::Result<i64> {
    let is_parent: bool = conn.query_row(
        "SELECT parent_id IS NULL FROM category WHERE id = ?1",
        [category_id],
        |row| row.get(0),
    )?;

    let net: i64 = if is_parent {
        conn.query_row(
            "SELECT COALESCE(SUM(s.amount_cents), 0)
             FROM split s
             JOIN \"transaction\" t ON t.id = s.transaction_id
             WHERE strftime('%Y-%m', t.booking_date) = ?2
               AND (s.category_id = ?1 OR s.category_id IN (SELECT id FROM category WHERE parent_id = ?1))",
            (category_id, month),
            |row| row.get(0),
        )?
    } else {
        conn.query_row(
            "SELECT COALESCE(SUM(s.amount_cents), 0)
             FROM split s
             JOIN \"transaction\" t ON t.id = s.transaction_id
             WHERE strftime('%Y-%m', t.booking_date) = ?2 AND s.category_id = ?1",
            (category_id, month),
            |row| row.get(0),
        )?
    };
    Ok(net.abs())
}

/// on-track < 80%, warning >= 80%, over >= 100% — never pace-adjusted (SPEC.md §9).
pub fn state(target_cents: i64, spent_cents: i64) -> BudgetState {
    if target_cents <= 0 {
        return if spent_cents == 0 { BudgetState::OnTrack } else { BudgetState::Over };
    }
    let ratio = spent_cents as f64 / target_cents as f64;
    if ratio >= 1.0 {
        BudgetState::Over
    } else if ratio >= 0.8 {
        BudgetState::Warning
    } else {
        BudgetState::OnTrack
    }
}

/// Sum of expense Splits in Categories with no active target for `month` — the "Ohne
/// Budget" aggregate line (SPEC.md §9): a number only, no state.
pub fn unbudgeted_expense_cents(conn: &Connection, month: &str) -> rusqlite::Result<i64> {
    let net: i64 = conn.query_row(
        "SELECT COALESCE(SUM(s.amount_cents), 0)
         FROM split s
         JOIN \"transaction\" t ON t.id = s.transaction_id
         JOIN category c ON c.id = s.category_id
         WHERE c.kind = 'expense'
           AND strftime('%Y-%m', t.booking_date) = ?1
           AND NOT EXISTS (
               SELECT 1 FROM budget_target bt
               WHERE bt.category_id = s.category_id
                 AND bt.effective_from_month <= ?1
                 AND bt.amount_cents IS NOT NULL
               ORDER BY bt.effective_from_month DESC LIMIT 1
           )",
        [month],
        |row| row.get(0),
    )?;
    Ok(net.abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup(conn: &Connection) {
        conn.execute("INSERT INTO category (id, parent_id, name, kind) VALUES (1, NULL, 'Essen & Trinken', 'expense')", [])
            .unwrap();
        conn.execute("INSERT INTO category (id, parent_id, name, kind) VALUES (2, 1, 'Supermarkt', 'expense')", [])
            .unwrap();
        conn.execute("INSERT INTO category (id, parent_id, name, kind) VALUES (3, 1, 'Restaurant', 'expense')", [])
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
    fn parent_rolls_up_subcategory_spending() {
        let conn = db::open(":memory:").unwrap();
        setup(&conn);
        insert_split(&conn, 2, -3000, "2024-05-01");
        insert_split(&conn, 3, -2000, "2024-05-15");

        assert_eq!(spent_cents(&conn, 1, "2024-05").unwrap(), 5000);
        assert_eq!(spent_cents(&conn, 2, "2024-05").unwrap(), 3000);
    }

    #[test]
    fn refunds_reduce_spent() {
        let conn = db::open(":memory:").unwrap();
        setup(&conn);
        insert_split(&conn, 2, -3000, "2024-05-01");
        insert_split(&conn, 2, 1000, "2024-05-02"); // refund

        assert_eq!(spent_cents(&conn, 2, "2024-05").unwrap(), 2000);
    }

    #[test]
    fn effective_target_picks_most_recent_row_at_or_before_month() {
        let conn = db::open(":memory:").unwrap();
        setup(&conn);
        conn.execute(
            "INSERT INTO budget_target (category_id, amount_cents, effective_from_month) VALUES (1, 40000, '2024-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO budget_target (category_id, amount_cents, effective_from_month) VALUES (1, 50000, '2024-06')",
            [],
        )
        .unwrap();

        assert_eq!(effective_target_cents(&conn, 1, "2024-03").unwrap(), Some(40000));
        assert_eq!(effective_target_cents(&conn, 1, "2024-06").unwrap(), Some(50000));
        assert_eq!(effective_target_cents(&conn, 1, "2023-12").unwrap(), None);
    }

    #[test]
    fn null_amount_ends_the_budget() {
        let conn = db::open(":memory:").unwrap();
        setup(&conn);
        conn.execute(
            "INSERT INTO budget_target (category_id, amount_cents, effective_from_month) VALUES (1, 40000, '2024-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO budget_target (category_id, amount_cents, effective_from_month) VALUES (1, NULL, '2024-06')",
            [],
        )
        .unwrap();

        assert_eq!(effective_target_cents(&conn, 1, "2024-03").unwrap(), Some(40000));
        assert_eq!(effective_target_cents(&conn, 1, "2024-06").unwrap(), None);
    }

    #[test]
    fn state_thresholds() {
        assert_eq!(state(10000, 5000), BudgetState::OnTrack);
        assert_eq!(state(10000, 8000), BudgetState::Warning);
        assert_eq!(state(10000, 10000), BudgetState::Over);
        assert_eq!(state(10000, 12000), BudgetState::Over);
    }

    #[test]
    fn unbudgeted_expense_sums_categories_without_an_active_target() {
        let conn = db::open(":memory:").unwrap();
        setup(&conn);
        conn.execute(
            "INSERT INTO budget_target (category_id, amount_cents, effective_from_month) VALUES (2, 40000, '2024-01')",
            [],
        )
        .unwrap();
        insert_split(&conn, 2, -3000, "2024-05-01"); // budgeted category
        insert_split(&conn, 3, -2000, "2024-05-01"); // no target -> unbudgeted

        assert_eq!(unbudgeted_expense_cents(&conn, "2024-05").unwrap(), 2000);
    }
}
