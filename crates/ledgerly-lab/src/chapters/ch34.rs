//! Chapter 34 — Symptom-to-check decision tree for connection problems.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symptom { PoolTimeout, TooManyConnections, LatencySpikeAtOpen, IdleInTransaction }

pub struct Check { pub question: &'static str, pub sql: &'static str }

pub fn first_checks(s: Symptom) -> &'static [Check] {
    match s {
        Symptom::PoolTimeout => &[
            Check { question: "Is the pool actually exhausted, or is the query slow?",
                    sql: "SELECT count(*) FROM pg_stat_activity WHERE state = 'active'" },
            Check { question: "How long are connections held?",
                    sql: "SELECT max(now() - query_start) FROM pg_stat_activity" },
        ],
        Symptom::TooManyConnections => &[
            Check { question: "Did a new deployment multiply the instance count?",
                    sql: "SELECT count(*) FROM pg_stat_activity GROUP BY application_name" },
        ],
        Symptom::LatencySpikeAtOpen => &[
            Check { question: "Is the pool cold-starting all connections at once?",
                    sql: "SELECT count(*) FROM pg_stat_activity WHERE backend_start > now() - interval '1 min'" },
        ],
        Symptom::IdleInTransaction => &[
            Check { question: "Which query left a transaction open?",
                    sql: "SELECT pid, state, query FROM pg_stat_activity \
                          WHERE state = 'idle in transaction'" },
        ],
    }
}

/// Pool timeouts are a capacity symptom. Restarting the app clears the queue
/// for a minute and then the same thing happens.
pub fn root_cause(s: Symptom) -> &'static str {
    match s {
        Symptom::PoolTimeout => "hold time times concurrency exceeds pool size",
        Symptom::TooManyConnections => "instances times pool size exceeds max_connections",
        Symptom::LatencySpikeAtOpen => "cold pool warm-up overlaps with peak traffic",
        Symptom::IdleInTransaction => "a transaction is held across an await",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_symptom_has_at_least_one_check() {
        for s in [Symptom::PoolTimeout, Symptom::TooManyConnections,
                  Symptom::LatencySpikeAtOpen, Symptom::IdleInTransaction] {
            assert!(!first_checks(s).is_empty());
            assert!(first_checks(s)[0].sql.to_lowercase().contains("pg_stat_activity"));
            assert!(!root_cause(s).is_empty());
        }
    }

    #[test]
    fn pool_timeout_is_not_fixed_by_a_restart() {
        assert_eq!(root_cause(Symptom::PoolTimeout),
                   "hold time times concurrency exceeds pool size");
    }
}
