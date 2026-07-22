//! mifi Rust core: the only DB writer and the only component touching secrets (SPEC.md §2, §12).

pub mod db;
pub mod import_hash;
mod migrations;
pub mod normalize;
pub mod seed;

pub use db::open;
