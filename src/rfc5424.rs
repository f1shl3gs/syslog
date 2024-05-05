use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::message::Protocol;
use crate::{Error, Facility, Message, ProcId, Severity, StructuredElement};

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

#[inline]
fn to_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    nanos: u32,
    offset: i32,
) -> Result<DateTime<FixedOffset>, Error> {
    let offset = FixedOffset::east_opt(offset).ok_or(Error::OutOfRangeTimezone)?;
    let datetime = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or(Error::InvalidTimestamp)?
        .and_hms_nano_opt(hour, minute, second, nanos)
        .ok_or(Error::InvalidTimestamp)?;

    // DateTime::from_local() takes a lot time. it's almost 40% of the
    // timestamp benchmark
    #[allow(deprecated)]
    Ok(DateTime::from_local(datetime, offset))
}

// Parse rfc3339
//
// https://datatracker.ietf.org/doc/html/rfc3339
#[inline]
pub fn parse_timestamp(buf: &[u8], offset: &mut usize) -> Result<DateTime<FixedOffset>, Error> {
    let len = buf.len();
    // 20 is the length of `1990-12-31T23:59:60Z`
    if len - *offset < 20 {
        return Err(Error::InvalidTimestamp);
    }

    let year = convert_4_digits(&buf[*offset..*offset + 4]) as i32;

    if buf[*offset + 4] != b'-' {
        return Err(Error::InvalidTimestamp);
    }

    *offset += 5;
    let month = convert_2_digits(&buf[*offset..*offset + 2]);

    if buf[*offset + 2] != b'-' {
        return Err(Error::InvalidTimestamp);
    }

    *offset += 3;
    let day = convert_2_digits(&buf[*offset..*offset + 2]);

    if buf[*offset + 2] != b'T' {
        return Err(Error::InvalidTimestamp);
    }
    *offset += 3;

    let hour = convert_2_digits(&buf[*offset..*offset + 2]);
    if buf[*offset + 2] != b':' {
        return Err(Error::InvalidTimestamp);
    }
    *offset += 3;

    let minute = convert_2_digits(&buf[*offset..*offset + 2]);
    if buf[*offset + 2] != b':' {
        return Err(Error::InvalidTimestamp);
    }
    *offset += 3;

    let second = convert_2_digits(&buf[*offset..*offset + 2]);
    *offset += 2;

    let next_char = buf[*offset];
    let nanos = if next_char == b'.' || next_char == b',' {
        let mut nanos = 0u32;
        let mut count = 0;
        *offset += 1;
        let end = std::cmp::min(*offset + 9, len);
        for ch in &buf[*offset..end] {
            if !ch.is_ascii_digit() {
                break;
            }

            count += 1;
            nanos = (nanos * 10) + (ch - b'0') as u32;
        }

        *offset += count;
        nanos * 10u32.pow(9 - count as u32)
    } else if next_char == b'z' || next_char == b'Z' {
        // no nanos, no offset. e.g. `1990-12-31T23:59:60Z`
        return to_datetime(year, month, day, hour, minute, second, 0, 0);
    } else {
        0
    };

    let sign = match buf[*offset] {
        b'z' | b'Z' => {
            // no offset. e.g. `1990-12-31T23:59:60Z`
            *offset += 1;
            return to_datetime(year, month, day, hour, minute, second, nanos, 0);
        }
        b'+' => 1,
        b'-' => -1,
        _ => return Err(Error::InvalidTimestamp),
    };

    *offset += 1;
    if len - *offset < 5 {
        return Err(Error::InvalidTimestamp);
    }

    let h = convert_2_digits(&buf[*offset..*offset + 2]) as i32;
    if buf[*offset + 2] != b':' {
        return Err(Error::InvalidTimestamp);
    }
    let m = convert_2_digits(&buf[*offset + 3..*offset + 5]) as i32;

    *offset += 5;

    to_datetime(
        year,
        month,
        day,
        hour,
        minute,
        second,
        nanos,
        sign * (h * 60 * 60 + m * 60),
    )
}

// SIMD is great but it is might not suitable here. Cause, in our case, the string is short.
#[inline]
fn take_until_whitespace<'a>(buf: &'a [u8], offset: &mut usize) -> Result<&'a str, Error> {
    for pos in *offset..buf.len() {
        if buf[pos] == b' ' {
            let value = unsafe { std::str::from_utf8_unchecked(&buf[*offset..pos]) };
            *offset = pos;
            return Ok(value);
        }
    }

    Err(Error::UnexpectedEndOfInput)
}

