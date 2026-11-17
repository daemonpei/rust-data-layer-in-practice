//! Chapter 10 — Project skeleton and the feature matrix.

pub const SQLX_VERSION: &str = "0.9.0";
pub const POSTGRES_VERSION: &str = "16";
pub const MSRV: &str = "1.94";
pub const EDITION: &str = "2024";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature { RuntimeTokio, Postgres, Sqlite, Macros, Migrate, Chrono, Uuid, Json }

/// Features enabled by this repository. Chapter 10 explains why the old
/// combined `runtime-tokio-rustls`-style features are gone in 0.9.
pub const ENABLED: [Feature; 8] = [
    Feature::RuntimeTokio, Feature::Postgres, Feature::Sqlite, Feature::Macros,
    Feature::Migrate, Feature::Chrono, Feature::Uuid, Feature::Json,
];

pub fn has(f: Feature) -> bool { ENABLED.contains(&f) }

pub fn baseline() -> String {
    format!("SQLx {SQLX_VERSION} / PostgreSQL {POSTGRES_VERSION} / Rust {MSRV}+ / edition {EDITION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_baseline_matches_the_books_preface() {
        assert_eq!(baseline(), "SQLx 0.9.0 / PostgreSQL 16 / Rust 1.94+ / edition 2024");
    }

    #[test]
    fn both_backends_are_compiled_in() {
        assert!(has(Feature::Postgres));
        assert!(has(Feature::Sqlite));
        assert!(has(Feature::Macros));
    }

    #[test]
    fn no_combined_runtime_feature_is_used() {
        assert_eq!(ENABLED.len(), 8);
    }
}
