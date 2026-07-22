//! One-off CLI: seeds the mifi database from the Finanzguru XLSX export (SPEC.md §11).
//!
//! Usage:
//!   FINANZGURU_XLSX=~/Downloads/finanzguru.xlsx \
//!   FINANZGURU_GIRO_REF=<Giro IBAN> \
//!   FINANZGURU_TAGESGELD_REF=<Tagesgeld IBAN> \
//!   FINANZGURU_PAYPAL_REF=<PayPal account ref> \
//!   MIFI_DB=./mifi.sqlite3 \
//!   cargo run -p core --bin seed_finanzguru
//!
//! Account refs are personal identifiers (IBANs, email) — they are never hardcoded into
//! the repo; supply them via environment variables local to your machine.

use mifi_core::seed::finanzguru::{self, AccountRefs};

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing required env var {name}"))
}

fn main() {
    let xlsx_path = env("FINANZGURU_XLSX");
    let db_path = env("MIFI_DB");
    let account_refs = AccountRefs {
        giro: env("FINANZGURU_GIRO_REF"),
        tagesgeld: env("FINANZGURU_TAGESGELD_REF"),
        paypal: env("FINANZGURU_PAYPAL_REF"),
    };

    let rows = finanzguru::parse_xlsx(&xlsx_path).expect("failed to parse Finanzguru export");
    println!("parsed {} rows from {xlsx_path}", rows.len());

    let mut conn = mifi_core::open(&db_path).expect("failed to open/migrate database");
    let summary = finanzguru::seed(&mut conn, &rows, &account_refs).expect("seed import failed");

    println!("{summary:#?}");
}
