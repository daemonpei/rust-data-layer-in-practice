//! Chapter 35 — SQLSTATE to action, for the errors that wake people up.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action { RetryWithBackoff, AbortAndFixSql, RollbackAndRetry, PageSomeone }

pub fn action_for(sqlstate: &str) -> Action {
    match sqlstate {
        "40001" | "40P01" => Action::RetryWithBackoff,
        "23505" | "23503" => Action::AbortAndFixSql,
        "25P02" => Action::RollbackAndRetry, // in_failed_sql_transaction
        "42P01" | "42703" => Action::AbortAndFixSql,
        "53300" => Action::PageSomeone,      // too_many_connections
        "57014" => Action::RollbackAndRetry, // query_canceled
        _ => Action::PageSomeone,
    }
}

pub fn human_name(sqlstate: &str) -> &'static str {
    match sqlstate {
        "40001" => "serialization_failure",
        "40P01" => "deadlock_detected",
        "23505" => "unique_violation",
        "25P02" => "in_failed_sql_transaction",
        "53300" => "too_many_connections",
        _ => "see Appendix B",
    }
}

/// A migration that fails midway on PostgreSQL rolls back as a whole DDL
/// transaction, but CREATE INDEX CONCURRENTLY cannot run inside one.
pub fn needs_recovery(migration_sql: &str) -> bool {
    migration_sql.contains("CONCURRENTLY")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_failures_are_retryable_and_syntax_errors_are_not() {
        assert_eq!(action_for("40001"), Action::RetryWithBackoff);
        assert_eq!(action_for("42601"), Action::PageSomeone);
        assert_eq!(action_for("42703"), Action::AbortAndFixSql);
    }

    #[test]
    fn a_failed_transaction_must_be_rolled_back_first() {
        assert_eq!(action_for("25P02"), Action::RollbackAndRetry);
        assert_eq!(human_name("25P02"), "in_failed_sql_transaction");
    }

    #[test]
    fn concurrent_indexes_need_manual_recovery() {
        assert!(needs_recovery("CREATE INDEX CONCURRENTLY i ON t (a)"));
        assert!(!needs_recovery("ALTER TABLE t ADD COLUMN a int"));
    }
}
