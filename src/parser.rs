use std::str::{from_utf8, FromStr};

use chrono::{DateTime, FixedOffset, LocalResult, NaiveDate, TimeZone};

use super::{Error, Facility, Message, ProcId, Severity, StructuredElement};

// We parse with this super-duper-dinky hand-coded recursive descent parser because we don't really
// have much other choice:
//
//  - Regexp is much slower (at least a factor of 4), and we still end up having to parse the
//    somewhat-irregular SD
//  - LALRPOP requires non-ambiguous tokenization
//  - Rust-PEG doesn't work on anything except nightly
//
// So here we are. The macros make it a bit better.
//
// General convention is that the parse state is represented by a string slice named "rest"; the
// macros will update that slice as they consume tokens.

macro_rules! maybe_expect_char {
    ($s:expr, $e: expr) => {
        match $s.chars().next() {
            Some($e) => Some(&$s[1..]),
            _ => None,
        }
    };
}

macro_rules! take_item {
    ($e:expr, $r:expr) => {{
        let (t, r) = $e?;
        $r = r;
        t
    }};
}

macro_rules! take_char {
    ($e: expr, $c:expr) => {{
        $e = match $e.chars().next() {
            Some($c) => &$e[1..],
            Some(_) => return Err(Error::ExpectedChar($c)),
            None => return Err(Error::UnexpectedEndOfInput),
        }
    }};
}

fn take_while<F>(input: &str, f: F, max_chars: usize) -> (&str, Option<&str>)
where
    F: Fn(char) -> bool,
{
    for (idx, chr) in input.char_indices() {
        if !f(chr) {
            return (&input[..idx], Some(&input[idx..]));
        }
        if idx == max_chars {
            return (&input[..idx], Some(&input[idx..]));
        }
    }
    ("", None)
}

fn parse_sd_id(input: &str) -> Result<(&str, &str), Error> {
    let (res, rest) = take_while(input, |c| c != ' ' && c != '=' && c != ']', 128);
    if let Some(rest) = rest {
        Ok((res, rest))
    } else {
        Err(Error::UnexpectedEndOfInput)
    }
}

/** Parse a `param_value`... a.k.a. a quoted string */
fn parse_param_value(input: &str) -> Result<(&str, &str), Error> {
    let mut rest = input;
    take_char!(rest, '"');

    let mut escaped = false;
    for (index, ch) in rest.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '"' {
            return Ok((&rest[..index], &rest[index + 1..]));
        }
    }

    Err(Error::UnexpectedEndOfInput)
}

fn parse_sd_params(input: &str) -> Result<(Vec<(&str, &str)>, &str), Error> {
    let mut params = Vec::new();
    let mut top = input;
    loop {
        if let Some(rest2) = maybe_expect_char!(top, ' ') {
            let mut rest = rest2;
            let name = take_item!(parse_sd_id(rest), rest);
            take_char!(rest, '=');
            let value = take_item!(parse_param_value(rest), rest);

            params.push((name, value));
            top = rest;
        } else {
            return Ok((params, top));
        }
    }
}

fn parse_sde(sde: &str) -> Result<((&str, Vec<(&str, &str)>), &str), Error> {
    let mut rest = sde;
    take_char!(rest, '[');
    let id = take_item!(parse_sd_id(rest), rest);
    let params = take_item!(parse_sd_params(rest), rest);
    take_char!(rest, ']');
    Ok(((id, params), rest))
}

fn parse_structured_data(input: &str) -> Result<(Vec<StructuredElement<&str>>, &str), Error> {
    let mut sd = vec![];

    if let Some(rest) = input.strip_prefix('-') {
        return Ok((sd, rest));
    }

    let mut rest = input;
    while !rest.is_empty() {
        let (id, params) = take_item!(parse_sde(rest), rest);
        sd.push(StructuredElement { id, params });
        if rest.starts_with(' ') {
            break;
        }
    }

    Ok((sd, rest))
}

