use crate::Error;
use chrono::{DateTime, FixedOffset, NaiveDate};

// get a character from the bytes as as a decimal
macro_rules! get_digit {
    ($bytes:ident, $index:expr, $error:ident) => {
        match $bytes.get($index) {
            Some(c) if c.is_ascii_digit() => c - b'0',
            _ => return Err(Error::$error),
        }
    };
}

pub fn parse_timestamp_rfc3339(buf: &[u8]) -> Result<DateTime<FixedOffset>, Error> {
    // First up, parse the full date if we can
    let (year, month, day) = parse_date(buf)?;

    // Next parse the separator between date and time
    let sep = buf.get(10).copied();
    if sep != Some(b'T') && sep != Some(b't') && sep != Some(b' ') && sep != Some(b'_') {
        return Err(Error::InvalidCharDateTimeSep);
    }

    // Next try to parse the time
    let (hour, minute, second, nanosecond, offset) = parse_time(buf, 11)?;

    let offset = FixedOffset::east_opt(offset.unwrap_or(0)).ok_or(Error::OutOfRangeTimezone)?;
    let datetime = NaiveDate::from_ymd_opt(year, month, day)
        .expect("year, month and day are checked already")
        .and_hms_nano_opt(hour, minute, second, nanosecond)
        .expect("hour, minute, second and nano second are checked already");

    #[allow(deprecated)]
    Ok(DateTime::from_local(datetime, offset))
}

fn parse_date(buf: &[u8]) -> Result<(i32, u32, u32), Error> {
    if buf.len() < 10 {
        return Err(Error::TimestampTooShort);
    }

    let (year, month, day) = unsafe {
        let year = convert_4_digits(&buf[..4]) as i32;
        match buf.get_unchecked(4) {
            b'-' => (),
            _ => return Err(Error::InvalidCharDateTimeSep),
        }

        let month = convert_2_digits(&buf[5..7]);
        match buf.get_unchecked(7) {
            b'-' => (),
            _ => return Err(Error::InvalidCharDateTimeSep),
        }

        let day = convert_2_digits(&buf[8..10]);

        (year, month, day)
    };

    // calculate the maximum number of days in the month, accounting for leap years in the
    // gregorian calendar
    let max_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => return Err(Error::OutOfRangeMonth),
    };

    if day < 1 || day > max_days {
        return Err(Error::OutOfRangeDay);
    }

    Ok((year, month, day))
}

/// Parse a time from bytes with a starting index, extra characters
/// at the end of the string result in an error
fn parse_time(buf: &[u8], offset: usize) -> Result<(u32, u32, u32, u32, Option<i32>), Error> {
    let (hour, minute, second, microsecond, mut position) =
        parse_time_without_timezone(buf, offset)?;

    // Parse the timezone offset
    let mut tz_offset: Option<i32> = None;

    if let Some(next_char) = buf.get(position).copied() {
        position += 1;
        if next_char == b'Z' || next_char == b'z' {
            tz_offset = Some(0);
        } else {
            let sign = match next_char {
                b'+' => 1,
                b'-' => -1,
                226 => {
                    // U+2212 MINUS "−" is allowed under ISO 8601 for negative timezones
                    // > python -c 'print([c for c in "−".encode()])'
                    // its raw byte values are [226, 136, 146]
                    if buf.get(position).copied() != Some(136) {
                        return Err(Error::InvalidCharTzSign);
                    }
                    if buf.get(position + 1).copied() != Some(146) {
                        return Err(Error::InvalidCharTzSign);
                    }
                    position += 2;
                    -1
                }
                _ => return Err(Error::InvalidCharTzSign),
            };

            let h1 = get_digit!(buf, position, InvalidCharTzHour) as i32;
            let h2 = get_digit!(buf, position + 1, InvalidCharTzHour) as i32;

            let m1 = match buf.get(position + 2) {
                Some(b':') => {
                    position += 3;
                    get_digit!(buf, position, InvalidCharTzMinute) as i32
                }
                Some(c) if c.is_ascii_digit() => {
                    position += 2;
                    (c - b'0') as i32
                }
                _ => return Err(Error::InvalidCharTzMinute),
            };
            let m2 = get_digit!(buf, position + 1, InvalidCharTzMinute) as i32;

            let minute_seconds = m1 * 600 + m2 * 60;
            if minute_seconds >= 3600 {
                return Err(Error::OutOfRangeTzMinute);
            }

            let offset_val = sign * (h1 * 36000 + h2 * 3600 + minute_seconds);
            // TZ must be less than 24 hours to match python
            if offset_val.abs() >= 24 * 3600 {
                return Err(Error::OutOfRangeTimezone);
            }
            tz_offset = Some(offset_val);
            position += 2;
        }
    }

    if buf.len() > position {
        return Err(Error::ExtraCharacters);
    }

    Ok((hour, minute, second, microsecond, tz_offset))
}

