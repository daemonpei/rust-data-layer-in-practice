//! Chapter 2 — Sequential I/O, random I/O and fsync.
//! Why a WAL append is cheap and a scattered index update is not.

/// Sequential writes pay a per-page cost only; the arm never moves.
pub fn sequential_ms(pages: u64, per_page_ms: f64) -> f64 {
    pages as f64 * per_page_ms
}

/// Random writes pay the seek once per page.
pub fn random_ms(pages: u64, seek_ms: f64, per_page_ms: f64) -> f64 {
    pages as f64 * (seek_ms + per_page_ms)
}

/// Group commit lets N transactions share one fsync. Returns average ms
/// per transaction, including the fixed flush cost.
pub fn group_commit_ms(transactions: u64, flush_ms: f64, window_ms: f64) -> f64 {
    if transactions == 0 { return 0.0; }
    flush_ms / transactions as f64 + window_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_io_costs_more_than_sequential() {
        let seq = sequential_ms(1000, 0.02);
        let rnd = random_ms(1000, 0.4, 0.02);
        assert!(rnd > seq * 10.0);
        assert!((seq - 20.0).abs() < 1e-9);
    }

    #[test]
    fn group_commit_amortises_the_flush() {
        assert!((group_commit_ms(1, 2.0, 0.1) - 2.1).abs() < 1e-9);
        assert!((group_commit_ms(20, 2.0, 0.1) - 0.2).abs() < 1e-9);
    }

    #[test]
    fn zero_transactions_is_not_a_division_by_zero() {
        assert_eq!(group_commit_ms(0, 2.0, 0.1), 0.0);
    }
}