fn parse_pri_val(pri: i32) -> Result<(Severity, Facility), Error> {
    let sev = Severity::from_int(pri & 0x7).ok_or(Error::BadSeverityInPri)?;
    let fac = Facility::from_int(pri >> 3).ok_or(Error::BadFacilityInPri)?;
    Ok((sev, fac))
}

/// Parse an i32
fn parse_num<T>(s: &str, min_digits: usize, max_digits: usize) -> Result<(T, &str), Error>
where
    T: FromStr<Err = std::num::ParseIntError>,
{
    let (res, rest1) = take_while(s, |c| ('0'..='9').contains(&c), max_digits);
    let rest = rest1.ok_or(Error::UnexpectedEndOfInput)?;
    if res.len() < min_digits {
        Err(Error::TooFewDigits)
    } else if res.len() > max_digits {
        Err(Error::TooManyDigits)
    } else {
        Ok((T::from_str(res)?, rest))
    }
}

fn parse_decimal(d: &str, min_digits: usize, max_digits: usize) -> Result<(i32, &str), Error> {
    parse_num::<i32>(d, min_digits, max_digits).map(|(val, s)| {
        let mut multiplicand = 1;
        let z = 10 - (d.len() - s.len());

        for _i in 1..(z) {
            multiplicand *= 10;
        }
        (val * multiplicand, s)
    })
}

fn parse_timestamp(m: &str) -> Result<(Option<DateTime<FixedOffset>>, &str), Error> {
    let mut rest = m;
    if let Some(rest) = rest.strip_prefix('-') {
        return Ok((None, rest));
    }

    let year = take_item!(parse_num(rest, 4, 4), rest);
    take_char!(rest, '-');
    let month = take_item!(parse_num(rest, 2, 2), rest);
    take_char!(rest, '-');
    let day = take_item!(parse_num(rest, 2, 2), rest);

    take_char!(rest, 'T');
    let hour = take_item!(parse_num(rest, 2, 2), rest);
    take_char!(rest, ':');
    let minute = take_item!(parse_num(rest, 2, 2), rest);
    take_char!(rest, ':');
    let second = take_item!(parse_num(rest, 2, 2), rest);
    let nano = if rest.starts_with('.') {
        take_char!(rest, '.');
        take_item!(parse_decimal(rest, 1, 6), rest) as u32
    } else {
        0
    };

    let offset = match rest.chars().next() {
        None => 0,
        Some('Z') => {
            rest = &rest[1..];
            0
        }
        Some(c) => {
            let (sign, irest) = match c {
                // Note: signs are backwards as per RFC3339
                '-' => (-1, &rest[1..]),
                '+' => (1, &rest[1..]),
                _ => {
                    return Err(Error::InvalidUTCOffset);
                }
            };
            let hours = i32::from_str(&irest[0..2])?;
            let minutes = i32::from_str(&irest[3..5])?;
            rest = &irest[5..];
            sign * (hours * 60 * 60 + minutes * 60)
        }
    };

    let offset = FixedOffset::east_opt(offset).ok_or(Error::InvalidOffset)?;
    let datetime = NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_nano_opt(hour, minute, second, nano)
        .unwrap();

    if let LocalResult::Single(ts) = offset.from_local_datetime(&datetime) {
        Ok((Some(ts), rest))
    } else {
        Err(Error::InvalidOffset)
    }
}

fn parse_term(
    m: &str,
    min_length: usize,
    max_length: usize,
) -> Result<(Option<&str>, &str), Error> {
    if m.starts_with('-') && (m.len() <= 1 || m.as_bytes()[1] == 0x20) {
        return Ok((None, &m[1..]));
    }
    let byte_ary = m.as_bytes();
    for (idx, chr) in byte_ary.iter().enumerate() {
        if *chr < 33 || *chr > 126 {
            if idx < min_length {
                return Err(Error::TooFewDigits);
            }
            let utf8_ary = from_utf8(&byte_ary[..idx])?;
            return Ok((Some(utf8_ary), &m[idx..]));
        }
        if idx >= max_length {
            let utf8_ary = from_utf8(&byte_ary[..idx])?;
            return Ok((Some(utf8_ary), &m[idx..]));
        }
    }

    Err(Error::UnexpectedEndOfInput)
}

