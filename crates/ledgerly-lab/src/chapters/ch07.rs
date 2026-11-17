//! Chapter 7 — What an application-level query engine does before SQL exists.
//! Predicate pushdown, modelled without a database.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    Eq(String, String),
    Gt(String, i64),
    In(String, Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub table: String,
    pub pushed_down: Vec<Predicate>,
    pub residual: Vec<Predicate>,
    pub limit: Option<usize>,
}

/// The engine can push a predicate into the storage layer only when the
/// backing field is indexed. Everything else stays as a residual filter.
pub fn plan_query(table: &str, predicates: Vec<Predicate>,
                  indexed: &[&str], limit: Option<usize>) -> Plan {
    let mut pushed_down = Vec::new();
    let mut residual = Vec::new();
    for p in predicates {
        let field = match &p {
            Predicate::Eq(f, _) | Predicate::Gt(f, _) | Predicate::In(f, _) => f.as_str(),
        };
        if indexed.contains(&field) { pushed_down.push(p); } else { residual.push(p); }
    }
    Plan { table: table.to_owned(), pushed_down, residual, limit }
}

impl Plan {
    /// Lower to SQL. Residual predicates are a comment: an honest marker that
    /// this engine still has to filter rows in application code.
    pub fn to_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);
        if !self.pushed_down.is_empty() {
            let parts: Vec<String> = self.pushed_down.iter().map(|p| match p {
                Predicate::Eq(f, _) => format!("{f} = ?"),
                Predicate::Gt(f, _) => format!("{f} > ?"),
                Predicate::In(f, v) => format!("{f} IN ({})", vec!["?"; v.len()].join(", ")),
            }).collect();
            sql.push_str(" WHERE ");
            sql.push_str(&parts.join(" AND "));
        }
        if let Some(n) = self.limit { sql.push_str(&format!(" LIMIT {n}")); }
        if !self.residual.is_empty() {
            sql.push_str(&format!(" -- residual filters: {}", self.residual.len()));
        }
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_predicates_are_pushed_down() {
        let plan = plan_query("usage_events",
            vec![Predicate::Eq("tenant_id".into(), "t1".into()),
                 Predicate::Gt("amount".into(), 100)],
            &["tenant_id"], Some(50));
        assert_eq!(plan.pushed_down.len(), 1);
        assert_eq!(plan.residual.len(), 1);
        assert!(plan.to_sql().starts_with("SELECT * FROM usage_events WHERE tenant_id = ?"));
        assert!(plan.to_sql().ends_with("LIMIT 50 -- residual filters: 1"));
    }

    #[test]
    fn an_in_list_binds_one_placeholder_per_value() {
        let plan = plan_query("t", vec![Predicate::In("slug".into(),
            vec!["a".into(), "b".into(), "c".into()])], &["slug"], None);
        assert!(plan.to_sql().contains("slug IN (?, ?, ?)"));
    }

    #[test]
    fn no_predicates_means_no_where_clause() {
        assert_eq!(plan_query("t", vec![], &[], None).to_sql(), "SELECT * FROM t");
    }
}
