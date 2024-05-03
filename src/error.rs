use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    BadSeverityInPri,
    BadFacilityInPri,
    UnexpectedEndOfInput,
    ExpectedChar(char),

    InvalidTimestamp,
    OutOfRangeTimezone,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadSeverityInPri => f.write_str("bad severity in message"),
            Error::BadFacilityInPri => f.write_str("bad facility in message"),
            Error::UnexpectedEndOfInput => f.write_str("unexpected eof"),
            Error::ExpectedChar(c) => write!(f, "unexpected input at character {c}"),
            // Timestamp
            Error::InvalidTimestamp => f.write_str("invalid timestamp"),
            Error::OutOfRangeTimezone => f.write_str("timezone offset is out of range"),
        }
    }
}
