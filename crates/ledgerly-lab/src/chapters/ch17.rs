//! Chapter 17 — A newtype that still binds like the underlying column.
//! `#[sqlx(transparent)]` forwards Encode/Decode/Type to the inner type.

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(raw: &str) -> anyhow::Result<Self> {
        if raw.is_empty() || raw.len() > 64 || !raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            anyhow::bail!("invalid tenant id: {raw}");
        }
        Ok(Self(raw.to_owned()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

pub async fn insert(pool: &crate::SqlitePool, id: &TenantId, slug: &str) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind(id).bind(slug).execute(pool).await?;
    Ok(())
}

pub async fn read_back(pool: &crate::SqlitePool, slug: &str) -> anyhow::Result<TenantId> {
    Ok(sqlx::query_scalar::<_, TenantId>("SELECT tenant_id FROM tenants WHERE slug = ?")
        .bind(slug).fetch_one(pool).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newtype_rejects_a_bad_identifier() {
        assert!(TenantId::new("t-1").is_ok());
        assert!(TenantId::new("t-1; DROP TABLE tenants").is_err());
        assert!(TenantId::new("").is_err());
    }

    #[tokio::test]
    async fn it_round_trips_through_the_driver() {
        let pool = crate::sqlite_memory().await.unwrap();
        let id = TenantId::new("t-42").unwrap();
        insert(&pool, &id, "acme").await.unwrap();
        assert_eq!(read_back(&pool, "acme").await.unwrap(), id);
    }
}
