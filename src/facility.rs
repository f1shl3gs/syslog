/// Syslog facilities. Taken From RFC 5424, but I've heard that some platforms mix these around.
/// Names are from Linux.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
#[allow(non_camel_case_types)]
pub enum Facility {
    KERN = 0,
    USER = 1,
    MAIL = 2,
    DAEMON = 3,
    AUTH = 4,
    SYSLOG = 5,
    LPR = 6,
    NEWS = 7,
    UUCP = 8,
    CRON = 9,
    AUTHPRIV = 10,
    FTP = 11,
    NTP = 12,
    AUDIT = 13,
    ALERT = 14,
    CLOCKD = 15,
    LOCAL0 = 16,
    LOCAL1 = 17,
    LOCAL2 = 18,
    LOCAL3 = 19,
    LOCAL4 = 20,
    LOCAL5 = 21,
    LOCAL6 = 22,
    LOCAL7 = 23,
}

impl Facility {
    /// Convert an int (as used in the wire serialization) into a `Facility`
    pub(crate) fn from_int(i: i32) -> Option<Self> {
        let fac = match i {
            0 => Facility::KERN,
            1 => Facility::USER,
            2 => Facility::MAIL,
            3 => Facility::DAEMON,
            4 => Facility::AUTH,
            5 => Facility::SYSLOG,
            6 => Facility::LPR,
            7 => Facility::NEWS,
            8 => Facility::UUCP,
            9 => Facility::CRON,
            10 => Facility::AUTHPRIV,
            11 => Facility::FTP,
            12 => Facility::NTP,
            13 => Facility::AUDIT,
            14 => Facility::ALERT,
            15 => Facility::CLOCKD,
            16 => Facility::LOCAL0,
            17 => Facility::LOCAL1,
            18 => Facility::LOCAL2,
            19 => Facility::LOCAL3,
            20 => Facility::LOCAL4,
            21 => Facility::LOCAL5,
            22 => Facility::LOCAL6,
            23 => Facility::LOCAL7,
            _ => return None,
        };

        Some(fac)
    }

    /// Convert a syslog facility into a unique string representation
    pub fn as_str(self) -> &'static str {
        match self {
            Facility::KERN => "kern",
            Facility::USER => "user",
            Facility::MAIL => "mail",
            Facility::DAEMON => "daemon",
            Facility::AUTH => "auth",
            Facility::SYSLOG => "syslog",
            Facility::LPR => "lpr",
            Facility::NEWS => "news",
            Facility::UUCP => "uucp",
            Facility::CRON => "cron",
            Facility::AUTHPRIV => "authpriv",
            Facility::FTP => "ftp",
            Facility::NTP => "ntp",
            Facility::AUDIT => "audit",
            Facility::ALERT => "alert",
            Facility::CLOCKD => "clockd",
            Facility::LOCAL0 => "local0",
            Facility::LOCAL1 => "local1",
            Facility::LOCAL2 => "local2",
            Facility::LOCAL3 => "local3",
            Facility::LOCAL4 => "local4",
            Facility::LOCAL5 => "local5",
            Facility::LOCAL6 => "local6",
            Facility::LOCAL7 => "local7",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Facility;

    #[test]
    fn test_deref() {
        assert_eq!(Facility::KERN.as_str(), "kern");
    }
}
