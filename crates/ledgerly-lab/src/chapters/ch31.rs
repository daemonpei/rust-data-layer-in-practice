//! Chapter 31 — A resumable backfill.
//! The watermark is stored with the data, so a crash resumes at the right row.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Watermark { pub last_id: u64, pub done: bool }

impl Watermark {
    pub fn start() -> Self { Self { last_id: 0, done: false } }

    /// One batch. `highest_id` is the largest id in the source table.
    pub fn advance(&mut self, batch_size: u64, highest_id: u64) -> (u64, u64) {
        if self.done { return (self.last_id, self.last_id); }
        let from = self.last_id;
        let to = (from + batch_size).min(highest_id);
        self.last_id = to;
        if to >= highest_id { self.done = true; }
        (from, to)
    }
}

/// Throttle: pause_ms grows as replication lag approaches the budget.
pub fn pause_ms(lag_bytes: u64, budget_bytes: u64, max_pause_ms: u64) -> u64 {
    if budget_bytes == 0 { return max_pause_ms; }
    let ratio = (lag_bytes as f64 / budget_bytes as f64).min(1.0);
    (ratio * max_pause_ms as f64) as u64
}

/// The batch UPDATE from Chapter 28, bound to a keyset range.
pub const BACKFILL_SQL: &str =
    "UPDATE usage_events SET billing_period = to_char(occurred_at, 'YYYYMM') \
     WHERE billing_period IS NULL AND event_id > $1 \
     AND event_id IN (SELECT event_id FROM usage_events \
                      WHERE event_id > $1 ORDER BY event_id LIMIT $2)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batches_partition_the_whole_range_without_gaps() {
        let mut w = Watermark::start();
        let mut ranges = Vec::new();
        for _ in 0..12 { ranges.push(w.advance(10_000, 100_000)); }
        assert_eq!(ranges[0], (0, 10_000));
        assert_eq!(ranges[9], (90_000, 100_000));
        assert!(w.done);
        assert_eq!(w.advance(10_000, 100_000), (100_000, 100_000));
    }

    #[test]
    fn throttling_tracks_replication_lag() {
        assert_eq!(pause_ms(0, 1000, 500), 0);
        assert_eq!(pause_ms(500, 1000, 500), 250);
        assert_eq!(pause_ms(5000, 1000, 500), 500);
    }

    #[test]
    fn the_update_is_bounded_by_a_batch() {
        assert!(BACKFILL_SQL.contains("LIMIT $2"));
    }
}
