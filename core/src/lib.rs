//! mifi Rust core: the only DB writer and the only component touching secrets (SPEC.md §2, §12).

pub mod db;
mod migrations;

pub use db::open;
