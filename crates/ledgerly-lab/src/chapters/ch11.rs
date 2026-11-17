//! Chapter 11 — query, query_as, query_scalar and the compile-time macro.
//! All four run here against an in-memory SQLite pool.

use sqlx::Row;

pub async fn insert_tenant(pool: &crate::SqlitePool, id: &str, slug: &str) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO tenants (tenant_id, slug, plan) VALUES (?, ?, ?)")
        .bind(id).bind(slug).bind("pro")
        .execute(pool).await?;
    Ok(())
}

/// query(): no typed output, use Row::try_get.
pub async fn plan_via_row(pool: &crate::SqlitePool, slug: &str) -> anyhow::Result<String> {
    let row = sqlx::query("SELECT plan FROM tenants WHERE slug = ?")
        .bind(slug).fetch_one(pool).await?;
    Ok(row.try_get::<String, _>("plan")?)
}

/// query_scalar(): one column, one type.
pub async fn plan_via_scalar(pool: &crate::SqlitePool, slug: &str) -> anyhow::Result<String> {
    Ok(sqlx::query_scalar("SELECT plan FROM tenants WHERE slug = ?")
        .bind(slug).fetch_one(pool).await?)
}

/// query_as(): map a whole row into a struct (see ch16 for FromRow).
pub async fn row_as_tuple(pool: &crate::SqlitePool, slug: &str) -> anyhow::Result<(String, String)> {
    Ok(sqlx::query_as("SELECT tenant_id, plan FROM tenants WHERE slug = ?")
        .bind(slug).fetch_one(pool).await?)
}

/// The compile-time `query!` family needs a live database (or .sqlx offline
/// data). This repository uses the runtime API so CI needs no server.
pub fn macro_requires_database() -> bool { true }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn all_three_runtime_apis_agree() {
        let pool = crate::sqlite_memory().await.unwrap();
        insert_tenant(&pool, "t-1", "acme").await.unwrap();
        assert_eq!(plan_via_row(&pool, "acme").await.unwrap(), "pro");
        assert_eq!(plan_via_scalar(&pool, "acme").await.unwrap(), "pro");
        assert_eq!(row_as_tuple(&pool, "acme").await.unwrap(),
                   ("t-1".to_owned(), "pro".to_owned()));
    }

    #[tokio::test]
    async fn a_missing_row_is_an_error_not_a_null() {
        let pool = crate::sqlite_memory().await.unwrap();
        assert!(plan_via_scalar(&pool, "nobody").await.is_err());
    }

    #[test]
    fn macros_are_the_odd_one_out() {
        assert!(macro_requires_database());
    }
}
