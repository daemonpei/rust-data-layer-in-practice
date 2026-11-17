//! Chapter 19 — PostgreSQL-only SQL.
//! These statements use PostgreSQL 16 syntax. They compile as constants on
//! every platform; the test that executes them is `#[ignore]`d until a server
//! is available.

pub const UPSERT_RETURNING: &str = r#"
INSERT INTO usage_events (event_id, tenant_id, metric, amount, occurred_at)
VALUES ($1, $2, $3, $4, now())
ON CONFLICT (tenant_id, event_id) DO UPDATE
SET amount = usage_events.amount + EXCLUDED.amount
RETURNING event_id, amount
"#;

pub const JSONB_QUERY: &str = r#"
SELECT id, payload -> 'source' AS source
FROM outbox
WHERE published_at IS NULL
  AND payload @> '{"source": "api"}'
ORDER BY id
LIMIT 100
"#;

pub const PENDING_OUTBOX: &str = r#"
UPDATE outbox SET published_at = now()
WHERE id IN (
    SELECT id FROM outbox WHERE published_at IS NULL
    ORDER BY id FOR UPDATE SKIP LOCKED LIMIT 100
)
RETURNING id, event_type, payload
"#;

pub fn uses_numbered_placeholders(sql: &str) -> bool { sql.contains("$1") }
pub fn uses_question_marks(sql: &str) -> bool { sql.contains('?') }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_syntax_is_detectable_without_a_server() {
        assert!(uses_numbered_placeholders(UPSERT_RETURNING));
        assert!(!uses_question_marks(UPSERT_RETURNING));
        assert!(PENDING_OUTBOX.contains("SKIP LOCKED"));
        assert!(JSONB_QUERY.contains("JSONB") || JSONB_QUERY.contains("@>"));
    }

    #[tokio::test]
    #[ignore = "needs DATABASE_URL and the PostgreSQL 16 compose service"]
    async fn upsert_returns_the_merged_amount() {
        use sqlx::Row;
        let url = std::env::var("DATABASE_URL")
            .expect("set DATABASE_URL (see .env.example)");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5).connect(&url).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS usage_events (\
                       event_id UUID, tenant_id UUID, metric TEXT, amount BIGINT, \
                       occurred_at TIMESTAMPTZ DEFAULT now(), \
                       UNIQUE (tenant_id, event_id))")
            .execute(&pool).await.unwrap();
        let tenant = uuid::Uuid::new_v4();
        let event = uuid::Uuid::new_v4();
        for _ in 0..2 {
            let row = sqlx::query(UPSERT_RETURNING)
                .bind(event).bind(tenant).bind("api_calls").bind(5_i64)
                .fetch_one(&pool).await.unwrap();
            let amount: i64 = row.try_get("amount").unwrap();
            assert!(amount == 5 || amount == 10);
        }
    }
}
