//! Chapter 25 — Retry only the errors that can succeed on a second attempt.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure { Deadlock, Serialization, PoolTimeout, UniqueViolation, Syntax, Unknown }

pub fn retryable(f: Failure) -> bool {
    matches!(f, Failure::Deadlock | Failure::Serialization | Failure::PoolTimeout)
}

/// Full jitter keeps a fleet of instances from retrying in lockstep.
pub fn backoff(attempt: u32, base: Duration, cap: Duration, jitter: f64) -> Duration {
    let multiplier = 1_u32.checked_shl(attempt.min(20)).unwrap_or(u32::MAX);
    let raw = Duration::from_millis(
        (base.as_millis() as u64).saturating_mul(multiplier as u64));
    let capped = raw.min(cap);
    let ms = capped.as_millis() as f64 * (1.0 - jitter.clamp(0.0, 1.0) * 0.5);
    Duration::from_millis(ms as u64)
}

pub fn sqlstate_to_failure(code: &str) -> Failure {
    match code {
        "40P01" => Failure::Deadlock,
        "40001" => Failure::Serialization,
        "23505" | "2067" => Failure::UniqueViolation,
        "42601" => Failure::Syntax,
        _ => Failure::Unknown,
    }
}

/// An idempotency key is required before any retry of a write.
pub fn may_retry_write(f: Failure, has_idempotency_key: bool) -> bool {
    retryable(f) && has_idempotency_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_transient_failures_are_retried() {
        assert!(retryable(Failure::Deadlock));
        assert!(!retryable(Failure::UniqueViolation));
        assert!(!retryable(Failure::Syntax));
        assert_eq!(sqlstate_to_failure("40P01"), Failure::Deadlock);
    }

    #[test]
    fn backoff_grows_and_stops_at_the_cap() {
        let base = Duration::from_millis(10);
        let cap = Duration::from_millis(200);
        assert!(backoff(0, base, cap, 0.0) < backoff(3, base, cap, 0.0));
        assert_eq!(backoff(20, base, cap, 0.0), cap);
    }

    #[test]
    fn a_write_without_an_idempotency_key_is_never_retried() {
        assert!(!may_retry_write(Failure::Deadlock, false));
        assert!(may_retry_write(Failure::Deadlock, true));
    }
}
