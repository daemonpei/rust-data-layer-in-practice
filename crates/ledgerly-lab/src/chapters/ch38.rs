//! Chapter 38 — Read/write splitting with an explicit consistency window.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target { Primary, Replica }

pub fn route(query: &str, force_primary: bool) -> Target {
    let lower = query.trim_start().to_ascii_lowercase();
    let is_write = lower.starts_with("insert") || lower.starts_with("update")
        || lower.starts_with("delete") || lower.contains(" for update")
        || lower.starts_with("create") || lower.starts_with("alter");
    if force_primary || is_write { Target::Primary } else { Target::Replica }
}

/// After a write, reads that must see it stay on the primary for this long.
pub fn within_consistency_window(elapsed_ms: u64, replica_lag_ms: u64) -> bool {
    elapsed_ms < replica_lag_ms
}

pub fn route_after_write(elapsed_ms: u64, replica_lag_ms: u64) -> Target {
    if within_consistency_window(elapsed_ms, replica_lag_ms) {
        Target::Primary
    } else {
        Target::Replica
    }
}

/// Replica lag is not a boolean; a stale read is a correctness bug.
pub fn stale_risk_pct(lag_ms: u64, window_ms: u64) -> f64 {
    if window_ms == 0 { return 100.0; }
    (lag_ms as f64 / window_ms as f64 * 100.0).min(100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_locking_reads_go_to_the_primary() {
        assert_eq!(route("INSERT INTO t VALUES (1)", false), Target::Primary);
        assert_eq!(route("SELECT * FROM t FOR UPDATE", false), Target::Primary);
        assert_eq!(route("SELECT * FROM t", false), Target::Replica);
    }

    #[test]
    fn a_read_right_after_a_write_stays_on_the_primary() {
        assert_eq!(route_after_write(10, 200), Target::Primary);
        assert_eq!(route_after_write(300, 200), Target::Replica);
    }

    #[test]
    fn lag_is_quantified() {
        assert_eq!(stale_risk_pct(50, 200), 25.0);
        assert_eq!(stale_risk_pct(500, 200), 100.0);
    }
}
