//! Chapter 21 — Pool capacity is calculated, not guessed.
//! The five formulas from the Appendix C formula card.

#[derive(Debug, Clone, Copy)]
pub struct Capacity {
    pub app_instances: u32,
    pub max_connections: u32,
    pub reserved_for_admin: u32,
    pub peak_concurrency_per_instance: u32,
}

/// Formula 1: what each instance may take without starving admin sessions.
pub fn pool_max_per_instance(c: Capacity) -> u32 {
    let usable = c.max_connections.saturating_sub(c.reserved_for_admin);
    if c.app_instances == 0 { return 0; }
    usable / c.app_instances
}

/// Formula 2: total connections actually held at peak.
pub fn peak_connections(c: Capacity, per_instance: u32) -> u32 {
    c.app_instances * per_instance
}

/// Formula 3: time a request waits for a connection, in ms, when all are busy.
pub fn wait_ms(peak_concurrency: u32, pool_size: u32, hold_ms: u32) -> u32 {
    if pool_size == 0 { return u32::MAX; }
    let queued = peak_concurrency.saturating_sub(pool_size) as f64;
    let batches = (queued / pool_size as f64).ceil() as u32;
    batches * hold_ms
}

/// Formula 4: pool timeout should exceed the worst wait plus one query.
pub fn recommended_timeout_ms(peak_wait_ms: u32, query_budget_ms: u32) -> u32 {
    peak_wait_ms + query_budget_ms
}

/// Formula 5: how many instances a given server can still accept.
pub fn max_instances(c: Capacity, per_instance: u32) -> u32 {
    if per_instance == 0 { return 0; }
    c.max_connections.saturating_sub(c.reserved_for_admin) / per_instance
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book_example() -> Capacity {
        // 8 instances, max_connections 200, 30 reserved: the Appendix C card.
        Capacity { app_instances: 8, max_connections: 200,
                   reserved_for_admin: 30, peak_concurrency_per_instance: 40 }
    }

    #[test]
    fn the_book_example_yields_twenty_one_connections_each() {
        let c = book_example();
        assert_eq!(pool_max_per_instance(c), 21);
        assert_eq!(peak_connections(c, 21), 168);
        assert_eq!(max_instances(c, 21), 8);
    }

    #[test]
    fn a_guessed_pool_size_overcommits_the_server() {
        let c = book_example();
        assert!(peak_connections(c, 40) > c.max_connections - c.reserved_for_admin);
    }

    #[test]
    fn wait_and_timeout_are_derived_not_guessed() {
        assert_eq!(wait_ms(40, 21, 10), 10);
        assert_eq!(recommended_timeout_ms(10, 250), 260);
        assert_eq!(max_instances(book_example(), 0), 0);
    }
}
