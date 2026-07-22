//! Categorization layers 1–2: merchant memory + Naive Bayes (SPEC.md §7).
//!
//! Layer 3 (local LLM sweep) is a separate, later concern — this module only ever
//! assigns `category_source = 'auto'` and never touches a `user`-set Split. Transfer
//! legs are permanently excluded (they stay uncategorized by definition).

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};

use crate::normalize::tokenize;

/// top1/top2 posterior ratio must be at least this to accept an NB assignment
/// (SPEC.md §7) — tunable.
const NB_CONFIDENCE_RATIO: f64 = 3.0;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct CategorizeSummary {
    pub merchant_memory_assigned: usize,
    pub naive_bayes_assigned: usize,
    pub left_uncategorized: usize,
}

struct UncategorizedSplit {
    split_id: i64,
    counterparty_normalized: String,
    counterparty_raw: String,
    purpose_raw: String,
}

fn uncategorized_splits(conn: &Connection) -> rusqlite::Result<Vec<UncategorizedSplit>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, t.counterparty_normalized, t.counterparty_raw, t.purpose_raw
         FROM split s
         JOIN \"transaction\" t ON t.id = s.transaction_id
         WHERE s.category_id IS NULL
           AND s.category_source = 'auto'
           AND t.id NOT IN (SELECT leg_a_txn_id FROM transfer UNION SELECT leg_b_txn_id FROM transfer)",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(UncategorizedSplit {
            split_id: row.get(0)?,
            counterparty_normalized: row.get(1)?,
            counterparty_raw: row.get(2)?,
            purpose_raw: row.get(3)?,
        })
    })?;
    rows.collect()
}

fn merchant_rule_category(conn: &Connection, normalized_merchant: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT category_id FROM merchant_rule WHERE normalized_merchant = ?1",
        [normalized_merchant],
        |row| row.get(0),
    )
    .optional()
}

struct NaiveBayesModel {
    /// (token, category_id) -> count
    token_category_counts: HashMap<(String, i64), i64>,
    /// category_id -> total tokens seen
    category_token_totals: HashMap<i64, i64>,
    /// category_id -> number of already-categorized Splits (prior)
    category_priors: HashMap<i64, i64>,
    total_categorized: i64,
    vocab_size: i64,
}

