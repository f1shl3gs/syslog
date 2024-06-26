fn main() {
    let text = r#"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut="3" eventSource="Application" eventID="1011"] BOMAn application event log entry"#.as_bytes();
    let guard = pprof::ProfilerGuard::new(100).unwrap();

    for _ in 0..100000000 {
        let _ = syslog::rfc5424::parse_message(text);
    }

    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}
