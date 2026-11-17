//! Chapter 3 — B+tree vs LSM.
//! Read amplification vs write amplification, expressed as probe counts.

/// A B+tree reaches a leaf in log2 of the number of leaf pages.
pub fn btree_probe_steps(leaf_pages: usize) -> usize {
    if leaf_pages <= 1 { return 1; }
    (usize::BITS - (leaf_pages - 1).leading_zeros()) as usize
}

/// An LSM point lookup may have to consult every level. A bloom filter
/// removes most of them; `bloom_false_positive` is the fraction that remain.
pub fn lsm_probe_steps(levels: usize, bloom_false_positive: f64) -> f64 {
    1.0 + (levels.saturating_sub(1)) as f64 * bloom_false_positive
}

/// Write amplification of a size-tiered LSM. Each level merges
/// `size_ratio` runs into the next one.
pub fn lsm_write_amplification(levels: usize, size_ratio: u32) -> f64 {
    if levels == 0 || size_ratio < 2 { return 1.0; }
    1.0 + (levels - 1) as f64 * size_ratio as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btree_depth_grows_logarithmically() {
        assert_eq!(btree_probe_steps(1), 1);
        assert_eq!(btree_probe_steps(1024), 10);
        assert_eq!(btree_probe_steps(1_048_576), 20);
    }

    #[test]
    fn bloom_filter_collapses_lsm_read_amplification() {
        assert_eq!(lsm_probe_steps(5, 1.0), 5.0);
        assert!((lsm_probe_steps(5, 0.01) - 1.04).abs() < 1e-9);
    }

    #[test]
    fn lsm_pays_for_writes_in_amplification() {
        assert_eq!(lsm_write_amplification(0, 10), 1.0);
        assert_eq!(lsm_write_amplification(4, 10), 31.0);
    }
}