/// Parse time
///     * Hour: 0 to 23
///     * Minute: 0 to 59
///     * Second: 0 to 59
///     * NanoSecond: 0 to 999999999
///     * Position: position of the cursor after parsing
fn parse_time_without_timezone(
    buf: &[u8],
    offset: usize,
) -> Result<(u32, u32, u32, u32, usize), Error> {
    if buf.len() - offset < 5 {
        return Err(Error::TimestampTooShort);
    }

    let hour = convert_2_digits(&buf[offset..offset + 2]);
    if hour > 23 {
        return Err(Error::OutOfRangeHour);
    }
    match unsafe { buf.get_unchecked(offset + 2) } {
        b':' => (),
        _ => return Err(Error::InvalidCharTimeSep),
    }

    let minute = convert_2_digits(&buf[offset + 3..offset + 5]);
    if minute > 59 {
        return Err(Error::OutOfRangeMinute);
    }

    let mut length: usize = 5;
    let (second, nano_second) = match buf.get(offset + 5) {
        Some(b':') => {
            let second = convert_2_digits(&buf[offset + 6..offset + 8]);
            if second > 59 {
                return Err(Error::OutOfRangeSecond);
            }
            length = 8;

            let mut nano_second = 0;
            let frac_sep = buf.get(offset + 8).copied();
            if frac_sep == Some(b'.') || frac_sep == Some(b',') {
                length = 9;
                let mut i: usize = 0;
                loop {
                    match buf.get(offset + length + i) {
                        Some(c) if c.is_ascii_digit() => {
                            // If we've passed `i=6` then we are "truncating" the extra precision
                            // The easiest way to do this is to simply no-op and continue the loop
                            if i < 9 {
                                nano_second *= 10;
                                nano_second += (c - b'0') as u32;
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                    i += 1;
                    if i > 9 {
                        return Err(Error::SecondFractionTooLong);
                    }
                }
                if i == 0 {
                    return Err(Error::SecondFractionMissing);
                }
                if i < 9 {
                    nano_second *= 10_u32.pow(9 - i as u32);
                }
                length += i;
            }

            (second, nano_second)
        }
        _ => (0, 0),
    };

    Ok((
        hour as u32,
        minute as u32,
        second as u32,
        nano_second,
        offset + length,
    ))
}

#[inline]
fn convert_2_digits(digits: &[u8]) -> u32 {
    let bytes: [u8; 2] = digits.try_into().unwrap();
    let chunk = u16::from_ne_bytes(bytes) as u32;
    let lower = (chunk & 0x0f00) >> 8;
    let upper = (chunk & 0x000f) * 10;

    lower + upper
}

#[inline]
fn convert_4_digits(digits: &[u8]) -> u32 {
    let bytes: [u8; 4] = digits.try_into().unwrap();
    let mut chunk = u32::from_ne_bytes(bytes);

    let mut lower = (chunk & 0x0f000f00) >> 8;
    let mut upper = (chunk & 0x000f000f) * 10;

    chunk = lower + upper;

    lower = (chunk & 0x00ff0000) >> 16;
    upper = (chunk & 0x000000ff) * 100;

    lower + upper
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn timestamp() {
        let ts = parse_timestamp_rfc3339("2015-02-18T23:16:09Z".as_bytes()).unwrap();
        assert_eq!(
            FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2015, 2, 18, 23, 16, 9)
                .unwrap(),
            ts
        );

        let edt = FixedOffset::east_opt(5 * 60 * 60).unwrap();
        let ts = parse_timestamp_rfc3339("2015-02-18T23:59:59.234567+05:00".as_bytes()).unwrap();
        assert_eq!(
            edt.from_local_datetime(
                &NaiveDate::from_ymd_opt(2015, 2, 18)
                    .unwrap()
                    .and_hms_micro_opt(23, 59, 59, 234_567)
                    .unwrap()
            )
            .unwrap(),
            ts
        )
    }

    #[test]
    fn compare() {
        for input in ["1985-04-12T23:20:50.52Z"] {
            let got = parse_timestamp_rfc3339(input.as_bytes()).unwrap();
            let want = chrono::DateTime::parse_from_rfc3339(input).unwrap();
            assert_eq!(got, want, "input: {input}, want: {}", want.to_rfc3339())
        }
    }
}
