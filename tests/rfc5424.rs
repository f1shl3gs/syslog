use chrono::{Duration, FixedOffset, TimeZone};
use syslog::rfc5424::parse_message;
use syslog::{Facility, Message, ProcId, Protocol, Severity, StructuredElement};

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

#[test]
fn syslog_ng_network_syslog_protocol() {
    let msg = "i am foobar";
    let raw = format!(
        r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - {}{} {}"#,
        r#"[meta sequenceId="1" sysUpTime="37" language="EN"]"#,
        r#"[origin ip="192.168.0.1" software="test"]"#,
        msg
    );

    assert_eq!(
        parse_message(raw.as_bytes()).unwrap(),
        Message {
            facility: Facility::USER,
            severity: Severity::NOTICE,
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2019, 2, 13, 19, 48, 34)
                    .unwrap()
            ),
            hostname: Some("74794bfb6795"),
            appname: Some("root"),
            procid: Some(ProcId::PID(8449)),
            msgid: None,
            protocol: Protocol::RFC5424(1),
            structured_data: vec![
                StructuredElement {
                    id: "meta",
                    params: vec![("sequenceId", "1"), ("sysUpTime", "37"), ("language", "EN")]
                },
                StructuredElement {
                    id: "origin",
                    params: vec![("ip", "192.168.0.1"), ("software", "test"),]
                }
            ],
            msg: "i am foobar",
        }
    )
}

#[ignore]
#[test]
fn handles_incorrect_sd_element() {
    let msg = format!(
        r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - {} qwerty"#,
        r#"[incorrect x]"#
    );

    let should = Message {
        facility: Facility::USER,
        severity: Severity::NOTICE,
        timestamp: Some(
            FixedOffset::west_opt(0)
                .unwrap()
                .with_ymd_and_hms(2019, 2, 13, 19, 48, 34)
                .unwrap(),
        ),
        hostname: Some("74794bfb6795"),
        appname: Some("root"),
        procid: Some(ProcId::PID(8449)),
        msgid: None,
        protocol: Protocol::RFC5424(1),
        structured_data: vec![],
        msg: "qwerty",
    };

    assert_eq!(parse_message(msg.as_bytes()).unwrap(), should);

    let msg = format!(
        r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - {} qwerty"#,
        r#"[incorrect x=]"#
    );

    assert_eq!(parse_message(msg.as_bytes()).unwrap(), should);
}

#[test]
fn handles_empty_sd_element() {
    let msg = format!(
        r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - {} qwerty"#,
        r#"[empty]"#
    );

    assert_eq!(
        parse_message(msg.as_bytes()).unwrap(),
        Message {
            facility: Facility::USER,
            severity: Severity::NOTICE,
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2019, 2, 13, 19, 48, 34)
                    .unwrap()
            ),
            hostname: Some("74794bfb6795"),
            appname: Some("root"),
            procid: Some(ProcId::PID(8449)),
            msgid: None,
            protocol: Protocol::RFC5424(1),
            structured_data: vec![StructuredElement {
                id: "empty",
                params: vec![]
            }],
            msg: "qwerty",
        }
    );

    let msg = format!(
        r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - {} qwerty"#,
        r#"[non_empty x="1"][empty]"#
    );

    assert_eq!(
        parse_message(msg.as_bytes()).unwrap(),
        Message {
            facility: Facility::USER,
            severity: Severity::NOTICE,
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2019, 2, 13, 19, 48, 34)
                    .unwrap()
            ),
            hostname: Some("74794bfb6795"),
            appname: Some("root"),
            procid: Some(ProcId::PID(8449)),
            msgid: None,
            protocol: Protocol::RFC5424(1),
            structured_data: vec![
                StructuredElement {
                    id: "non_empty",
                    params: vec![("x", "1")]
                },
                StructuredElement {
                    id: "empty",
                    params: vec![]
                },
            ],
            msg: "qwerty",
        }
    );

    let msg = format!(
        r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - {} qwerty"#,
        r#"[empty][non_empty x="1"]"#
    );

    assert_eq!(
        parse_message(msg.as_bytes()).unwrap(),
        Message {
            facility: Facility::USER,
            severity: Severity::NOTICE,
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2019, 2, 13, 19, 48, 34)
                    .unwrap()
            ),
            hostname: Some("74794bfb6795"),
            appname: Some("root"),
            procid: Some(ProcId::PID(8449)),
            msgid: None,
            protocol: Protocol::RFC5424(1),
            structured_data: vec![
                StructuredElement {
                    id: "empty",
                    params: vec![]
                },
                StructuredElement {
                    id: "non_empty",
                    params: vec![("x", "1")]
                },
            ],
            msg: "qwerty",
        }
    );

    let msg = format!(
        r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - {} qwerty"#,
        r#"[empty not_really="testing the test"]"#
    );

    assert_eq!(
        parse_message(msg.as_bytes()).unwrap(),
        Message {
            facility: Facility::USER,
            severity: Severity::NOTICE,
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2019, 2, 13, 19, 48, 34)
                    .unwrap()
            ),
            hostname: Some("74794bfb6795"),
            appname: Some("root"),
            procid: Some(ProcId::PID(8449)),
            msgid: None,
            protocol: Protocol::RFC5424(1),
            structured_data: vec![StructuredElement {
                id: "empty",
                params: vec![("not_really", "testing the test")]
            },],
            msg: "qwerty",
        }
    );
}

