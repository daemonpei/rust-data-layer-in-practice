//! Chapter 23 — Idempotency is a database constraint, not a Rust mutex.

/// The idempotency key is derived from business data, so a retry produces
/// the same key even from a different instance.
pub fn idempotency_key(tenant: &str, metric: &str, bucket: i64) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    tenant.hash(&mut h); metric.hash(&mut h); bucket.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// `INSERT ... ON CONFLICT DO NOTHING` makes the write safe to retry.
pub async fn record_event(pool: &crate::SqlitePool, key: &str, amount: i64)
    -> anyhow::Result<u64>
{
    let res = sqlx::query("INSERT INTO usage_events \
                           (event_id, tenant_id, metric, amount, occurred_at) \
                           VALUES (?, ?, ?, ?, ?) \
                           ON CONFLICT (tenant_id, event_id) DO NOTHING")
        .bind(key).bind("t-1").bind("api_calls").bind(amount).bind(1_i64)
        .execute(pool).await?;
    Ok(res.rows_affected())
}

/// The atomic accumulator: concurrent writers cannot lose an update.
pub async fn accumulate(pool: &crate::SqlitePool, key: &str, amount: i64)
    -> anyhow::Result<i64>
{
    sqlx::query("INSERT INTO usage_events \
                 (event_id, tenant_id, metric, amount, occurred_at) \
                 VALUES (?, ?, ?, ?, ?) \
                 ON CONFLICT (tenant_id, event_id) \
                 DO UPDATE SET amount = amount + excluded.amount")
        .bind(key).bind("t-1").bind("api_calls").bind(amount).bind(1_i64)
        .execute(pool).await?;
    Ok(sqlx::query_scalar("SELECT amount FROM usage_events WHERE event_id = ?")
        .bind(key).fetch_one(pool).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_input_always_produces_the_same_key() {
        assert_eq!(idempotency_key("t-1", "api_calls", 42),
                   idempotency_key("t-1", "api_calls", 42));
        assert_ne!(idempotency_key("t-1", "api_calls", 42),
                   idempotency_key("t-1", "api_calls", 43));
    }

    #[tokio::test]
    async fn a_retry_inserts_nothing_the_second_time() {
        let pool = crate::sqlite_memory().await.unwrap();
        let key = idempotency_key("t-1", "api_calls", 42);
        assert_eq!(record_event(&pool, &key, 5).await.unwrap(), 1);
        assert_eq!(record_event(&pool, &key, 5).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn two_writers_both_count() {
        let pool = crate::sqlite_memory().await.unwrap();
        let key = idempotency_key("t-1", "api_calls", 7);
        accumulate(&pool, &key, 5).await.unwrap();
        assert_eq!(accumulate(&pool, &key, 3).await.unwrap(), 8);
    }
}
