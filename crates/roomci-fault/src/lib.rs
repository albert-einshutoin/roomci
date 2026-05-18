//! Fault-injection primitives shared across the roomci runtime.
//!
//! A scenario may declare faults (cloud broker offline, WAN down, edge power
//! lost, etc.) with a start time and optional end time. The runtime keeps an
//! [`ActiveFault`] for each declared fault and consults [`ActiveFault::is_active_at`]
//! when emitting events or evaluating assertions.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// A fault that has been activated in the scenario timeline.
///
/// `target` matches the scenario fault target (`mqtt.cloud`, `wan.primary`,
/// `edge.primary`, ...). `fault_type` matches the declared fault type
/// (`offline`, `power_lost`, ...). `ends_at` is `None` for faults that persist
/// for the rest of the scenario.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ActiveFault {
    pub target: String,
    pub fault_type: String,
    pub starts_at: DateTime<FixedOffset>,
    pub ends_at: Option<DateTime<FixedOffset>>,
}

impl ActiveFault {
    /// Returns `true` when the fault is active at `at`.
    ///
    /// A fault with no `ends_at` is active from `starts_at` onward.
    pub fn is_active_at(&self, at: DateTime<FixedOffset>) -> bool {
        at >= self.starts_at && self.ends_at.map(|ends_at| at <= ends_at).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(seconds: i64) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .unwrap()
            + chrono::Duration::seconds(seconds)
    }

    fn fault(starts: i64, ends: Option<i64>) -> ActiveFault {
        ActiveFault {
            target: "mqtt.cloud".to_string(),
            fault_type: "offline".to_string(),
            starts_at: at(starts),
            ends_at: ends.map(at),
        }
    }

    #[test]
    fn open_ended_fault_is_active_from_start_onward() {
        let active = fault(60, None);

        assert!(!active.is_active_at(at(0)));
        assert!(active.is_active_at(at(60)));
        assert!(active.is_active_at(at(3_600)));
    }

    #[test]
    fn bounded_fault_includes_start_and_end_inclusive() {
        let active = fault(30, Some(90));

        assert!(active.is_active_at(at(30)));
        assert!(active.is_active_at(at(60)));
        assert!(active.is_active_at(at(90)));
    }

    #[test]
    fn bounded_fault_is_inactive_before_start_and_after_end() {
        let active = fault(30, Some(90));

        assert!(!active.is_active_at(at(29)));
        assert!(!active.is_active_at(at(91)));
    }
}