#[ignore]
#[test]
fn handles_weird_whitespace() {
    // this should also match rsyslog omfwd with template=RSYSLOG_SyslogProtocol23Format
    let raw = r#"
           <13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - [meta sequenceId="1"] i am foobar
            "#;
    let cleaned = r#"<13>1 2019-02-13T19:48:34+00:00 74794bfb6795 root 8449 - [meta sequenceId="1"] i am foobar"#;

    assert_eq!(
        parse_message(raw.as_bytes()).unwrap(),
        parse_message(cleaned.as_bytes()).unwrap()
    );
}

#[test]
fn logical_system_juniper_routers() {
    let raw = r#"<28>1 2020-05-22T14:59:09.250-03:00 OX-XXX-MX204 OX-XXX-CONTEUDO:rpd 6589 - - bgp_listen_accept: %DAEMON-4: Connection attempt from unconfigured neighbor: 2001:XXX::219:166+57284"#;

    assert_eq!(
        parse_message(raw.as_bytes()).unwrap(),
        Message {
            facility: Facility::DAEMON,
            severity: Severity::WARNING,
            timestamp: Some(
                FixedOffset::west_opt(1800 * 6).unwrap()
                    .with_ymd_and_hms(2020, 5, 22,14, 59, 9).unwrap() + Duration::microseconds(250000)
            ),
            hostname: Some("OX-XXX-MX204"),
            appname: Some("OX-XXX-CONTEUDO:rpd"),
            procid: Some(ProcId::PID(6589)),
            msgid: None,
            protocol: Protocol::RFC5424(1),
            structured_data: vec![],
            msg: "bgp_listen_accept: %DAEMON-4: Connection attempt from unconfigured neighbor: 2001:XXX::219:166+57284",
        }
    );
}

#[test]
fn parse_ipv4_hostname() {
    let msg = "<34>1 2003-10-11T22:14:15.003Z 42.52.1.1 su - ID47 - bananas and peas";
    assert_eq!(
        Message {
            facility: Facility::AUTH,
            severity: Severity::CRIT,
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2003, 10, 11, 22, 14, 15)
                    .unwrap()
                    + Duration::milliseconds(3)
            ),
            hostname: Some("42.52.1.1"),
            appname: Some("su"),
            procid: None,
            msgid: Some("ID47"),
            protocol: Protocol::RFC5424(1),
            structured_data: vec![],
            msg: "bananas and peas",
        },
        parse_message(msg.as_bytes()).unwrap()
    )
}

#[test]
fn parse_ipv6_hostname() {
    let msg = "<34>1 2003-10-11T22:14:15.003Z ::FFFF:129.144.52.38 su - ID47 - bananas and peas";
    assert_eq!(
        Message {
            facility: Facility::AUTH,
            severity: Severity::CRIT,
            timestamp: Some(
                FixedOffset::west_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2003, 10, 11, 22, 14, 15)
                    .unwrap()
                    + Duration::milliseconds(3)
            ),
            hostname: Some("::FFFF:129.144.52.38"),
            appname: Some("su"),
            procid: None,
            msgid: Some("ID47"),
            protocol: Protocol::RFC5424(1),
            structured_data: vec![],
            msg: "bananas and peas",
        },
        parse_message(msg.as_bytes()).unwrap()
    )
}
