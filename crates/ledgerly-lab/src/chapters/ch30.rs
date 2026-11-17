//! Chapter 30 — Tenant routing.
//! A tenant name must never reach SQL as an identifier; only as a bound value.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy { SharedSchema, SchemaPerTenant, DatabasePerTenant }

pub fn valid_slug(slug: &str) -> bool {
    !slug.is_empty() && slug.len() <= 32
        && slug.chars().next().unwrap().is_ascii_lowercase()
        && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn schema_for(strategy: Strategy, slug: &str) -> anyhow::Result<String> {
    if !valid_slug(slug) { anyhow::bail!("invalid tenant slug: {slug}"); }
    Ok(match strategy {
        Strategy::SharedSchema => "public".to_owned(),
        Strategy::SchemaPerTenant => format!("tenant_{slug}"),
        Strategy::DatabasePerTenant => format!("ledgerly_{slug}"),
    })
}

/// Safe: the tenant is a value. Unsafe: it would be interpolated into DDL.
pub fn filtered_query() -> &'static str {
    "SELECT * FROM usage_events WHERE tenant_id = $1"
}

/// Migrations fan out; each target needs its own idempotent apply record.
pub fn targets(strategy: Strategy, tenants: &[&str]) -> anyhow::Result<Vec<String>> {
    tenants.iter().map(|t| schema_for(strategy, t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injection_shaped_names_are_rejected() {
        assert!(valid_slug("acme_2"));
        for bad in ["", "Acme", "acme; DROP TABLE tenants", "a-b", "x".repeat(33).as_str()] {
            assert!(!valid_slug(bad), "{bad}");
        }
    }

    #[test]
    fn each_strategy_produces_a_different_target() {
        assert_eq!(schema_for(Strategy::SharedSchema, "acme").unwrap(), "public");
        assert_eq!(schema_for(Strategy::SchemaPerTenant, "acme").unwrap(), "tenant_acme");
        assert_eq!(schema_for(Strategy::DatabasePerTenant, "acme").unwrap(), "ledgerly_acme");
    }

    #[test]
    fn the_query_never_interpolates_the_tenant() {
        assert!(filtered_query().contains("$1"));
        assert!(targets(Strategy::SchemaPerTenant, &["acme", "globex"]).unwrap().len() == 2);
    }
}
