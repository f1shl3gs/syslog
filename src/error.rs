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

    /// input is too short
    TimestampTooShort,
    /// invalid datetime separator, expected `T`, `t`, `_` or space
    InvalidCharDateTimeSep,
    /// timezone offset must be less than 24 hours
    OutOfRangeTimezone,
    /// month value is outside expected range of 1-12
    OutOfRangeMonth,
    /// day value is outside expected range
    OutOfRangeDay,
    /// invalid timezone sign
    InvalidCharTzSign,
    /// invalid timezone minute
    InvalidCharTzMinute,
    /// timezone minute value is outside expected range of 0-59
    OutOfRangeTzMinute,
    /// unexpected extra characters at the end of the input
    ExtraCharacters,
    /// hour value is outside expected range of 0-23
    OutOfRangeHour,
    /// minute value is outside expected range of 0-59
    OutOfRangeMinute,
    /// second fraction value is more than 6 digits long
    SecondFractionTooLong,
    /// second fraction digits missing after `.`
    SecondFractionMissing,
    /// invalid character in year
    InvalidCharYear,
    /// invalid character in month
    InvalidCharMonth,
    /// invalid character in day
    InvalidCharDay,
    /// invalid character in hour
    InvalidCharHour,
    /// invalid time separator, expected `:`
    InvalidCharTimeSep,
    /// invalid character in minute
    InvalidCharMinute,
    /// invalid character in second
    InvalidCharSecond,
    /// second value is outside expected range of 0-59
    OutOfRangeSecond,
    /// invalid timezone hour
    InvalidCharTzHour,
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
            Error::BaseUnicodeError(err) => write!(f, "unicode error: {err}"),
            Error::UnicodeError(err) => write!(f, "unicode error: {err}"),
            Error::ExpectedChar(c) => write!(f, "unexpected input at character {c}"),
            Error::IntConversion(err) => write!(f, "integer conversion error: {err}"),
            _ => todo!(),
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
