//! Contract normalization (SPEC.md §13 Verträge): converting any interval's expected
//! Amount to a monthly-equivalent figure for the "fixed costs/month" stat tile.

/// Average calendar weeks/month over a year, matching the biweekly/weekly clustering
/// tolerances already used by the seed's Contract grouping (SPEC.md §8).
const WEEKS_PER_MONTH: f64 = 52.0 / 12.0;

/// Normalizes `amount_cents` at the given `interval` to its monthly-equivalent value.
/// Unknown intervals return the amount unchanged (never silently zeroed).
pub fn monthly_normalized_cents(amount_cents: i64, interval: &str) -> i64 {
    let factor = match interval {
        "weekly" => WEEKS_PER_MONTH,
        "biweekly" => WEEKS_PER_MONTH / 2.0,
        "monthly" => 1.0,
        "quarterly" => 1.0 / 3.0,
        "yearly" => 1.0 / 12.0,
        _ => 1.0,
    };
    (amount_cents as f64 * factor).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monthly_amount_is_unchanged() {
        assert_eq!(monthly_normalized_cents(1499, "monthly"), 1499);
    }

    #[test]
    fn yearly_divides_by_twelve() {
        assert_eq!(monthly_normalized_cents(12000, "yearly"), 1000);
    }

    #[test]
    fn quarterly_divides_by_three() {
        assert_eq!(monthly_normalized_cents(300, "quarterly"), 100);
    }

    #[test]
    fn biweekly_scales_by_roughly_2_17() {
        assert_eq!(monthly_normalized_cents(1000, "biweekly"), 2167);
    }

    #[test]
    fn weekly_scales_by_roughly_4_33() {
        assert_eq!(monthly_normalized_cents(1000, "weekly"), 4333);
    }

    #[test]
    fn unknown_interval_falls_back_to_unchanged_amount() {
        assert_eq!(monthly_normalized_cents(500, "bogus"), 500);
    }
}
