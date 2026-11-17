//! Chapter 28 — The three-step migration.
//! Each phase must be deployable on its own, with old and new code running
//! side by side.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase { Expand, Backfill, Contract }

impl Phase {
    pub fn next(self) -> Option<Phase> {
        match self {
            Phase::Expand => Some(Phase::Backfill),
            Phase::Backfill => Some(Phase::Contract),
            Phase::Contract => None,
        }
    }
    pub fn ddl(self) -> &'static [&'static str] {
        match self {
            Phase::Expand => &[
                "ALTER TABLE usage_events ADD COLUMN billing_period TEXT",
                "ALTER TABLE usage_events ADD CONSTRAINT billing_period_check \
                 CHECK (billing_period IS NULL OR billing_period ~ '^[0-9]{6}$') NOT VALID",
            ],
            Phase::Backfill => &[
                "UPDATE usage_events SET billing_period = to_char(occurred_at, 'YYYYMM') \
                 WHERE billing_period IS NULL AND event_id > $1 LIMIT 10000",
                "ALTER TABLE usage_events VALIDATE CONSTRAINT billing_period_check",
            ],
            Phase::Contract => &[
                "ALTER TABLE usage_events ALTER COLUMN billing_period SET NOT NULL",
                "ALTER TABLE usage_events DROP COLUMN legacy_period",
            ],
        }
    }
}

/// SET NOT NULL may only run once every old row has a value. Deploying the
/// contract phase early is the classic 3 a.m. outage.
pub fn contract_is_safe(rows_with_null: u64) -> bool { rows_with_null == 0 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_phases_run_in_one_direction_and_stop() {
        assert_eq!(Phase::Expand.next(), Some(Phase::Backfill));
        assert_eq!(Phase::Backfill.next(), Some(Phase::Contract));
        assert_eq!(Phase::Contract.next(), None);
    }

    #[test]
    fn expand_uses_not_valid_so_it_does_not_scan_the_table() {
        assert!(Phase::Expand.ddl()[1].contains("NOT VALID"));
        assert!(Phase::Backfill.ddl()[1].contains("VALIDATE"));
    }

    #[test]
    fn contract_waits_for_the_backfill() {
        assert!(!contract_is_safe(1));
        assert!(contract_is_safe(0));
    }
}
