//! Chapter 36 — Estimate before rewriting, then read EXPLAIN ANALYZE.

/// Cost of a plan in arbitrary units: index probes plus row fetches.
pub struct PlanCost {
    pub pages: u64,
    pub rows: u64,
    pub selectivity: f64,
    pub random_page: f64,
    pub sequential_page: f64,
}

impl PlanCost {
    pub fn seq_scan(&self) -> f64 { self.pages as f64 * self.sequential_page }

    pub fn index_scan(&self) -> f64 {
        let matching = self.rows as f64 * self.selectivity;
        (self.pages.max(2) as f64).log2() + matching * self.random_page
    }

    /// A bad estimate is often stale statistics, not a bad query.
    pub fn estimate_error(&self, actual_rows: u64) -> f64 {
        let expected = self.rows as f64 * self.selectivity;
        if actual_rows == 0 { return f64::INFINITY; }
        (expected - actual_rows as f64).abs() / actual_rows as f64
    }
}

pub fn recommend_analyze(estimate_error: f64) -> bool { estimate_error > 5.0 }

#[cfg(test)]
mod tests {
    use super::*;

    fn cost(selectivity: f64) -> PlanCost {
        PlanCost { pages: 10_000, rows: 1_000_000, selectivity,
                   random_page: 0.4, sequential_page: 0.02 }
    }

    #[test]
    fn a_selective_predicate_prefers_the_index() {
        let c = cost(0.0001);
        assert!(c.index_scan() < c.seq_scan());
    }

    #[test]
    fn reading_half_the_table_prefers_a_scan() {
        let c = cost(0.5);
        assert!(c.seq_scan() < c.index_scan());
    }

    #[test]
    fn a_large_estimate_error_means_stale_statistics() {
        assert!(recommend_analyze(cost(0.0001).estimate_error(10)));
        assert!(!recommend_analyze(cost(0.0001).estimate_error(100)));
    }
}