fn parse_sd_params<'a>(
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<Vec<(&'a str, &'a str)>, Error> {
    let mut params = Vec::with_capacity(4);

    loop {
        let key = parse_param_key(buf, offset)?;

        if buf[*offset] != b'=' {
            return Err(Error::ExpectedChar('='));
        }
        *offset += 1;

        let value = parse_param_value(buf, offset)?;

        params.push((key, value));

        match buf[*offset] {
            b']' => {
                *offset += 1;
                break;
            }
            b' ' => {
                *offset += 1;
                continue;
            }
            _ch => return Err(Error::InvalidStructuredData),
        }
    }

    Ok(params)
}

#[inline]
fn parse_param_key<'a>(buf: &'a [u8], offset: &mut usize) -> Result<&'a str, Error> {
    for pos in *offset..buf.len() {
        let ch = buf[pos];

        if ch == b'=' || ch == b']' {
            let key = unsafe { std::str::from_utf8_unchecked(&buf[*offset..pos]) };
            *offset = pos;
            return Ok(key);
        }
    }

    Err(Error::UnexpectedEndOfInput)
}

#[inline]
fn parse_param_value<'a>(buf: &'a [u8], offset: &mut usize) -> Result<&'a str, Error> {
    if buf[*offset] != b'"' {
        return Err(Error::ExpectedChar('"'));
    }
    *offset += 1;

    for pos in *offset..buf.len() {
        if buf[pos] == b'"' {
            let value = unsafe { std::str::from_utf8_unchecked(&buf[*offset..pos]) };
            *offset = pos + 1; // 1 for the double quota
            return Ok(value);
        }
    }

    Err(Error::UnexpectedEndOfInput)
}

// example: [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"]
#[inline]
fn parse_structured_element<'a>(
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<StructuredElement<&'a str>, Error> {
    if buf[*offset] != b'[' {
        return Err(Error::ExpectedChar('['));
    }
    *offset += 1;

    // empty structured element, e.g. `[]`
    if buf[*offset] == b']' {
        *offset += 1;
        return Ok(StructuredElement {
            id: "",
            params: vec![],
        });
    }

    // parse id
    let mut id = "";
    for pos in *offset..buf.len() {
        let ch = buf[pos];
        if ch == b' ' {
            id = unsafe { std::str::from_utf8_unchecked(&buf[*offset..pos]) };
            *offset = pos + 1;
            break;
        }

        if ch == b']' {
            // just id no key-value pairs
            id = unsafe { std::str::from_utf8_unchecked(&buf[*offset..pos]) };
            *offset = pos + 1;
            return Ok(StructuredElement { id, params: vec![] });
        }
    }

    // parse params
    let params = parse_sd_params(buf, offset)?;

    Ok(StructuredElement { id, params })
}

fn parse_structured_data<'a>(
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<Vec<StructuredElement<&'a str>>, Error> {
    // 4 is RawVec::MIN_NON_ZERO_CAP
    let mut elements = Vec::with_capacity(4);

    loop {
        let element = parse_structured_element(buf, offset)?;
        elements.push(element);

        // 1. empty message(aka STRUCTURED-DATA Only),
        // 2. structured data is done
        if *offset == buf.len() || buf[*offset] == b' ' {
            break;
        }
    }

    Ok(elements)
}

