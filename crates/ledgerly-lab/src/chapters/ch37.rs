//! Chapter 37 — Bound values, tenant isolation, and masking.

/// Identifiers cannot be bound, so they must be whitelisted. This is the
/// one place string interpolation is acceptable.
pub fn safe_identifier(raw: &str) -> anyhow::Result<&str> {
    const ALLOWED: [&str; 5] = ["tenant_id", "slug", "metric", "amount", "occurred_at"];
    ALLOWED.iter().copied().find(|a| *a == raw)
        .ok_or_else(|| anyhow::anyhow!("identifier {raw} is not whitelisted"))
}

/// Every tenant-scoped query must carry the tenant predicate.
pub fn is_tenant_scoped(sql: &str) -> bool {
    let lower = sql.to_ascii_lowercase();
    lower.contains("tenant_id") && (lower.contains("= $1") || lower.contains("= ?"))
}

pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((user, domain)) => {
            let shown: String = user.chars().take(1).collect();
            format!("{shown}***@{domain}")
        }
        None => "***".to_owned(),
    }
}

/// A row-level check that runs even if a query forgot its WHERE clause.
pub fn row_belongs_to(row_tenant: &str, request_tenant: &str) -> bool {
    row_tenant == request_tenant
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_whitelisted_identifiers_reach_ddl() {
        assert_eq!(safe_identifier("amount").unwrap(), "amount");
        assert!(safe_identifier("amount); DROP TABLE tenants; --").is_err());
    }

    #[test]
    fn a_query_without_the_tenant_predicate_is_rejected() {
        assert!(is_tenant_scoped("SELECT * FROM usage_events WHERE tenant_id = $1"));
        assert!(!is_tenant_scoped("SELECT * FROM usage_events"));
    }

    #[test]
    fn logs_do_not_leak_the_full_address() {
        assert_eq!(mask_email("alice@example.com"), "a***@example.com");
        assert_eq!(mask_email("not-an-email"), "***");
        assert!(row_belongs_to("t-1", "t-1"));
        assert!(!row_belongs_to("t-2", "t-1"));
    }
}
