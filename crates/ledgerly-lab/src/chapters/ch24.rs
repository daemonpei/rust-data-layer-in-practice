//! Chapter 24 — Keyset pagination.
//! OFFSET degrades linearly and skips rows when data changes underneath it.

/// The seek predicate: (occurred_at, event_id) < last seen pair.
pub fn page_sql(limit: i64) -> String {
    format!(
        "SELECT event_id, occurred_at FROM usage_events \
         WHERE tenant_id = ? AND (occurred_at, event_id) < (?, ?) \
         ORDER BY occurred_at DESC, event_id DESC LIMIT {limit}")
}

/// SQLite has no row-value comparison in every build, so the repository uses
/// the expanded form. It is equivalent and portable.
pub const SEEK_SQL: &str =
    "SELECT event_id, occurred_at FROM usage_events \
     WHERE tenant_id = ? AND (occurred_at < ? OR (occurred_at = ? AND event_id < ?)) \
     ORDER BY occurred_at DESC, event_id DESC LIMIT ?";

#[derive(Debug, Clone)]
pub struct Cursor { pub at: i64, pub id_before: String }

pub async fn fetch_page(pool: &crate::SqlitePool, tenant: &str,
                        cursor: Option<Cursor>, limit: i64)
    -> anyhow::Result<Vec<(String, i64)>>
{
    // On the first page the cursor is "later than everything". Later pages
    // resume strictly after the last (timestamp, id) pair that was returned.
    let (at, id) = cursor.map(|c| (c.at, c.id_before))
        .unwrap_or((i64::MAX, "~".to_owned()));
    Ok(sqlx::query_as(SEEK_SQL)
        .bind(tenant).bind(at).bind(at).bind(id).bind(limit)
        .fetch_all(pool).await?)
}

pub fn offset_cost_ms(page: u32, page_size: u32, per_row_ms: f64) -> f64 {
    page as f64 * page_size as f64 * per_row_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seed(pool: &crate::SqlitePool, n: i64) {
        for i in 0..n {
            sqlx::query("INSERT INTO usage_events \
                         (event_id, tenant_id, metric, amount, occurred_at) \
                         VALUES (?, ?, ?, ?, ?)")
                .bind(format!("e{i:03}")).bind("t-1").bind("api_calls")
                .bind(1_i64).bind(i).execute(pool).await.unwrap();
        }
    }

    #[tokio::test]
    async fn pages_do_not_overlap() {
        let pool = crate::sqlite_memory().await.unwrap();
        seed(&pool, 10).await;
        let first = fetch_page(&pool, "t-1", None, 4).await.unwrap();
        assert_eq!(first.len(), 4);
        let last = first.last().unwrap();
        let cursor = Cursor { at: last.1, id_before: last.0.clone() };
        let second = fetch_page(&pool, "t-1", Some(cursor), 4).await.unwrap();
        assert_eq!(second.len(), 4);
        assert!(second.iter().all(|r| !first.contains(r)));
    }

    #[test]
    fn offset_gets_slower_on_every_page() {
        assert!(offset_cost_ms(100, 50, 0.01) > offset_cost_ms(1, 50, 0.01) * 50.0);
        assert!(page_sql(25).contains("LIMIT 25"));
    }
}
