//! Chapter 14 — QueryBuilder builds a bound IN list without string glue.

use sqlx::{QueryBuilder, Row};

pub async fn find_slugs(pool: &crate::SqlitePool, slugs: &[&str])
    -> anyhow::Result<Vec<String>>
{
    if slugs.is_empty() { return Ok(Vec::new()); }
    let mut qb = QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT slug FROM tenants WHERE slug IN (");
    let mut separated = qb.separated(", ");
    for s in slugs {
        separated.push_bind(*s);
    }
    qb.push(") ORDER BY slug");
    let rows = qb.build().fetch_all(pool).await?;
    Ok(rows.iter()
        .map(|r| r.try_get::<String, _>("slug"))
        .collect::<Result<Vec<_>, _>>()?)
}

/// A second example: optional filters, the place hand-written SQL usually
/// goes wrong with trailing ANDs.
pub fn build_filtered(plan: Option<&str>, limit: i64) -> sqlx::query_builder::QueryBuilder<sqlx::Sqlite> {
    let mut qb = QueryBuilder::<sqlx::Sqlite>::new("SELECT slug FROM tenants WHERE 1 = 1");
    if let Some(p) = plan {
        qb.push(" AND plan = ").push_bind(p.to_owned());
    }
    qb.push(" LIMIT ").push_bind(limit);
    qb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_in_list_binds_each_value_separately() {
        let pool = crate::sqlite_memory().await.unwrap();
        for s in ["acme", "globex", "initech"] {
            sqlx::query("INSERT INTO tenants (tenant_id, slug) VALUES (?, ?)")
                .bind(s).bind(s).execute(&pool).await.unwrap();
        }
        let got = find_slugs(&pool, &["acme", "initech"]).await.unwrap();
        assert_eq!(got, vec!["acme".to_owned(), "initech".to_owned()]);
    }

    #[tokio::test]
    async fn an_empty_in_list_never_reaches_sql() {
        let pool = crate::sqlite_memory().await.unwrap();
        assert!(find_slugs(&pool, &[]).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn optional_filters_and_a_bound_limit() {
        let pool = crate::sqlite_memory().await.unwrap();
        sqlx::query("INSERT INTO tenants (tenant_id, slug, plan) VALUES (?, ?, ?)")
            .bind("t1").bind("acme").bind("pro").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO tenants (tenant_id, slug, plan) VALUES (?, ?, ?)")
            .bind("t2").bind("globex").bind("free").execute(&pool).await.unwrap();
        let rows = build_filtered(Some("pro"), 10).build().fetch_all(&pool).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].try_get::<String, _>("slug").unwrap(), "acme");
    }
}