/// Parse a string into a `Message` object
///
/// # Arguments
///
///  * `s`: Anything convertible to a string
///
/// # Returns
///
///  * `ParseErr` if the string is not parseable as an RFC5424 message
///
/// # Example
///
/// ```no_run
/// use syslog::parse_message;
///
/// let message = parse_message("<78>1 2016-01-15T00:04:01+00:00 host1 CROND 10391 - [meta sequenceId=\"29\"] some_message").unwrap();
///
/// assert!(message.hostname.unwrap() == "host1");
/// ```
pub fn parse_message(input: &str) -> Result<Message<&str>, Error> {
    let mut rest = input;
    take_char!(rest, '<');
    let prival = take_item!(parse_num(rest, 1, 3), rest);
    take_char!(rest, '>');
    let (sev, fac) = parse_pri_val(prival)?;
    let version = take_item!(parse_num(rest, 1, 2), rest);
    take_char!(rest, ' ');
    let timestamp = take_item!(parse_timestamp(rest), rest);
    take_char!(rest, ' ');
    let hostname = take_item!(parse_term(rest, 1, 255), rest);
    take_char!(rest, ' ');
    let appname = take_item!(parse_term(rest, 1, 48), rest);
    take_char!(rest, ' ');
    let procid = take_item!(parse_term(rest, 1, 128), rest).map(|s| match i32::from_str(&s) {
        Ok(n) => ProcId::PID(n),
        Err(_) => ProcId::Name(s),
    });
    take_char!(rest, ' ');
    let msgid = take_item!(parse_term(rest, 1, 32), rest);
    take_char!(rest, ' ');
    let structured_data = take_item!(parse_structured_data(rest), rest);
    let msg = match maybe_expect_char!(rest, ' ') {
        Some(r) => r,
        None => rest,
    };

    Ok(Message {
        severity: sev,
        facility: fac,
        version,
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
    use std::mem;

    use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};

    use super::*;

    #[test]
    fn rfc5424_examples() {
        // https://datatracker.ietf.org/doc/html/rfc5424#section-6.5
        for input in [
            r##"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - BOM'su root' failed for lonvick on /dev/pts/8"##,
            r##"<165>1 2003-08-24T05:14:15.000003-07:00 192.0.2.1 myproc 8710 - - %% It's time to make the do-nuts."##,
            r##"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"] BOMAn application event log entry..."##,
            r##"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"][examplePriority@32473 class="high"]"##,
        ] {
            let _msg = parse_message(input).unwrap();
        }
    }

    // #[test]
    // fn rfc3164_examples() {
    //     // https://datatracker.ietf.org/doc/html/rfc3164#section-5.4
    //     for input in [
    //         r##"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8"##,
    //         r##"<13>Feb  5 17:32:18 10.0.0.99 Use the BFG!"##,
    //         r##"<165>Aug 24 05:34:00 CST 1987 mymachine myproc[10]: %% It's time to make the do-nuts.  %%  Ingredients: Mix=OK, Jelly=OK # Devices: Mixer=OK, Jelly_Injector=OK, Frier=OK # Transport: Conveyer1=OK, Conveyer2=OK # %%"##,
    //         r##"<0>1990 Oct 22 10:52:01 TZ-6 scapegoat.dmz.example.org 10.1.2.3 sched[0]: That's All Folks!"##,
    //     ] {
    //         let _msg = parse_message(input).unwrap();
    //     }
    // }

    #[test]
    fn test_simple() {
        let msg = parse_message("<1>1 - - - - - -").expect("Should parse empty message");
        assert!(msg.facility == Facility::KERN);
        assert!(msg.severity == Severity::ALERT);
        assert!(msg.timestamp.is_none());
        assert!(msg.hostname.is_none());
        assert!(msg.appname.is_none());
        assert!(msg.procid.is_none());
        assert!(msg.msgid.is_none());
        assert!(msg.structured_data.is_empty());
    }

    #[test]
    fn test_with_time_zulu() {
        let msg = parse_message("<1>1 2015-01-01T00:00:00Z host - - - -")
            .expect("Should parse empty message");
        let want = DateTime::parse_from_rfc3339("2015-01-01T00:00:00Z").unwrap();
        assert_eq!(msg.timestamp, Some(want));
    }

    #[test]
    fn test_with_time_offset() {
        let msg = parse_message("<1>1 2015-01-01T00:00:00+00:00 - - - - -")
            .expect("Should parse empty message");
        let want = DateTime::parse_from_rfc3339("2015-01-01T00:00:00+00:00").unwrap();
        assert_eq!(msg.timestamp, Some(want));
    }

    #[test]
    fn test_with_time_offset_nonzero() {
        let msg = parse_message("<1>1 2015-01-01T00:00:00-10:00 - - - - -")
            .expect("Should parse empty message");
        let want = DateTime::parse_from_rfc3339("2015-01-01T00:00:00-10:00").unwrap();
        assert_eq!(msg.timestamp, Some(want));
        // example from RFC 3339
        let msg1 = parse_message("<1>1 2015-01-01T18:50:00-04:00 - - - - -")
            .expect("Should parse empty message");
        let msg2 = parse_message("<1>1 2015-01-01T22:50:00Z - - - - -")
            .expect("Should parse empty message");
        assert_eq!(msg1.timestamp, msg2.timestamp);
        // example with fractional minutes
        let msg1 = parse_message("<1>1 2019-01-20T00:46:39+05:45 - - - - -")
            .expect("Should parse empty message");
        let msg2 = parse_message("<1>1 2019-01-19T11:01:39-08:00 - - - - -")
            .expect("Should parse empty message");
        assert_eq!(msg1.timestamp, msg2.timestamp);
    }

    #[test]
    fn test_complex() {
        let msg = parse_message("<78>1 2016-01-15T00:04:01+00:00 host1 CROND 10391 - [meta sequenceId=\"29\"] some_message").expect("Should parse complex message");
        assert_eq!(msg.facility, Facility::CRON);
        assert_eq!(msg.severity, Severity::INFO);
        assert_eq!(msg.hostname, Some("host1"));
        assert_eq!(msg.appname, Some("CROND"));
        assert_eq!(msg.procid, Some(ProcId::PID(10391)));
        assert_eq!(msg.msg, "some_message");
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("2016-01-15T00:04:01+00:00").unwrap())
        );
        assert_eq!(msg.structured_data.len(), 1);
        msg.structured_data
            .iter()
            .find(|element| {
                if element.id != "meta" {
                    return false;
                }

                element
                    .params
                    .iter()
                    .find(|(k, v)| *k == "sequenceId" && *v == "29")
                    .is_some()
            })
            .expect("Should contain meta sequenceId");
    }

    #[test]
    fn test_sd_empty() {
        let msg = parse_message(
            "<78>1 2016-01-15T00:04:01Z host1 CROND 10391 - [meta@1234] some_message",
        )
        .expect("Should parse message with empty structured data");
        assert_eq!(msg.facility, Facility::CRON);
        assert_eq!(msg.severity, Severity::INFO);
        assert_eq!(msg.hostname, Some("host1"));
        assert_eq!(msg.appname, Some("CROND"));
        assert_eq!(msg.procid, Some(ProcId::PID(10391)));
        assert_eq!(msg.msg, String::from("some_message"));
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("2016-01-15T00:04:01Z").unwrap())
        );
        assert_eq!(msg.structured_data.len(), 1);
        assert_eq!(
            msg.structured_data
                .iter()
                .find(|element| element.id == "meta@1234")
                .expect("should contain meta")
                .params
                .len(),
            0
        );
    }

    #[test]
    fn test_sd_features() {
        let msg = parse_message("<78>1 2016-01-15T00:04:01Z host1 CROND 10391 - [meta sequenceId=\"29\" sequenceBlah=\"foo\"][my key=\"value\"][meta bar=\"baz=\"] some_message").expect("Should parse complex message");
        assert_eq!(msg.facility, Facility::CRON);
        assert_eq!(msg.severity, Severity::INFO);
        assert_eq!(msg.hostname, Some("host1"));
        assert_eq!(msg.appname, Some("CROND"));
        assert_eq!(msg.procid, Some(ProcId::PID(10391)));
        assert_eq!(msg.msg, String::from("some_message"));
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("2016-01-15T00:04:01Z").unwrap())
        );
        assert_eq!(msg.structured_data.len(), 3);
        assert_eq!(
            msg.structured_data.iter().fold(0, |acc, item| {
                if item.id == "meta" {
                    item.params.len() + acc
                } else {
                    acc
                }
            }),
            3
        );
    }

    #[test]
    fn test_sd_with_escaped_quote() {
        let msg_text = r#"<1>1 - - - - - [meta key="val\"ue"] message"#;
        let msg = parse_message(msg_text).expect("should parse");

        let (key, value) = msg
            .structured_data
            .iter()
            .find(|element| element.id == "meta")
            .unwrap()
            .params
            .first()
            .unwrap();

        assert_eq!(*key, "key");
        assert_eq!(*value, r#"val\"ue"#);
    }

    #[test]
    fn test_other_message() {
        let msg_text = r#"<190>1 2016-02-21T01:19:11+00:00 batch6sj - - - [meta sequenceId="21881798" x-group="37051387"][origin x-service="tracking"] metascutellar conversationalist nephralgic exogenetic graphy streng outtaken acouasm amateurism prenotice Lyonese bedull antigrammatical diosphenol gastriloquial bayoneteer sweetener naggy roughhouser dighter addend sulphacid uneffectless ferroprussiate reveal Mazdaist plaudite Australasian distributival wiseman rumness Seidel topazine shahdom sinsion mesmerically pinguedinous ophthalmotonometer scuppler wound eciliate expectedly carriwitchet dictatorialism bindweb pyelitic idic atule kokoon poultryproof rusticial seedlip nitrosate splenadenoma holobenthic uneternal Phocaean epigenic doubtlessly indirection torticollar robomb adoptedly outspeak wappenschawing talalgia Goop domitic savola unstrafed carded unmagnified mythologically orchester obliteration imperialine undisobeyed galvanoplastical cycloplegia quinquennia foremean umbonal marcgraviaceous happenstance theoretical necropoles wayworn Igbira pseudoangelic raising unfrounced lamasary centaurial Japanolatry microlepidoptera"#;
        parse_message(msg_text).expect("should parse as text");
    }

    #[test]
    fn test_bad_pri() {
        let msg = parse_message("<4096>1 - - - - - -");
        assert!(msg.is_err());
    }

    #[test]
    fn test_bad_match() {
        // we shouldn't be able to parse RFC3164 messages
        let msg = parse_message("<134>Feb 18 20:53:31 haproxy[376]: I am a message");
        assert!(msg.is_err());
    }

    #[test]
    fn test_example_timestamps() {
        // these are the example timestamps in the rfc

        let msg = parse_message("<1>1 1985-04-12T23:20:50.52Z host - - - -")
            .expect("Should parse empty message");
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("1985-04-12T23:20:50.52Z").unwrap())
        );

        let msg = parse_message("<1>1 1985-04-12T19:20:50.52+04:00 host - - - -")
            .expect("Should parse empty message");
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("1985-04-12T19:20:50.52+04:00").unwrap())
        );

        let msg = parse_message("<1>1 1985-04-12T19:20:50+04:00 host - - - -")
            .expect("Should parse empty message");
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("1985-04-12T19:20:50+04:00").unwrap())
        );

        let msg = parse_message("<1>1 2003-08-24T05:14:15.000003+07:00 host - - - -")
            .expect("Should parse empty message");
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("2003-08-24T05:14:15.000003+07:00").unwrap())
        );

        let msg = parse_message("<1>1 2003-08-24T05:14:15.000000003+07:00 host - - - -");
        assert!(msg.is_err(), "expected parse fail");
    }

    #[test]
    fn test_empty_sd_value() {
        let msg = parse_message(r#"<29>1 2018-05-14T08:23:01.520Z leyal_test4 mgd 13894 UI_CHILD_EXITED [junos@2636.1.1.1.2.57 pid="14374" return-value="5" core-dump-status="" command="/usr/sbin/mustd"]"#).expect("must parse");
        assert_eq!(msg.facility, Facility::DAEMON);
        assert_eq!(msg.severity, Severity::NOTICE);
        assert_eq!(msg.hostname, Some("leyal_test4"));
        assert_eq!(msg.appname, Some("mgd"));
        assert_eq!(msg.procid, Some(ProcId::PID(13894)));
        assert_eq!(msg.msg, String::from(""));
        assert_eq!(
            msg.timestamp,
            Some(DateTime::parse_from_rfc3339("2018-05-14T08:23:01.520Z").unwrap())
        );
        assert_eq!(msg.structured_data.len(), 1);

        let want = StructuredElement {
            id: "junos@2636.1.1.1.2.57",
            params: vec![
                ("pid", "14374"),
                ("return-value", "5"),
                ("core-dump-status", ""),
                ("command", "/usr/sbin/mustd"),
            ],
        };
        let got = msg.structured_data.first().unwrap();
        assert_eq!(got, &want);
    }

    #[test]
    fn test_fields_start_with_dash() {
        let msg = parse_message("<39>1 2018-05-15T20:56:58+00:00 -web1west -201805020050-bc5d6a47c3-master - - [meta sequenceId=\"28485532\"] 25450-uWSGI worker 6: getaddrinfo*.gaih_getanswer: got type \"DNAME\"").expect("should parse");
        assert_eq!(msg.hostname, Some("-web1west"));
        assert_eq!(msg.appname, Some("-201805020050-bc5d6a47c3-master"));
        let (key, value) = msg
            .structured_data
            .iter()
            .find(|element| element.id == "meta")
            .unwrap()
            .params
            .first()
            .unwrap();
        assert_eq!(*key, "sequenceId");
        assert_eq!(*value, "28485532");
        assert_eq!(
            msg.msg,
            "25450-uWSGI worker 6: getaddrinfo*.gaih_getanswer: got type \"DNAME\""
        );
    }

    #[test]
    fn test_truncated() {
        let err =
            parse_message("<39>1 2018-05-15T20:56:58+00:00 -web1west -").expect_err("should fail");
        assert_eq!(
            mem::discriminant(&err),
            mem::discriminant(&Error::UnexpectedEndOfInput)
        );
    }

    #[test]
    fn test_parse_timestamp() {
        let (ts, rest) = parse_timestamp("2015-02-18T23:16:09Z").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2015, 2, 18, 23, 16, 9)
                .unwrap(),
            ts.unwrap()
        );

        let edt = FixedOffset::east_opt(5 * 60 * 60).unwrap();
        let (ts, rest) = parse_timestamp("2015-02-18T23:59:59.234567+05:00").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            edt.from_local_datetime(
                &NaiveDate::from_ymd_opt(2015, 2, 18)
                    .unwrap()
                    .and_hms_micro_opt(23, 59, 59, 234_567)
                    .unwrap()
            )
            .unwrap(),
            ts.unwrap()
        )
    }
}
