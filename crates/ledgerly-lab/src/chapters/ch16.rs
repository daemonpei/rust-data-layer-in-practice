//! Chapter 16 — FromRow maps columns into a struct.

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct TenantRow {
    pub tenant_id: String,
    pub slug: String,
    #[sqlx(default)]
    pub plan: Option<String>,
}

pub async fn load(pool: &crate::SqlitePool, slug: &str) -> anyhow::Result<TenantRow> {
    Ok(sqlx::query_as::<_, TenantRow>(
        "SELECT tenant_id, slug, plan FROM tenants WHERE slug = ?")
        .bind(slug).fetch_one(pool).await?)
}

/// `#[sqlx(default)]` lets a missing column become None instead of an error.
pub async fn load_partial(pool: &crate::SqlitePool, slug: &str) -> anyhow::Result<TenantRow> {
    Ok(sqlx::query_as::<_, TenantRow>("SELECT tenant_id, slug FROM tenants WHERE slug = ?")
        .bind(slug).fetch_one(pool).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_full_row_decodes_into_the_struct() {
        let pool = crate::sqlite_memory().await.unwrap();
        sqlx::query("INSERT INTO tenants (tenant_id, slug, plan) VALUES (?, ?, ?)")
            .bind("t-1").bind("acme").bind("pro").execute(&pool).await.unwrap();
        assert_eq!(load(&pool, "acme").await.unwrap(), TenantRow {
            tenant_id: "t-1".into(), slug: "acme".into(), plan: Some("pro".into()),
        });
    }

    #[tokio::test]
    async fn a_missing_column_falls_back_to_the_default() {
        let pool = crate::sqlite_memory().await.unwrap();
        sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
            .bind("t-2").bind("globex").execute(&pool).await.unwrap();
        assert_eq!(load_partial(&pool, "globex").await.unwrap().plan, None);
    }
}
