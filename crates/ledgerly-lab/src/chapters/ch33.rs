//! Chapter 33 — Name the span, record the rows, sample the slow ones.

pub const SLOW_QUERY_MS: u64 = 250;

/// Span names must be low-cardinality: never put a tenant id in the name.
pub fn span_name(operation: &str) -> String {
    format!("db.{operation}")
}

pub fn is_slow(elapsed_ms: u64) -> bool { elapsed_ms >= SLOW_QUERY_MS }

/// Log 1 in N fast queries, every slow query.
pub fn should_log(elapsed_ms: u64, fast_sample_rate: u32, sequence: u64) -> bool {
    if is_slow(elapsed_ms) { return true; }
    fast_sample_rate > 0 && sequence % fast_sample_rate as u64 == 0
}

#[derive(Debug, Clone, Copy, Default)]
pub struct QueryMetrics { pub calls: u64, pub slow: u64, pub errors: u64 }

impl QueryMetrics {
    pub fn record(&mut self, elapsed_ms: u64, failed: bool) {
        self.calls += 1;
        if failed { self.errors += 1; }
        if is_slow(elapsed_ms) { self.slow += 1; }
    }
    pub fn slow_share(&self) -> f64 {
        if self.calls == 0 { return 0.0; }
        self.slow as f64 / self.calls as f64
    }
}

/// A real span, kept tiny so the chapter stays about the naming convention.
pub fn traced_count(pool: &crate::SqlitePool) -> tracing::Span {
    let span = tracing::info_span!("db.query", operation = "count", table = "tenants");
    let _ = pool;
    span
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_names_are_low_cardinality() {
        assert_eq!(span_name("select"), "db.select");
        assert!(!span_name("select").contains("acme"));
    }

    #[test]
    fn slow_queries_are_always_logged() {
        assert!(should_log(1000, 100, 7));
        assert!(should_log(10, 100, 0));
        assert!(!should_log(10, 100, 7));
    }

    #[test]
    fn metrics_expose_a_slow_share() {
        let mut m = QueryMetrics::default();
        m.record(10, false); m.record(300, false); m.record(5, true);
        assert_eq!(m.calls, 3);
        assert_eq!(m.errors, 1);
        assert!((m.slow_share() - 1.0 / 3.0).abs() < 1e-9);
    }
}
