//! In-memory representation of a single Syslog message.

use chrono::{DateTime, FixedOffset};

use crate::facility;
use crate::procid::ProcId;
use crate::severity;
use crate::structured_data::StructuredElement;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Protocol {
    RFC3164,
    RFC5424(u32),
}

/// A RFC5424-protocol syslog message
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message<S: AsRef<str> + Ord + PartialEq + Clone> {
    pub severity: severity::Severity,
    pub facility: facility::Facility,
    pub protocol: Protocol,
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub hostname: Option<S>,
    pub appname: Option<S>,
    pub procid: Option<ProcId<S>>,
    pub msgid: Option<S>,
    // NOTE: param value is not escaped
    pub structured_data: Vec<StructuredElement<S>>,
    pub msg: S,
}
