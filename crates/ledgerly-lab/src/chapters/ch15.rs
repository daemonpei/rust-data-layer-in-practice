//! Chapter 15 — Classifying sqlx::Error before deciding what to do.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action { Fail, Retry, IgnoreDuplicate, Reconnect }

pub fn classify(err: &sqlx::Error) -> Action {
    match err {
        sqlx::Error::Database(db) => {
            let code = db.code().unwrap_or_default().to_string();
            match code.as_str() {
                // PostgreSQL: unique_violation. SQLite: constraint violation.
                "23505" | "2067" => Action::IgnoreDuplicate,
                // PostgreSQL: deadlock_detected / serialization_failure.
                "40P01" | "40001" => Action::Retry,
                _ => Action::Fail,
            }
        }
        sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut => Action::Reconnect,
        sqlx::Error::Io(_) | sqlx::Error::Tls(_) => Action::Retry,
        _ => Action::Fail,
    }
}

/// A unique violation is usually an idempotency hit, not an incident.
pub async fn insert_once(pool: &crate::SqlitePool, id: &str, slug: &str)
    -> anyhow::Result<Action>
{
    let res = sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
        .bind(id).bind(slug).execute(pool).await;
    match res {
        Ok(_) => Ok(Action::Fail), // nothing went wrong; Fail means "no action"
        Err(e) => Ok(classify(&e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_duplicate_tenant_is_an_idempotency_signal() {
        let pool = crate::sqlite_memory().await.unwrap();
        insert_once(&pool, "t-1", "acme").await.unwrap();
        let action = insert_once(&pool, "t-1", "acme").await.unwrap();
        assert_eq!(action, Action::IgnoreDuplicate);
    }

    #[test]
    fn retryable_postgres_codes_are_recognised() {
        assert_eq!(classify(&sqlx::Error::PoolClosed), Action::Reconnect);
        assert_eq!(classify(&sqlx::Error::RowNotFound), Action::Fail);
    }

    #[tokio::test]
    async fn a_missing_row_error_is_not_retried() {
        let pool = crate::sqlite_memory().await.unwrap();
        let e = sqlx::query_scalar::<_, String>("SELECT plan FROM tenants WHERE slug = ?")
            .bind("nobody").fetch_one(&pool).await.unwrap_err();
        assert_eq!(classify(&e), Action::Fail);
    }
}
