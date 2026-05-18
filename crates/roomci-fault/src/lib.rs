use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ActiveFault {
    pub target: String,
    pub fault_type: String,
    pub starts_at: DateTime<FixedOffset>,
    pub ends_at: Option<DateTime<FixedOffset>>,
}

impl ActiveFault {
    pub fn is_active_at(&self, at: DateTime<FixedOffset>) -> bool {
        at >= self.starts_at && self.ends_at.map(|ends_at| at <= ends_at).unwrap_or(true)
    }
}
