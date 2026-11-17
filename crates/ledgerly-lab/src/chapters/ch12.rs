//! Chapter 12 — Compile-time verification and the 0.9 `SqlSafeStr` boundary.
//!
//! In 0.9 `sqlx::query()` takes `impl SqlSafeStr`. A `&'static str` is trusted;
//! a `String` built at run time is not. `AssertSqlSafe` is the explicit opt-out,
//! and its name is the code review hook.

use sqlx::AssertSqlSafe;

/// Trusted: the literal is part of the binary and can be checked by `query!`.
pub async fn static_sql(pool: &crate::SqlitePool) -> anyhow::Result<i64> {
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM tenants")
        .fetch_one(pool).await?;
    Ok(n)
}

/// Untrusted: the caller chose the column. We assert it against a whitelist,
/// then mark the assembled string as reviewed.
pub async fn dynamic_column(pool: &crate::SqlitePool, column: &str) -> anyhow::Result<i64> {
    const ALLOWED: [&str; 3] = ["tenant_id", "slug", "plan"];
    if !ALLOWED.contains(&column) {
        anyhow::bail!("column {column} is not in the whitelist");
    }
    let sql = format!("SELECT count({column}) FROM tenants");
    let n: i64 = sqlx::query_scalar(AssertSqlSafe(sql))
        .fetch_one(pool).await?;
    Ok(n)
}

/// A cheap CI rule: anything that needs `AssertSqlSafe` must be reviewed.
pub fn needs_review(sql_source: &str) -> bool {
    !sql_source.starts_with("\"") || sql_source.contains("format!")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[tokio::test]
    async fn static_and_dynamic_both_run() {
        let pool = crate::sqlite_memory().await.unwrap();
        sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
            .bind("t-1").bind("acme").execute(&pool).await.unwrap();
        assert_eq!(static_sql(&pool).await.unwrap(), 1);
        assert_eq!(dynamic_column(&pool, "slug").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn an_unlisted_column_is_rejected_before_sql_is_built() {
        let pool = crate::sqlite_memory().await.unwrap();
        let err = dynamic_column(&pool, "plan; DROP TABLE tenants").await.unwrap_err();
        assert!(err.to_string().contains("whitelist"));
    }

    #[test]
    fn the_review_flag_fires_on_generated_sql() {
        assert!(needs_review("format!(\"SELECT {x}\")"));
        assert!(!needs_review("\"SELECT 1\""));
    }

    #[tokio::test]
    async fn raw_sql_also_accepts_the_assertion_wrapper() {
        let pool = crate::sqlite_memory().await.unwrap();
        let row = sqlx::query(AssertSqlSafe(String::from("SELECT 7 AS n")))
            .fetch_one(&pool).await.unwrap();
        assert_eq!(row.try_get::<i64, _>("n").unwrap(), 7);
    }
}
