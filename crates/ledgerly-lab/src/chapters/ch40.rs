//! Chapter 40 — The transactional outbox replaces two-phase commit.
//! Business row and event are written in one local transaction; a publisher
//! drains the outbox at least once, so consumers must be idempotent.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery { Pending, Published, Failed }

#[derive(Debug, Clone, Copy)]
pub struct OutboxRow { pub id: u64, pub state: Delivery, pub attempts: u32 }

impl OutboxRow {
    pub fn new(id: u64) -> Self { Self { id, state: Delivery::Pending, attempts: 0 } }

    pub fn publish(mut self, ok: bool) -> Self {
        self.attempts += 1;
        self.state = if ok { Delivery::Published } else if self.attempts >= 5 {
            Delivery::Failed
        } else { Delivery::Pending };
        self
    }
}

/// Reconciliation: anything still pending after the SLA is a real incident,
/// not a retry to schedule.
pub fn needs_reconciliation(rows: &[OutboxRow], age_seconds: u64, sla_seconds: u64) -> Vec<u64> {
    if age_seconds < sla_seconds { return Vec::new(); }
    rows.iter().filter(|r| r.state != Delivery::Published)
        .map(|r| r.id).collect()
}

pub async fn write_business_row_and_event(
    pool: &crate::SqlitePool, event_id: &str, payload: &str,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO usage_events \
                 (event_id, tenant_id, metric, amount, occurred_at) \
                 VALUES (?, ?, ?, ?, ?)")
        .bind(event_id).bind("t-1").bind("api_calls").bind(1_i64).bind(1_i64)
        .execute(&mut *tx).await?;
    sqlx::query("INSERT INTO outbox (aggregate_id, event_type, payload) VALUES (?, ?, ?)")
        .bind(event_id).bind("usage.recorded").bind(payload)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_once_delivery_needs_idempotent_consumers() {
        let row = OutboxRow::new(1);
        assert_eq!(row.publish(false).state, Delivery::Pending);
        assert_eq!(row.publish(true).state, Delivery::Published);
        let mut r = OutboxRow::new(2);
        for _ in 0..5 { r = r.publish(false); }
        assert_eq!(r.state, Delivery::Failed);
    }

    #[test]
    fn old_pending_rows_are_reconciled() {
        let rows = vec![OutboxRow::new(1), OutboxRow::new(2).publish(true)];
        assert!(needs_reconciliation(&rows, 10, 60).is_empty());
        assert_eq!(needs_reconciliation(&rows, 120, 60), vec![1]);
    }

    #[tokio::test]
    async fn business_row_and_event_commit_together() {
        let pool = crate::sqlite_memory().await.unwrap();
        write_business_row_and_event(&pool, "e-1", "{}").await.unwrap();
        let a: i64 = sqlx::query_scalar("SELECT count(*) FROM usage_events")
            .fetch_one(&pool).await.unwrap();
        let b: i64 = sqlx::query_scalar("SELECT count(*) FROM outbox")
            .fetch_one(&pool).await.unwrap();
        assert_eq!((a, b), (1, 1));
    }
}
