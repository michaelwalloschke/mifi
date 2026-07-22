//! Transfer detection (SPEC.md §3, CONTEXT.md Transfer).
//!
//! Auto-detect: amount + sign + ±4-day window + a different own Account. A confident
//! (unambiguous) pair auto-links; anything with more than one candidate on either side
//! is surfaced instead of guessed. Linking always nulls both legs' category — Transfer
//! legs are uncategorized by definition, whatever a prior categorization pass assigned.

use rusqlite::Connection;

const WINDOW_DAYS: i64 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub transaction_id: i64,
    pub account_id: i64,
    pub booking_date: chrono::NaiveDate,
    pub amount_cents: i64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TransferDetectionSummary {
    pub auto_linked: usize,
    pub ambiguous: Vec<(i64, i64)>,
}

fn unlinked_candidates(conn: &Connection) -> rusqlite::Result<Vec<Candidate>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.account_id, t.booking_date, t.amount_cents
         FROM \"transaction\" t
         WHERE t.id NOT IN (SELECT leg_a_txn_id FROM transfer UNION SELECT leg_b_txn_id FROM transfer)",
    )?;
    let rows = stmt.query_map([], |row| {
        let date_str: String = row.get(2)?;
        Ok(Candidate {
            transaction_id: row.get(0)?,
            account_id: row.get(1)?,
            booking_date: chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap(),
            amount_cents: row.get(3)?,
        })
    })?;
    rows.collect()
}

/// Finds, for every unlinked transaction, every other unlinked transaction that could be
/// its Transfer counterpart: opposite sign, equal magnitude, different account, within
/// the ±4-day window.
fn find_counterpart_candidates(candidates: &[Candidate]) -> std::collections::HashMap<i64, Vec<i64>> {
    let mut result: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for a in candidates {
        if a.amount_cents == 0 {
            continue;
        }
        for b in candidates {
            if a.transaction_id == b.transaction_id {
                continue;
            }
            if b.amount_cents != -a.amount_cents {
                continue;
            }
            if b.account_id == a.account_id {
                continue;
            }
            let diff = (a.booking_date - b.booking_date).num_days().abs();
            if diff > WINDOW_DAYS {
                continue;
            }
            result.entry(a.transaction_id).or_default().push(b.transaction_id);
        }
    }
    result
}

/// Runs Transfer auto-detection over every unlinked Transaction and links confident
/// (mutually-unique) pairs. Re-run at the end of every Sync Run (SPEC.md §5).
pub fn detect(conn: &mut Connection) -> rusqlite::Result<TransferDetectionSummary> {
    let candidates = unlinked_candidates(conn)?;
    let counterparts = find_counterpart_candidates(&candidates);

    let mut summary = TransferDetectionSummary::default();
    let mut linked = std::collections::HashSet::new();

    let mut pairs: Vec<(i64, i64)> = counterparts
        .iter()
        .filter(|(_, cands)| cands.len() == 1)
        .filter_map(|(&a, cands)| {
            let b = cands[0];
            // Mutual uniqueness: b's only candidate must be a.
            if counterparts.get(&b).map(|c| c.as_slice()) == Some(&[a][..]) {
                Some((a.min(b), a.max(b)))
            } else {
                None
            }
        })
        .collect();
    pairs.sort_unstable();
    pairs.dedup();

    let tx = conn.transaction()?;
    for (leg_a, leg_b) in pairs {
        if linked.contains(&leg_a) || linked.contains(&leg_b) {
            continue;
        }
        link(&tx, leg_a, leg_b, "auto")?;
        linked.insert(leg_a);
        linked.insert(leg_b);
        summary.auto_linked += 1;
    }

    for (&a, cands) in &counterparts {
        if cands.len() > 1 && !linked.contains(&a) {
            for &b in cands {
                let pair = (a.min(b), a.max(b));
                if !summary.ambiguous.contains(&pair) {
                    summary.ambiguous.push(pair);
                }
            }
        }
    }
    tx.commit()?;

    Ok(summary)
}

fn link(conn: &Connection, leg_a: i64, leg_b: i64, link_source: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO transfer (leg_a_txn_id, leg_b_txn_id, link_source) VALUES (?1, ?2, ?3)",
        (leg_a, leg_b, link_source),
    )?;
    conn.execute(
        "UPDATE split SET category_id = NULL WHERE transaction_id IN (?1, ?2)",
        (leg_a, leg_b),
    )?;
    Ok(())
}

