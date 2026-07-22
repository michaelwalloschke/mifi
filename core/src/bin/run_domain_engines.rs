//! One-off/dev CLI: runs the post-sync domain engines (Transfer detection, then
//! categorization) against an existing database. Mirrors what a real Sync Run does at
//! the end of each run (SPEC.md §5) — useful after seeding or CSV import.
//!
//! Usage: MIFI_DB=./mifi.sqlite3 cargo run -p core --bin run_domain_engines

use mifi_core::domain::{categorize, transfer};

fn main() {
    let db_path = std::env::var("MIFI_DB").unwrap_or_else(|_| panic!("missing required env var MIFI_DB"));
    let mut conn = mifi_core::open(&db_path).expect("failed to open/migrate database");

    let transfer_summary = transfer::detect(&mut conn).expect("transfer detection failed");
    println!("{transfer_summary:#?}");

    let categorize_summary = categorize::apply(&mut conn).expect("categorization failed");
    println!("{categorize_summary:#?}");
}
