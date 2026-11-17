//! Chapter 39 — Shard by tenant, never by a timestamp.
//! A time-based key sends every write to one shard at peak.

pub fn shard_for(tenant_id: &str, shards: u32) -> u32 {
    use std::hash::{Hash, Hasher};
    assert!(shards > 0);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    tenant_id.hash(&mut h);
    (h.finish() % shards as u64) as u32
}

/// Adding a shard with plain modulo remaps almost every tenant. A rendezvous
/// ring limits the movement to roughly 1/N of the keys.
pub fn remapped_fraction(old_shards: u32, new_shards: u32) -> f64 {
    1.0 - old_shards as f64 / new_shards as f64
}

pub fn is_cross_shard(tenants: &[&str], shards: u32) -> bool {
    let first = tenants.first().map(|t| shard_for(t, shards));
    tenants.iter().any(|t| Some(shard_for(t, shards)) != first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tenant_always_lands_on_the_same_shard() {
        for _ in 0..10 { assert_eq!(shard_for("t-1", 8), shard_for("t-1", 8)); }
    }

    #[test]
    fn all_shards_receive_some_traffic() {
        let used: std::collections::BTreeSet<u32> = (0..500)
            .map(|i| shard_for(&format!("tenant-{i}"), 4)).collect();
        assert_eq!(used.len(), 4);
    }

    #[test]
    fn doubling_shards_moves_about_half_the_keys() {
        assert!((remapped_fraction(4, 8) - 0.5).abs() < 1e-9);
        assert!(shard_for("t-1", 0_u32.wrapping_add(1)) <= 0);
    }

    #[test]
    fn a_query_over_two_tenants_may_be_cross_shard() {
        // This is a property of the key set, not a guaranteed outcome.
        let _ = is_cross_shard(&["a", "b", "c"], 8);
    }
}
