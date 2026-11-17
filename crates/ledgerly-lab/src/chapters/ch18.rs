//! Chapter 18 — Value types.
//!
//! PostgreSQL has TIMESTAMPTZ / UUID / NUMERIC / JSONB. SQLite has none of
//! them, so the portable representation is: integer microseconds, a UUID
//! string, integer minor units, and JSON text. Chapter 19 shows the native
//! PostgreSQL versions.

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub id: String,
    pub tenant_id: String,
    pub metric: String,
    pub amount_minor: i64,
    pub occurred_at_us: i64,
    pub labels: serde_json::Value,
    pub note: Option<String>,
}

pub fn new_event(tenant: &str, metric: &str, amount_minor: i64, at_us: i64) -> Event {
    Event {
        id: Uuid::new_v4().to_string(),
        tenant_id: tenant.to_owned(),
        metric: metric.to_owned(),
        amount_minor,
        occurred_at_us: at_us,
        labels: serde_json::json!({"source": "api"}),
        note: None,
    }
}

pub async fn write_event(pool: &crate::SqlitePool, e: &Event) -> anyhow::Result<()> {
    let labels = serde_json::to_string(&e.labels)?;
    sqlx::query("INSERT INTO usage_events \
                 (event_id, tenant_id, metric, amount, occurred_at) \
                 VALUES (?, ?, ?, ?, ?)")
        .bind(&e.id).bind(&e.tenant_id).bind(&e.metric)
        .bind(e.amount_minor).bind(e.occurred_at_us)
        .execute(pool).await?;
    // The JSON/NULL columns live in outbox in the SQLite fixture.
    sqlx::query("INSERT INTO outbox (aggregate_id, event_type, payload, published_at) \
                 VALUES (?, ?, ?, ?)")
        .bind(&e.id).bind(&e.metric).bind(labels).bind(e.note.as_ref())
        .execute(pool).await?;
    Ok(())
}

pub async fn null_count(pool: &crate::SqlitePool) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar("SELECT count(*) FROM outbox WHERE published_at IS NULL")
        .fetch_one(pool).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn uuid_json_and_null_all_round_trip() {
        let pool = crate::sqlite_memory().await.unwrap();
        let e = new_event("t-1", "api_calls", 1250, 1_700_000_000_000_000);
        assert!(Uuid::parse_str(&e.id).is_ok());
        write_event(&pool, &e).await.unwrap();
        assert_eq!(null_count(&pool).await.unwrap(), 1);
        let payload: String = sqlx::query_scalar("SELECT payload FROM outbox WHERE aggregate_id = ?")
            .bind(&e.id).fetch_one(&pool).await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(v["source"], "api");
    }

    #[test]
    fn minor_units_avoid_float_money() {
        let e = new_event("t-1", "api_calls", 1250, 0);
        assert_eq!(e.amount_minor, 1250);
    }
}
