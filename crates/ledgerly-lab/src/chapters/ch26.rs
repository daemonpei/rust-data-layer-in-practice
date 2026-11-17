//! Chapter 26 — Batch the writes, keep each statement below the parameter cap.

/// PostgreSQL has a 65,535 bind-parameter limit. Five columns means at most
/// 13,107 rows per INSERT, and the book uses a smaller batch for latency.
pub fn max_rows_per_batch(columns: u32, parameter_cap: u32) -> usize {
    if columns == 0 { return 0; }
    (parameter_cap / columns) as usize
}

/// Build a multi-row INSERT with one placeholder group per row.
pub fn insert_sql(rows: usize) -> String {
    let groups: Vec<&str> = (0..rows).map(|_| "(?, ?, ?, ?, ?)").collect();
    format!("INSERT INTO usage_events \
             (event_id, tenant_id, metric, amount, occurred_at) VALUES {}",
            groups.join(", "))
}

pub async fn bulk_insert(pool: &crate::SqlitePool, ids: &[String], batch_size: usize)
    -> anyhow::Result<u64>
{
    let mut written = 0;
    for chunk in ids.chunks(batch_size.max(1)) {
        let mut query = sqlx::query(AssertSqlSafe(insert_sql(chunk.len())));
        for (i, id) in chunk.iter().enumerate() {
            query = query.bind(id).bind("t-1").bind("api_calls")
                .bind(1_i64).bind(i as i64);
        }
        let res = query.execute(pool).await?;
        written += res.rows_affected();
    }
    Ok(written)
}

use sqlx::AssertSqlSafe;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parameter_cap_sets_the_batch_size() {
        assert_eq!(max_rows_per_batch(5, 65_535), 13_107);
        assert_eq!(max_rows_per_batch(0, 65_535), 0);
    }

    #[test]
    fn one_placeholder_group_per_row() {
        assert_eq!(insert_sql(3).matches("(?, ?, ?, ?, ?)").count(), 3);
    }

    #[tokio::test]
    async fn chunks_are_all_written() {
        let pool = crate::sqlite_memory().await.unwrap();
        let ids: Vec<String> = (0..25).map(|i| format!("e{i}")).collect();
        assert_eq!(bulk_insert(&pool, &ids, 10).await.unwrap(), 25);
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM usage_events")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(n, 25);
    }
}