impl NaiveBayesModel {
    fn load(conn: &Connection) -> rusqlite::Result<Self> {
        let mut token_category_counts = HashMap::new();
        let mut category_token_totals: HashMap<i64, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare("SELECT token, category_id, count FROM nb_token_count")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
            })?;
            for row in rows {
                let (token, category_id, count) = row?;
                *category_token_totals.entry(category_id).or_insert(0) += count;
                token_category_counts.insert((token, category_id), count);
            }
        }

        let mut category_priors = HashMap::new();
        let mut total_categorized = 0i64;
        {
            let mut stmt =
                conn.prepare("SELECT category_id, COUNT(*) FROM split WHERE category_id IS NOT NULL GROUP BY category_id")?;
            let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
            for row in rows {
                let (category_id, count) = row?;
                category_priors.insert(category_id, count);
                total_categorized += count;
            }
        }

        let vocab_size: i64 = conn.query_row("SELECT COUNT(DISTINCT token) FROM nb_token_count", [], |r| r.get(0))?;

        Ok(Self { token_category_counts, category_token_totals, category_priors, total_categorized, vocab_size })
    }

    /// Returns the top-scoring category if its log-posterior beats the runner-up by at
    /// least `ln(NB_CONFIDENCE_RATIO)`, else `None` (honest uncategorized queue).
    fn classify(&self, tokens: &[String]) -> Option<i64> {
        if self.total_categorized == 0 || self.vocab_size == 0 {
            return None;
        }
        let mut scores: Vec<(i64, f64)> = self
            .category_priors
            .keys()
            .map(|&category_id| {
                let prior = self.category_priors[&category_id] as f64 / self.total_categorized as f64;
                let cat_total = *self.category_token_totals.get(&category_id).unwrap_or(&0) as f64;
                let mut log_score = prior.ln();
                for token in tokens {
                    let count = *self.token_category_counts.get(&(token.clone(), category_id)).unwrap_or(&0) as f64;
                    let p_token = (count + 1.0) / (cat_total + self.vocab_size as f64);
                    log_score += p_token.ln();
                }
                (category_id, log_score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        match scores.as_slice() {
            [] => None,
            [(only, _)] => Some(*only),
            [(top1, score1), (_, score2), ..] => {
                if score1 - score2 >= NB_CONFIDENCE_RATIO.ln() {
                    Some(*top1)
                } else {
                    None
                }
            }
        }
    }
}

/// Applies merchant memory then Naive Bayes to every uncategorized, non-Transfer Split.
/// Never overwrites a `user`-set Split; below-confidence rows are left in the honest
/// uncategorized queue (SPEC.md §7).
pub fn apply(conn: &mut Connection) -> rusqlite::Result<CategorizeSummary> {
    let splits = uncategorized_splits(conn)?;
    let model = NaiveBayesModel::load(conn)?;
    let mut summary = CategorizeSummary::default();

    let tx = conn.transaction()?;
    for split in splits {
        if let Some(category_id) = merchant_rule_category(&tx, &split.counterparty_normalized)? {
            tx.execute(
                "UPDATE split SET category_id = ?1, category_source = 'auto' WHERE id = ?2",
                (category_id, split.split_id),
            )?;
            summary.merchant_memory_assigned += 1;
            continue;
        }

        let tokens: Vec<String> =
            tokenize(&split.counterparty_raw).into_iter().chain(tokenize(&split.purpose_raw)).collect();
        match model.classify(&tokens) {
            Some(category_id) => {
                tx.execute(
                    "UPDATE split SET category_id = ?1, category_source = 'auto' WHERE id = ?2",
                    (category_id, split.split_id),
                )?;
                summary.naive_bayes_assigned += 1;
            }
            None => summary.left_uncategorized += 1,
        }
    }
    tx.commit()?;

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_transaction(conn: &Connection, counterparty: &str, purpose: &str) -> i64 {
        let nonce: i64 = conn.query_row("SELECT COUNT(*) FROM \"transaction\"", [], |r| r.get(0)).unwrap();
        conn.execute(
            "INSERT INTO \"transaction\" (
                account_id, booking_date, amount_cents, counterparty_raw, counterparty_normalized,
                purpose_raw, purpose_normalized, import_hash, occurrence_index, source
            ) VALUES (1, '2024-01-01', -1000, ?1, ?2, ?3, ?3, ?4, 0, 'csv-paypal')",
            (counterparty, crate::normalize::normalize_merchant(counterparty), purpose, format!("h-{counterparty}-{purpose}-{nonce}")),
        )
        .unwrap();
        let txn_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO split (transaction_id, amount_cents, category_id, category_source) VALUES (?1, -1000, NULL, 'auto')",
            [txn_id],
        )
        .unwrap();
        txn_id
    }

    fn insert_category(conn: &Connection, id: i64, name: &str) {
        conn.execute(
            "INSERT INTO category (id, parent_id, name, kind) VALUES (?1, NULL, ?2, 'expense')",
            (id, name),
        )
        .unwrap();
    }

    #[test]
    fn merchant_memory_takes_priority_over_nb() {
        let mut conn = db::open(":memory:").unwrap();
        insert_category(&conn, 1, "Essen");
        conn.execute(
            "INSERT INTO merchant_rule (normalized_merchant, category_id) VALUES ('hellofresh', 1)",
            [],
        )
        .unwrap();
        setup_transaction(&conn, "HelloFresh", "lieferung");

        let summary = apply(&mut conn).unwrap();
        assert_eq!(summary.merchant_memory_assigned, 1);
        assert_eq!(summary.naive_bayes_assigned, 0);

        let category_id: i64 =
            conn.query_row("SELECT category_id FROM split", [], |r| r.get(0)).unwrap();
        assert_eq!(category_id, 1);
    }

    #[test]
    fn naive_bayes_assigns_when_confidently_skewed() {
        let mut conn = db::open(":memory:").unwrap();
        insert_category(&conn, 1, "Essen");
        insert_category(&conn, 2, "Freizeit");
        for _ in 0..20 {
            let txn = setup_transaction(&conn, "Rewe", "einkauf");
            conn.execute("UPDATE split SET category_id = 1 WHERE transaction_id = ?1", [txn]).unwrap();
            for token in ["rewe", "einkauf"] {
                conn.execute(
                    "INSERT INTO nb_token_count (token, category_id, count) VALUES (?1, 1, 1)
                     ON CONFLICT (token, category_id) DO UPDATE SET count = count + 1",
                    (token,),
                )
                .unwrap();
            }
        }
        for _ in 0..20 {
            let txn = setup_transaction(&conn, "Kino", "film");
            conn.execute("UPDATE split SET category_id = 2 WHERE transaction_id = ?1", [txn]).unwrap();
            for token in ["kino", "film"] {
                conn.execute(
                    "INSERT INTO nb_token_count (token, category_id, count) VALUES (?1, 2, 1)
                     ON CONFLICT (token, category_id) DO UPDATE SET count = count + 1",
                    (token,),
                )
                .unwrap();
            }
        }

        setup_transaction(&conn, "Rewe", "einkauf");
        let summary = apply(&mut conn).unwrap();
        assert_eq!(summary.naive_bayes_assigned, 1);

        let category_id: Option<i64> = conn
            .query_row("SELECT category_id FROM split WHERE category_id IS NOT NULL ORDER BY id DESC LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(category_id, Some(1));
    }

    #[test]
    fn below_confidence_stays_uncategorized() {
        let mut conn = db::open(":memory:").unwrap();
        insert_category(&conn, 1, "Essen");
        setup_transaction(&conn, "Unknown Merchant", "mystery purchase");

        let summary = apply(&mut conn).unwrap();
        assert_eq!(summary.left_uncategorized, 1);

        let category_id: Option<i64> = conn.query_row("SELECT category_id FROM split", [], |r| r.get(0)).unwrap();
        assert_eq!(category_id, None);
    }

    #[test]
    fn never_touches_user_set_splits() {
        let mut conn = db::open(":memory:").unwrap();
        insert_category(&conn, 1, "Essen");
        let txn = setup_transaction(&conn, "Rewe", "einkauf");
        conn.execute(
            "UPDATE split SET category_id = 1, category_source = 'user' WHERE transaction_id = ?1",
            [txn],
        )
        .unwrap();

        let summary = apply(&mut conn).unwrap();
        assert_eq!(summary.merchant_memory_assigned, 0);
        assert_eq!(summary.naive_bayes_assigned, 0);
        assert_eq!(summary.left_uncategorized, 0);

        let (category_id, source): (i64, String) =
            conn.query_row("SELECT category_id, category_source FROM split", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!((category_id, source.as_str()), (1, "user"));
    }

    #[test]
    fn transfer_legs_are_never_categorized() {
        let mut conn = db::open(":memory:").unwrap();
        insert_category(&conn, 1, "Essen");
        conn.execute(
            "INSERT INTO merchant_rule (normalized_merchant, category_id) VALUES ('ownaccount', 1)",
            [],
        )
        .unwrap();
        let a = setup_transaction(&conn, "OwnAccount", "sparplan");
        let b = setup_transaction(&conn, "OwnAccount", "sparplan");
        conn.execute(
            "INSERT INTO transfer (leg_a_txn_id, leg_b_txn_id, link_source) VALUES (?1, ?2, 'auto')",
            (a, b),
        )
        .unwrap();

        let summary = apply(&mut conn).unwrap();
        assert_eq!(summary.merchant_memory_assigned, 0);
        assert_eq!(summary.left_uncategorized, 0);
    }
}
