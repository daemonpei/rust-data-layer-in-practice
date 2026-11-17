//! Chapter 41 — Cache-aside with jitter, and an explicit invalidation order.

use std::collections::HashMap;

pub fn ttl_with_jitter(base_seconds: u64, tenant_hash: u64) -> u64 {
    // Up to +/-20%, derived from the key so retries agree on the same TTL.
    let pct = (tenant_hash % 41) as f64 / 100.0 - 0.20;
    ((base_seconds as f64) * (1.0 + pct)).round().max(1.0) as u64
}

#[derive(Debug, Default)]
pub struct Cache { values: HashMap<String, (String, u64)> }

impl Cache {
    pub fn get(&self, key: &str, now: u64) -> Option<&str> {
        self.values.get(key)
            .filter(|(_, expires)| *expires > now)
            .map(|(v, _)| v.as_str())
    }
    pub fn put(&mut self, key: &str, value: &str, now: u64, ttl: u64) {
        self.values.insert(key.to_owned(), (value.to_owned(), now + ttl));
    }
    pub fn invalidate(&mut self, key: &str) { self.values.remove(key); }
}

/// Delete the cache entry *after* the database commit. Deleting it before
/// lets a concurrent reader repopulate the stale value.
pub fn invalidation_order() -> &'static [&'static str] {
    &["commit", "invalidate", "optional delayed double-delete"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitter_never_produces_a_zero_ttl() {
        for h in 0..1000 { assert!(ttl_with_jitter(30, h) >= 11); }
        assert_eq!(ttl_with_jitter(30, 20), 30);
    }

    #[test]
    fn entries_expire_and_can_be_invalidated() {
        let mut c = Cache::default();
        c.put("tenant:t-1", "pro", 0, 30);
        assert_eq!(c.get("tenant:t-1", 10), Some("pro"));
        assert_eq!(c.get("tenant:t-1", 40), None);
        c.put("tenant:t-1", "pro", 0, 30);
        c.invalidate("tenant:t-1");
        assert_eq!(c.get("tenant:t-1", 1), None);
    }

    #[test]
    fn invalidation_happens_after_the_commit() {
        assert_eq!(invalidation_order()[0], "commit");
    }
}
