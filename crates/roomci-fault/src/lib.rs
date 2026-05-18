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
