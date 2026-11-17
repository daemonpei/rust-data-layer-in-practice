//! Chapter 8 — The same read, four ways.
//! This is the comparison from the book, kept as strings so it compiles
//! without any ORM dependency.

pub const SCHEMA: &str = "tenants(tenant_id, slug, plan)";

/// SQLx: the statement a DBA hands you works as written.
pub fn sqlx_listing() -> &'static str {
    "SELECT tenant_id, slug, plan FROM tenants WHERE slug = $1"
}

/// Diesel: the DSL compiles the query into SQL.
pub fn diesel_listing() -> &'static str {
    "tenants::table.filter(tenants::slug.eq(slug)).select(Tenant::as_select())"
}

/// SeaORM: entity find with a column comparison.
pub fn seaorm_listing() -> &'static str {
    "Tenants::find().filter(tenants::Column::Slug.eq(slug)).one(db)"
}

/// Toasty: an application-level query expression.
pub fn toasty_listing() -> &'static str {
    "Tenant::filter_by_slug(slug).query(&db).await?"
}

/// Rough token count: a stand-in for how much machinery sits between the
/// reader and the SQL that actually runs.
pub fn abstraction_tokens(listing: &str) -> usize {
    listing.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_expressions_read_the_same_table() {
        for q in [sqlx_listing(), diesel_listing(), seaorm_listing(), toasty_listing()] {
            assert!(q.to_lowercase().contains("slug"));
        }
    }

    #[test]
    fn sqlx_uses_a_numbered_placeholder() {
        assert!(sqlx_listing().contains("$1"));
    }

    #[test]
    fn the_four_listings_are_not_the_same_string() {
        let all = [sqlx_listing(), diesel_listing(), seaorm_listing(), toasty_listing()];
        for i in 0..4 {
            for j in (i + 1)..4 { assert_ne!(all[i], all[j]); }
        }
        assert!(all.iter().all(|q| abstraction_tokens(q) > 0));
    }
}
