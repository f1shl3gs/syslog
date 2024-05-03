use chrono::{Duration, FixedOffset, TimeZone};
use syslog::rfc5424::parse_message;
use syslog::{Facility, Message, Protocol, Severity, StructuredElement};

#[test]
fn parse_5424_no_structured_data() {
    let input = "<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - BOM'su root' failed for lonvick on /dev/pts/8";

    assert_eq!(
        parse_message(input.as_bytes()).unwrap(),
        Message {
            facility: Facility::AUTH,
            severity: Severity::CRIT,
            protocol: Protocol::RFC5424(1),
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2003, 10, 11, 22, 14, 15)
                    .unwrap()
                    + Duration::milliseconds(3)
            ),
            hostname: Some("mymachine.example.com"),
            appname: Some("su"),
            procid: None,
            msgid: Some("ID47"),
            structured_data: vec![],
            msg: "BOM'su root' failed for lonvick on /dev/pts/8",
        }
    );
}

#[test]
fn parse_5424_structured_data() {
    let msg = "<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] BOMAn application event log entry...";

    assert_eq!(
        parse_message(msg.as_bytes()).unwrap(),
        Message {
            facility: Facility::LOCAL4,
            severity: Severity::NOTICE,
            protocol: Protocol::RFC5424(1),
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2003, 10, 11, 22, 14, 15)
                    .unwrap()
                    + Duration::milliseconds(3)
            ),
            hostname: Some("mymachine.example.com"),
            appname: Some("evntslog"),
            procid: None,
            msgid: Some("ID47"),
            structured_data: vec![StructuredElement {
                id: "exampleSDID@32473",
                params: vec![
                    ("iut", "3"),
                    ("eventSource", "Application"),
                    ("eventID", "1011")
                ]
            },],
            msg: "BOMAn application event log entry...",
        }
    );
}

#[test]
fn parse_5424_empty_structured_data() {
    let msg = "<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"\" eventID=\"1011\"] BOMAn application event log entry...";

    assert_eq!(
        parse_message(msg.as_bytes()).unwrap(),
        Message {
            facility: Facility::LOCAL4,
            severity: Severity::NOTICE,
            protocol: Protocol::RFC5424(1),
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2003, 10, 11, 22, 14, 15)
                    .unwrap()
                    + Duration::milliseconds(3)
            ),
            hostname: Some("mymachine.example.com"),
            appname: Some("evntslog"),
            procid: None,
            msgid: Some("ID47"),
            structured_data: vec![StructuredElement {
                id: "exampleSDID@32473",
                params: vec![("iut", "3"), ("eventSource", ""), ("eventID", "1011")]
            },],
            msg: "BOMAn application event log entry...",
        }
    );
}

#[test]
fn parse_5424_multiple_structured_data() {
    let msg = "<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"][examplePriority@32473 class=\"high\"] BOMAn application event log entry...";

    assert_eq!(
        parse_message(msg.as_bytes()).unwrap(),
        Message {
            facility: Facility::LOCAL4,
            severity: Severity::NOTICE,
            protocol: Protocol::RFC5424(1),
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2003, 10, 11, 22, 14, 15)
                    .unwrap()
                    + Duration::milliseconds(3)
            ),
            hostname: Some("mymachine.example.com"),
            appname: Some("evntslog"),
            procid: None,
            msgid: Some("ID47"),
            structured_data: vec![
                StructuredElement {
                    id: "exampleSDID@32473",
                    params: vec![
                        ("iut", "3"),
                        ("eventSource", "Application"),
                        ("eventID", "1011")
                    ]
                },
                StructuredElement {
                    id: "examplePriority@32473",
                    params: vec![("class", "high"),]
                }
            ],
            msg: "BOMAn application event log entry...",
        }
    );
}
