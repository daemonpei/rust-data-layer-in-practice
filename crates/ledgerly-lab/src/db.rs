//! Shared fixtures.
//!
//! SQLite `:memory:` gives each connection its own database, so the pool is
//! pinned to one connection. That is fine for single-task tests and keeps the
//! whole suite runnable without Docker.

pub use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

/// Schema used by the chapter tests. It mirrors migrations/ closely enough to
/// exercise the same SQL idioms, minus PostgreSQL-only types such as JSONB.
pub const SQLITE_SCHEMA: &[&str] = &[
    "CREATE TABLE tenants (\
       tenant_id TEXT PRIMARY KEY, slug TEXT NOT NULL UNIQUE, \
       plan TEXT NOT NULL DEFAULT 'starter')",
    "CREATE TABLE usage_events (\
       event_id TEXT NOT NULL, tenant_id TEXT NOT NULL, metric TEXT NOT NULL, \
       amount INTEGER NOT NULL CHECK (amount >= 0), occurred_at INTEGER NOT NULL, \
       PRIMARY KEY (tenant_id, event_id))",
    "CREATE INDEX usage_events_tenant_time_idx \
       ON usage_events (tenant_id, occurred_at DESC)",
    "CREATE TABLE outbox (\
       id INTEGER PRIMARY KEY AUTOINCREMENT, aggregate_id TEXT NOT NULL, \
       event_type TEXT NOT NULL, payload TEXT NOT NULL, published_at INTEGER)",
];

/// A fresh, empty in-memory database with the Ledgerly schema applied.
pub async fn sqlite_memory() -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    for stmt in SQLITE_SCHEMA {
        // SQLx 0.9 takes `impl SqlSafeStr`; a `&'static str` qualifies, a
        // borrowed `&&str` does not. See ch12 for the dynamic-string case.
        sqlx::query(*stmt).execute(&pool).await?;
    }
    Ok(pool)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn fixture_creates_all_tables() {
        let pool = super::sqlite_memory().await.unwrap();
        let names: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(names.iter().any(|n| n == "tenants"));
        assert!(names.iter().any(|n| n == "usage_events"));
        assert!(names.iter().any(|n| n == "outbox"));
    }
}
