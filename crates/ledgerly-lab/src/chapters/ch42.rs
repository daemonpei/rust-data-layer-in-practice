//! Chapter 42 — Track an LSN watermark and move cold rows down a tier.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier { Hot, Warm, Cold }

pub fn tier_for_age_days(age_days: u64, hot_days: u64, warm_days: u64) -> Tier {
    if age_days < hot_days { Tier::Hot }
    else if age_days < warm_days { Tier::Warm }
    else { Tier::Cold }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LsnWatermark { pub last_flushed: u64, pub confirmed: u64 }

impl LsnWatermark {
    /// Never advance past what the replica confirmed. Advancing early loses
    /// WAL segments and breaks recovery.
    pub fn advance(&mut self, candidate: u64) -> u64 {
        self.last_flushed = candidate.min(self.confirmed);
        self.last_flushed
    }
    pub fn lag(&self, current: u64) -> u64 { current.saturating_sub(self.last_flushed) }
}

pub fn archival_sql(tier: Tier) -> &'static str {
    match tier {
        Tier::Hot => "-- no action",
        Tier::Warm => "SELECT * FROM usage_events WHERE occurred_at < now() - interval '90 days'",
        Tier::Cold => "COPY (SELECT * FROM usage_events \
                       WHERE occurred_at < now() - interval '365 days') TO PROGRAM 'gzip > /archive/part.csv.gz'",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_are_decided_by_age() {
        assert_eq!(tier_for_age_days(10, 90, 365), Tier::Hot);
        assert_eq!(tier_for_age_days(120, 90, 365), Tier::Warm);
        assert_eq!(tier_for_age_days(400, 90, 365), Tier::Cold);
    }

    #[test]
    fn the_watermark_never_outruns_the_replica() {
        let mut w = LsnWatermark { last_flushed: 100, confirmed: 150 };
        assert_eq!(w.advance(200), 150);
        assert_eq!(w.lag(180), 30);
    }

    #[test]
    fn cold_rows_go_to_compressed_storage() {
        assert!(archival_sql(Tier::Cold).contains("gzip"));
        assert_eq!(archival_sql(Tier::Hot), "-- no action");
    }
}
