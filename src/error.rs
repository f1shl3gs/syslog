use std::{fmt::Display, num::ParseIntError};

#[derive(Debug)]
pub enum Error {
    BadSeverityInPri,
    BadFacilityInPri,
    UnexpectedEndOfInput,
    TooFewDigits,
    TooManyDigits,
    InvalidUTCOffset,
    BaseUnicodeError(std::str::Utf8Error),
    UnicodeError(std::string::FromUtf8Error),
    ExpectedChar(char),
    IntConversion(ParseIntError),
    InvalidOffset,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadSeverityInPri => f.write_str("bad severity in message"),
            Error::BadFacilityInPri => f.write_str("bad facility in message"),
            Error::UnexpectedEndOfInput => f.write_str("unexpected eof"),
            Error::TooFewDigits => f.write_str("too few digits in numeric field"),
            Error::TooManyDigits => f.write_str("too many digits in numeric field"),
            Error::InvalidUTCOffset => f.write_str("invalid UTC offset"),
            Error::BaseUnicodeError(err) => write!(f, "unicode error: {err}"),
            Error::UnicodeError(err) => write!(f, "unicode error: {err}"),
            Error::ExpectedChar(c) => write!(f, "unexpected input at character {c}"),
            Error::IntConversion(err) => write!(f, "integer conversion error: {err}"),
            Error::InvalidOffset => f.write_str("date had invalid UTC offset"),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::BaseUnicodeError(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::UnicodeError(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::IntConversion(value)
    }
}
