//! Chapter 1 — The path a write actually takes.
//! A latency budget is a sum over layers; naming the layers tells you where to look.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    App,
    DriverPool,
    ParsePlan,
    Wal,
    DiskFsync,
}

impl Layer {
    /// Order-of-magnitude figures, in microseconds. See Preface: these are
    /// comparison values from one internal load test, not a benchmark.
    pub fn typical_us(self) -> u64 {
        match self {
            Layer::App => 40,
            Layer::DriverPool => 120,
            Layer::ParsePlan => 180,
            Layer::Wal => 260,
            Layer::DiskFsync => 400,
        }
    }
}

pub const WRITE_PATH: [Layer; 5] = [
    Layer::App, Layer::DriverPool, Layer::ParsePlan, Layer::Wal, Layer::DiskFsync,
];

pub fn budget_us(layers: &[Layer]) -> u64 {
    layers.iter().map(|l| l.typical_us()).sum()
}

/// The layer worth optimising first is the one that costs the most.
pub fn slowest(layers: &[Layer]) -> Option<Layer> {
    layers.iter().copied().max_by_key(|l| l.typical_us())
}

pub fn share_of_total(layer: Layer, layers: &[Layer]) -> f64 {
    let total = budget_us(layers);
    if total == 0 { return 0.0; }
    layer.typical_us() as f64 / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_is_the_sum_of_the_layers() {
        assert_eq!(budget_us(&WRITE_PATH), 1000);
    }

    #[test]
    fn fsync_dominates_the_budget() {
        assert_eq!(slowest(&WRITE_PATH), Some(Layer::DiskFsync));
        assert!((share_of_total(Layer::DiskFsync, &WRITE_PATH) - 0.4).abs() < 1e-9);
    }

    #[test]
    fn skipping_a_layer_actually_changes_the_number() {
        let without_fsync = &WRITE_PATH[..4];
        assert_eq!(budget_us(without_fsync), 600);
    }
}
