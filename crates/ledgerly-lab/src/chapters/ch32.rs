//! Chapter 32 — Pooling mode decides which session features you may use.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolMode { Session, Transaction, Statement }

#[derive(Debug, Clone, Copy)]
pub enum Feature { PreparedStatement, AdvisoryLock, ListenNotify, TempTable, SetSearchPath }

pub fn supported(mode: PoolMode, feature: Feature) -> bool {
    match mode {
        PoolMode::Session => true,
        PoolMode::Transaction => matches!(feature, Feature::PreparedStatement),
        PoolMode::Statement => false,
    }
}

/// A serverless function opens many short-lived connections. Without a
/// pooler in front, max_connections is the first limit you hit.
pub fn connections_needed(instances: u32, pool_per_instance: u32, behind_pgbouncer: bool) -> u32 {
    if behind_pgbouncer {
        // PgBouncer multiplexes: the server sees roughly the concurrency,
        // not the number of client connections.
        (instances * pool_per_instance / 4).max(10)
    } else {
        instances * pool_per_instance
    }
}

pub fn rejects_unsafe_combination(mode: PoolMode, feature: Feature) -> bool {
    !supported(mode, feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_mode_breaks_session_state() {
        assert!(supported(PoolMode::Session, Feature::ListenNotify));
        assert!(!supported(PoolMode::Transaction, Feature::ListenNotify));
        assert!(!supported(PoolMode::Statement, Feature::PreparedStatement));
    }

    #[test]
    fn a_pooler_cuts_server_side_connections() {
        assert_eq!(connections_needed(50, 20, false), 1000);
        assert_eq!(connections_needed(50, 20, true), 250);
    }

    #[test]
    fn unsafe_combinations_are_flagged_before_deploy() {
        assert!(rejects_unsafe_combination(PoolMode::Transaction, Feature::AdvisoryLock));
        assert!(!rejects_unsafe_combination(PoolMode::Session, Feature::TempTable));
    }
}
