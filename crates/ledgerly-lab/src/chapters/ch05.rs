//! Chapter 5 — Buffer pool and plan choice.
//! A clock-sweep eviction policy and the index-vs-scan cost comparison.

#[derive(Debug, Clone)]
struct Frame { page_id: u64, used: bool }

#[derive(Debug)]
pub struct ClockBuffer {
    frames: Vec<Frame>,
    capacity: usize,
    hand: usize,
    hits: u64,
    misses: u64,
}

impl ClockBuffer {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self { frames: Vec::new(), capacity, hand: 0, hits: 0, misses: 0 }
    }

    /// Returns true on a hit. On a miss, evicts the first frame whose
    /// reference bit is clear, sweeping at most 2 * capacity times.
    pub fn access(&mut self, page_id: u64) -> bool {
        if let Some(f) = self.frames.iter_mut().find(|f| f.page_id == page_id) {
            f.used = true;
            self.hits += 1;
            return true;
        }
        self.misses += 1;
        if self.frames.len() < self.capacity {
            self.frames.push(Frame { page_id, used: true });
            return false;
        }
        for _ in 0..(2 * self.capacity) {
            let f = &mut self.frames[self.hand];
            if !f.used {
                *f = Frame { page_id, used: true };
                self.hand = (self.hand + 1) % self.capacity;
                return false;
            }
            f.used = false;
            self.hand = (self.hand + 1) % self.capacity;
        }
        self.frames[0] = Frame { page_id, used: true };
        false
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { return 0.0; }
        self.hits as f64 / total as f64
    }
}

/// Planner toy model: an index probe costs log2(pages) + rows * random_page,
/// a sequential scan costs all pages * sequential_page.
pub fn prefer_index_scan(pages: u64, matching_rows: u64,
                         random_page_ms: f64, sequential_page_ms: f64) -> bool {
    let probe = (pages.max(2) as f64).log2() + matching_rows as f64 * random_page_ms;
    let scan = pages as f64 * sequential_page_ms;
    probe < scan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_access_is_a_hit() {
        let mut buf = ClockBuffer::new(2);
        assert!(!buf.access(10));
        assert!(buf.access(10));
        assert_eq!(buf.hit_rate(), 0.5);
    }

    #[test]
    fn eviction_stays_within_capacity() {
        let mut buf = ClockBuffer::new(2);
        for p in 0..20 { buf.access(p); }
        assert_eq!(buf.frames.len(), 2);
        assert!(buf.hit_rate() < 0.2);
    }

    #[test]
    fn a_selective_predicate_wins_with_an_index() {
        assert!(prefer_index_scan(10_000, 5, 0.4, 0.02));
        assert!(!prefer_index_scan(10_000, 9_000, 0.4, 0.02));
    }
}