/// Parse an array of bytes into a `Message` object
///
/// NOTE: `SIMD` is great, but it might not be suitable here, cause our
/// header part is relatively short, so the performance might not be
/// as good as we expected.
pub fn parse_message(buf: &[u8]) -> Result<Message<&str>, Error> {
    let len = buf.len();

    // Parse priority
    //
    // https://datatracker.ietf.org/doc/html/rfc5424#section-6.2.1
    if len < 4 || buf[0] != b'<' {
        return Err(Error::ExpectedChar('<'));
    }

    let mut offset = 1;
    let mut prival = 0i32;
    for pos in 1..len {
        let ch = buf[pos];
        if !ch.is_ascii_digit() {
            if ch == b'>' {
                offset = pos + 1;
                break;
            }

            return Err(Error::ExpectedChar(ch as char));
        }

        prival = (prival * 10) + (ch - b'0') as i32;
    }

    let severity = Severity::try_from(prival & 0x7)?;
    let facility = Facility::try_from(prival >> 3)?;

    // Parse version
    //
    // https://datatracker.ietf.org/doc/html/rfc5424#section-9.1
    let version = {
        let ch = buf[offset];
        if !ch.is_ascii_digit() {
            return Err(Error::ExpectedChar(ch as char));
        }

        offset += 1;
        (ch - b'0') as u32
    };

    if buf[offset] != b' ' {
        return Err(Error::ExpectSeparator);
    }
    offset += 1;

    // Parse timestamp
    let timestamp = if buf[offset] == b'-' {
        offset += 1;
        None
    } else {
        Some(parse_timestamp(buf, &mut offset)?)
    };

    if buf[offset] != b' ' {
        return Err(Error::ExpectSeparator);
    }
    offset += 1;

    let hostname = if buf[offset] == b'-' {
        offset += 1;
        None
    } else {
        Some(take_until_whitespace(buf, &mut offset)?)
    };

    if buf[offset] != b' ' {
        return Err(Error::ExpectSeparator);
    }
    offset += 1;

    let appname = if buf[offset] == b'-' {
        offset += 1;
        None
    } else {
        Some(take_until_whitespace(buf, &mut offset)?)
    };

    if buf[offset] != b' ' {
        return Err(Error::ExpectSeparator);
    }
    offset += 1;

    let procid = if buf[offset] == b'-' {
        offset += 1;
        None
    } else {
        let s = take_until_whitespace(buf, &mut offset)?;
        match s.parse() {
            Ok(id) => Some(ProcId::PID(id)),
            _ => Some(ProcId::Name(s)),
        }
    };

    if buf[offset] != b' ' {
        return Err(Error::ExpectSeparator);
    }
    offset += 1;

    let msgid = if buf[offset] == b'-' {
        offset += 1;
        None
    } else {
        Some(take_until_whitespace(buf, &mut offset)?)
    };

    if buf[offset] != b' ' {
        return Err(Error::ExpectSeparator);
    }
    offset += 1;

    // structured data
    let structured_data = if buf[offset] == b'-' {
        offset += 1;
        Vec::new()
    } else {
        parse_structured_data(buf, &mut offset)?
    };

    // message
    if offset < len && buf[offset] == b' ' {
        offset += 1;
    }
    let msg = unsafe { std::str::from_utf8_unchecked(&buf[offset..]) };

    Ok(Message {
        severity,
        facility,
        protocol: Protocol::RFC5424(version),
        timestamp,
        hostname,
        appname,
        procid,
        msgid,
        structured_data,
        msg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_2_digits() {
        for i in 0..99 {
            let s = format!("{:2}", i);
            let actual = convert_2_digits(&s.as_bytes()[..2]);
            assert_eq!(actual, i);
        }
    }

    #[test]
    fn test_convert_4_digits() {
        for i in 0..9999 {
            let s = format!("{:4}", i);
            let actual = convert_4_digits(&s.as_bytes()[..4]);
            assert_eq!(i, actual);
        }
    }

    #[test]
    fn rfc5424_examples() {
        // https://datatracker.ietf.org/doc/html/rfc5424#section-6.5
        for input in [
            r##"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - BOM'su root' failed for lonvick on /dev/pts/8"##,
            r##"<165>1 2003-08-24T05:14:15.000003-07:00 192.0.2.1 myproc 8710 - - %% It's time to make the do-nuts."##,
            r##"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"] BOMAn application event log entry..."##,
            r##"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"][examplePriority@32473 class="high"]"##,
        ] {
            let _msg = parse_message(input.as_bytes()).unwrap();
        }
    }

    #[test]
    fn timestamp() {
        // https://datatracker.ietf.org/doc/html/rfc3339#section-5.8
        for input in [
            "1985-04-12T23:20:50.52Z",
            "1985-04-12T23:20:50.123456789Z",
            "1996-12-19T16:39:57-08:00",
            "1990-12-31T23:59:59Z",
            "1990-12-31T15:59:59-08:00",
            "1937-01-01T12:00:27.87+00:20",
        ] {
            let ref mut offset = 0;
            let got = parse_timestamp(input.as_bytes(), offset).unwrap();
            let want = chrono::DateTime::parse_from_rfc3339(input).unwrap();
            assert_eq!(got, want, "input: {input}, want: {}", want.to_rfc3339())
        }
    }

    #[test]
    fn multiple_structured_data() {
        let input = b"[exampleSDID@32473 iut=\"3\" eventSource=\"Application\"][examplePriority@32473 class=\"high\"] BOMAn application event log entry...";

        let elements = parse_structured_data(input, &mut 0).unwrap();
        assert_eq!(elements.len(), 2);
    }

    #[test]
    fn empty_structured_data() {
        for input in ["[] "] {
            let _ = parse_structured_data(input.as_bytes(), &mut 0).unwrap();
        }
    }
}
