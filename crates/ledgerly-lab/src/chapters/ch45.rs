//! Chapter 45 — Nine checks before the launch. All of them are mechanical.

#[derive(Debug, Clone, Copy)]
pub struct LaunchState {
    pub pool_size_calculated: bool,
    pub migrations_rehearsed: bool,
    pub idempotency_keys_on_writes: bool,
    pub retry_policy_bounded: bool,
    pub slow_query_log_on: bool,
    pub tenant_predicate_audited: bool,
    pub backup_restore_tested: bool,
    pub rollback_plan_written: bool,
    pub on_call_dashboard_ready: bool,
}

impl LaunchState {
    pub fn failures(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if !self.pool_size_calculated { out.push("pool size was guessed"); }
        if !self.migrations_rehearsed { out.push("migration not rehearsed on a copy"); }
        if !self.idempotency_keys_on_writes { out.push("writes cannot be retried safely"); }
        if !self.retry_policy_bounded { out.push("retries have no cap or jitter"); }
        if !self.slow_query_log_on { out.push("no slow query signal"); }
        if !self.tenant_predicate_audited { out.push("tenant isolation untested"); }
        if !self.backup_restore_tested { out.push("restore never tested"); }
        if !self.rollback_plan_written { out.push("no rollback plan"); }
        if !self.on_call_dashboard_ready { out.push("nothing to look at when it breaks"); }
        out
    }
    pub fn go(&self) -> bool { self.failures().is_empty() }
}

pub fn all_green() -> LaunchState {
    LaunchState {
        pool_size_calculated: true, migrations_rehearsed: true,
        idempotency_keys_on_writes: true, retry_policy_bounded: true,
        slow_query_log_on: true, tenant_predicate_audited: true,
        backup_restore_tested: true, rollback_plan_written: true,
        on_call_dashboard_ready: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_green_checklist_launches() {
        assert!(all_green().go());
    }

    #[test]
    fn each_missing_control_blocks_the_launch() {
        let mut s = all_green();
        s.backup_restore_tested = false;
        assert!(!s.go());
        assert_eq!(s.failures(), vec!["restore never tested"]);
    }

    #[test]
    fn there_are_exactly_nine_controls() {
        let mut s = all_green();
        s.pool_size_calculated = false;
        s.migrations_rehearsed = false;
        s.idempotency_keys_on_writes = false;
        s.retry_policy_bounded = false;
        s.slow_query_log_on = false;
        s.tenant_predicate_audited = false;
        s.backup_restore_tested = false;
        s.rollback_plan_written = false;
        s.on_call_dashboard_ready = false;
        assert_eq!(s.failures().len(), 9);
    }
}
