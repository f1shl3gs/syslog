use crate::Error;

/// Syslog Severities from RFC 5424.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum Severity {
    EMERG = 0,
    ALERT = 1,
    CRIT = 2,
    ERR = 3,
    WARNING = 4,
    NOTICE = 5,
    INFO = 6,
    DEBUG = 7,
}

/// Convert an int (as used in the wire serialization) into a `SyslogSeverity`
///
/// Returns an Option, but the wire protocol will only include 0..7, so should
/// never return None in practical usage.
impl TryFrom<i32> for Severity {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let severity = match value {
            0 => Severity::EMERG,
            1 => Severity::ALERT,
            2 => Severity::CRIT,
            3 => Severity::ERR,
            4 => Severity::WARNING,
            5 => Severity::NOTICE,
            6 => Severity::INFO,
            7 => Severity::DEBUG,
            _ => return Err(Error::BadSeverity),
        };

        Ok(severity)
    }
}

impl Severity {
    /// Convert a syslog severity into a unique string representation
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::EMERG => "emerg",
            Severity::ALERT => "alert",
            Severity::CRIT => "crit",
            Severity::ERR => "err",
            Severity::WARNING => "warning",
            Severity::NOTICE => "notice",
            Severity::INFO => "info",
            Severity::DEBUG => "debug",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Severity;

    #[test]
    fn deref() {
        assert_eq!(Severity::EMERG.as_str(), "emerg");
        assert_eq!(Severity::ALERT.as_str(), "alert");
        assert_eq!(Severity::CRIT.as_str(), "crit");
        assert_eq!(Severity::ERR.as_str(), "err");
        assert_eq!(Severity::WARNING.as_str(), "warning");
        assert_eq!(Severity::NOTICE.as_str(), "notice");
        assert_eq!(Severity::INFO.as_str(), "info");
        assert_eq!(Severity::DEBUG.as_str(), "debug");
    }
}
