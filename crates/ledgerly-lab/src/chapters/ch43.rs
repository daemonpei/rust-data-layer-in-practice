//! Chapter 43 — The transaction-rollback fixture.
//! Each test writes inside a transaction it never commits, so the next test
//! starts from a clean database without truncating anything.

pub async fn insert_in_transaction(pool: &crate::SqlitePool, id: &str)
    -> anyhow::Result<i64>
{
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind(id).bind(id).execute(&mut *tx).await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tenants")
        .fetch_one(&mut *tx).await?;
    tx.rollback().await?;
    Ok(count)
}

/// Tests that must see a real PostgreSQL plan (lock behaviour, EXPLAIN with
/// JSONB operators) belong in the ignored integration suite.
pub fn needs_real_postgres(reason: &str) -> bool {
    ["lock", "jsonb", "explain", "isolation"].iter().any(|k| reason.contains(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_first_test_sees_its_own_write() {
        let pool = crate::sqlite_memory().await.unwrap();
        assert_eq!(insert_in_transaction(&pool, "a").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn the_second_test_starts_clean() {
        let pool = crate::sqlite_memory().await.unwrap();
        assert_eq!(insert_in_transaction(&pool, "b").await.unwrap(), 1);
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM tenants")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn lock_and_isolation_tests_need_the_real_server() {
        assert!(needs_real_postgres("lock timeout on ALTER TABLE"));
        assert!(!needs_real_postgres("capacity formula"));
    }
}
