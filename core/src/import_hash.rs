//! Import Hash: per-account dedup fingerprint (SPEC.md §4, CONTEXT.md).

use std::collections::HashMap;

use sha2::{Digest, Sha256};

use crate::normalize::{normalize_merchant, normalize_purpose};

/// hash(booking date, amount, normalized counterparty, normalized purpose).
/// Occurrence index (same-day identicals) is assigned separately by `OccurrenceCounter`.
pub fn compute(booking_date: &str, amount_cents: i64, counterparty_raw: &str, purpose_raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(booking_date.as_bytes());
    hasher.update(b"\0");
    hasher.update(amount_cents.to_le_bytes());
    hasher.update(b"\0");
    hasher.update(normalize_merchant(counterparty_raw).as_bytes());
    hasher.update(b"\0");
    hasher.update(normalize_purpose(purpose_raw).as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Assigns increasing occurrence indices to same-day identical Import Hashes within one
/// account, in the order rows are encountered.
#[derive(Default)]
pub struct OccurrenceCounter {
    seen: HashMap<(i64, String), i64>,
}

impl OccurrenceCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_index(&mut self, account_id: i64, import_hash: &str) -> i64 {
        let entry = self.seen.entry((account_id, import_hash.to_string())).or_insert(-1);
        *entry += 1;
        *entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_produce_same_hash() {
        let a = compute("2022-05-30", -840, "Lotto24", "Lotto24 C98866478");
        let b = compute("2022-05-30", -840, "Lotto24", "Lotto24 C98866478");
        assert_eq!(a, b);
    }

    #[test]
    fn different_amount_produces_different_hash() {
        let a = compute("2022-05-30", -840, "Lotto24", "purpose");
        let b = compute("2022-05-30", -841, "Lotto24", "purpose");
        assert_ne!(a, b);
    }

    #[test]
    fn occurrence_counter_increments_for_repeats_within_account() {
        let mut counter = OccurrenceCounter::new();
        assert_eq!(counter.next_index(1, "h1"), 0);
        assert_eq!(counter.next_index(1, "h1"), 1);
        assert_eq!(counter.next_index(1, "h2"), 0);
        assert_eq!(counter.next_index(2, "h1"), 0);
    }
}
