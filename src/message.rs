//! In-memory representation of a single Syslog message.

use chrono::{DateTime, FixedOffset};

use crate::facility;
use crate::procid::ProcId;
use crate::severity;
use crate::structured_data::StructuredElement;

/// A RFC5424-protocol syslog message
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message<S: AsRef<str> + Ord + PartialEq + Clone> {
    pub severity: severity::Severity,
    pub facility: facility::Facility,
    pub version: i32,
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub hostname: Option<S>,
    pub appname: Option<S>,
    pub procid: Option<ProcId<S>>,
    pub msgid: Option<S>,
    pub structured_data: Vec<StructuredElement<S>>,
    pub msg: S,
}
