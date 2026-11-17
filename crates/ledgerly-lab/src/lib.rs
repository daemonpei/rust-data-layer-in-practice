//! Runnable companion code for *Rust Data Layer in Practice*.
//!
//! One module per chapter: `chapters::ch01` through `chapters::ch45`.
//! The README maps every chapter to its file and to the test that exercises it.
//!
//! Most chapters run against an in-memory SQLite database, so `cargo test`
//! needs no server. PostgreSQL-only examples are marked `#[ignore]` and are
//! described in README under "PostgreSQL integration tests".

pub mod chapters;
pub mod db;

pub use db::{sqlite_memory, SqlitePool};