/// Manual link — always allowed, regardless of ambiguity (SPEC.md §3).
pub fn confirm(conn: &mut Connection, leg_a: i64, leg_b: i64) -> rusqlite::Result<()> {
    link(conn, leg_a, leg_b, "user")
}

/// Manual unlink — the freed legs fall back to being ordinary (uncategorized) Transactions.
pub fn unlink(conn: &mut Connection, transfer_id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM transfer WHERE id = ?1", [transfer_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn insert_txn(conn: &Connection, account_id: i64, date: &str, amount_cents: i64) -> i64 {
        conn.execute(
            "INSERT INTO \"transaction\" (
                account_id, booking_date, amount_cents, counterparty_raw, counterparty_normalized,
                purpose_raw, purpose_normalized, import_hash, occurrence_index, source
            ) VALUES (?1, ?2, ?3, 'X', 'x', 'Y', 'y', ?4, 0, 'csv-paypal')",
            (account_id, date, amount_cents, format!("hash-{account_id}-{date}-{amount_cents}")),
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO split (transaction_id, amount_cents, category_id, category_source) VALUES (?1, ?2, NULL, 'auto')",
            (id, amount_cents),
        )
        .unwrap();
        id
    }

    #[test]
    fn auto_links_unambiguous_opposite_pair_across_accounts() {
        let mut conn = db::open(":memory:").unwrap();
        conn.execute("INSERT INTO category (id, parent_id, name, kind) VALUES (99, NULL, 'Test', 'expense')", [])
            .unwrap();
        let a = insert_txn(&conn, 1, "2024-05-01", -5000);
        let b = insert_txn(&conn, 4, "2024-05-02", 5000);
        conn.execute("UPDATE split SET category_id = 99 WHERE transaction_id IN (?1, ?2)", (a, b)).unwrap();

        let summary = detect(&mut conn).unwrap();
        assert_eq!(summary.auto_linked, 1);
        assert!(summary.ambiguous.is_empty());

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM transfer", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);

        for id in [a, b] {
            let category_id: Option<i64> = conn
                .query_row("SELECT category_id FROM split WHERE transaction_id = ?1", [id], |r| r.get(0))
                .unwrap();
            assert_eq!(category_id, None);
        }
    }

    #[test]
    fn does_not_link_same_account() {
        let mut conn = db::open(":memory:").unwrap();
        insert_txn(&conn, 1, "2024-05-01", -5000);
        insert_txn(&conn, 1, "2024-05-02", 5000);

        let summary = detect(&mut conn).unwrap();
        assert_eq!(summary.auto_linked, 0);
    }

    #[test]
    fn does_not_link_outside_window() {
        let mut conn = db::open(":memory:").unwrap();
        insert_txn(&conn, 1, "2024-05-01", -5000);
        insert_txn(&conn, 4, "2024-05-10", 5000);

        let summary = detect(&mut conn).unwrap();
        assert_eq!(summary.auto_linked, 0);
    }

    #[test]
    fn ambiguous_multi_candidate_pairs_are_surfaced_not_guessed() {
        let mut conn = db::open(":memory:").unwrap();
        insert_txn(&conn, 1, "2024-05-01", -5000);
        insert_txn(&conn, 4, "2024-05-01", 5000);
        insert_txn(&conn, 3, "2024-05-02", 5000);

        let summary = detect(&mut conn).unwrap();
        assert_eq!(summary.auto_linked, 0);
        assert_eq!(summary.ambiguous.len(), 2);

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM transfer", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn manual_confirm_and_unlink_work_regardless_of_ambiguity() {
        let mut conn = db::open(":memory:").unwrap();
        let a = insert_txn(&conn, 1, "2024-05-01", -5000);
        let b = insert_txn(&conn, 4, "2024-05-01", 5000);
        insert_txn(&conn, 3, "2024-05-02", 5000);

        confirm(&mut conn, a, b).unwrap();
        let transfer_id: i64 = conn.query_row("SELECT id FROM transfer", [], |r| r.get(0)).unwrap();

        unlink(&mut conn, transfer_id).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM transfer", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
