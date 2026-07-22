//! Counterparty/purpose normalization shared by Import Hash and categorization (SPEC.md §7).
//!
//! Card-network exports corrupt merchant names with mid-word spaces (`HelloFre sh`,
//! `Cleverbr idge`, `JetBrain s s.r.o.`). Dropping every non-alphanumeric character
//! before matching heals these for free, since a normalized name is only ever used
//! as an opaque grouping key, never shown to the user.

const LEGAL_SUFFIXES: &[&str] = &[
    "gmbhcokg", "gmbh", "mbh", "ag", "kgaa", "kg", "se", "ug", "ohg", "gbr", "ev", "inc", "ltd",
    "llc", "co", "corp", "srl", "sro", "sarl", "bv", "sa", "nv", "plc", "spa", "oy", "ab", "as",
];

/// Lowercases, strips all non-alphanumeric characters, and drops a trailing legal suffix.
/// Used as the merchant grouping key for merchant memory, NB tokens, and Import Hash.
pub fn normalize_merchant(raw: &str) -> String {
    let compact: String = raw
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect();

    let mut stripped = compact.as_str();
    loop {
        let mut trimmed_any = false;
        for suffix in LEGAL_SUFFIXES {
            if stripped.len() > suffix.len() && stripped.ends_with(suffix) {
                stripped = &stripped[..stripped.len() - suffix.len()];
                trimmed_any = true;
                break;
            }
        }
        if !trimmed_any {
            break;
        }
    }
    stripped.to_string()
}

/// Normalizes free-text purpose for Import Hash identity: lowercase, trimmed, whitespace
/// runs collapsed. Unlike `normalize_merchant`, this preserves word boundaries — purpose
/// text carries real multi-word meaning and only needs to be stable for hashing.
pub fn normalize_purpose(raw: &str) -> String {
    raw.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Splits normalized counterparty + purpose text into lowercase word tokens for
/// Naive Bayes token counts (SPEC.md §7).
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heals_mid_word_space_corruption() {
        assert_eq!(normalize_merchant("HelloFre sh"), "hellofresh");
        assert_eq!(normalize_merchant("Cleverbr idge"), "cleverbridge");
        assert_eq!(normalize_merchant("DisneyPl us"), "disneyplus");
    }

    #[test]
    fn strips_legal_suffix_after_healing_spaces() {
        assert_eq!(normalize_merchant("JetBrain s s.r.o."), "jetbrains");
    }

    #[test]
    fn strips_common_legal_suffixes() {
        assert_eq!(normalize_merchant("Axians Cloud + IT-Au GmbH"), "axiansclouditau");
        assert_eq!(normalize_merchant("Foo Bar AG"), "foobar");
    }

    #[test]
    fn normalize_purpose_collapses_whitespace() {
        assert_eq!(
            normalize_purpose("  Lohn  /  Gehalt   05/22 "),
            "lohn / gehalt 05/22"
        );
    }

    #[test]
    fn tokenize_splits_on_non_alphanumeric() {
        assert_eq!(
            tokenize("Lohn / Gehalt 05/22"),
            vec!["lohn", "gehalt", "05", "22"]
        );
    }
}
