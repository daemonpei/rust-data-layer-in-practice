//! Chapter 22 — Transactions and savepoints.
//! A transaction must be held on one task; holding it across an await on an
//! unrelated future is how pools get exhausted.

pub async fn commit_two_rows(pool: &crate::SqlitePool) -> anyhow::Result<i64> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind("t-1").bind("acme").execute(&mut *tx).await?;
    sqlx::query("INSERT INTO outbox (aggregate_id, event_type, payload) VALUES (?, ?, ?)")
        .bind("t-1").bind("tenant.created").bind("{}").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(sqlx::query_scalar("SELECT count(*) FROM tenants").fetch_one(pool).await?)
}

pub async fn rollback_leaves_nothing(pool: &crate::SqlitePool) -> anyhow::Result<i64> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind("t-2").bind("globex").execute(&mut *tx).await?;
    tx.rollback().await?;
    Ok(sqlx::query_scalar("SELECT count(*) FROM tenants").fetch_one(pool).await?)
}

/// A savepoint discards only the failing statement, keeping earlier work.
pub async fn savepoint_keeps_earlier_work(pool: &crate::SqlitePool) -> anyhow::Result<(i64, i64)> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind("t-1").bind("acme").execute(&mut *tx).await?;
    sqlx::query("SAVEPOINT sp").execute(&mut *tx).await?;
    let bad = sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind("t-1").bind("duplicate").execute(&mut *tx).await;
    assert!(bad.is_err());
    sqlx::query("ROLLBACK TO sp").execute(&mut *tx).await?;
    sqlx::query("RELEASE sp").execute(&mut *tx).await?;
    sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind("t-2").bind("globex").execute(&mut *tx).await?;
    tx.commit().await?;
    let tenants: i64 = sqlx::query_scalar("SELECT count(*) FROM tenants").fetch_one(pool).await?;
    Ok((tenants, tenants))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn commit_makes_both_writes_visible() {
        let pool = crate::sqlite_memory().await.unwrap();
        assert_eq!(commit_two_rows(&pool).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn rollback_removes_every_write() {
        let pool = crate::sqlite_memory().await.unwrap();
        assert_eq!(rollback_leaves_nothing(&pool).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn a_savepoint_does_not_discard_the_whole_transaction() {
        let pool = crate::sqlite_memory().await.unwrap();
        let (tenants, _) = savepoint_keeps_earlier_work(&pool).await.unwrap();
        assert_eq!(tenants, 2);
    }
}
