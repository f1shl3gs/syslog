use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Error {
    BadSeverity,
    BadFacility,
    UnexpectedEndOfInput,
    ExpectedChar(char),
    ExpectSeparator,
    InvalidStructuredData,

    InvalidTimestamp,
    OutOfRangeTimezone,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadSeverity => f.write_str("bad severity in message"),
            Error::BadFacility => f.write_str("bad facility in message"),
            Error::UnexpectedEndOfInput => f.write_str("unexpected eof"),
            Error::ExpectedChar(c) => write!(f, "unexpected input at character {c}"),
            Error::ExpectSeparator => f.write_str("expect a separator"),
            Error::InvalidStructuredData => f.write_str("invalid structured data"),
            // Timestamp
            Error::InvalidTimestamp => f.write_str("invalid timestamp"),
            Error::OutOfRangeTimezone => f.write_str("timezone offset is out of range"),
        }
    }
}
