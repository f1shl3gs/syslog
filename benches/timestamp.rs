use chrono::{FixedOffset, NaiveDate, TimeZone};
use criterion::{criterion_group, criterion_main, Criterion};

fn parse_timestamp(c: &mut Criterion) {
    let input = r#"2023-04-07T12:52:00.654321Z"#;
    let mut group = c.benchmark_group("parse");

    group.bench_function("own", |b| {
        b.iter(|| {
            let _ = syslog::parse_timestamp(input.as_bytes());
        })
    });

    group.bench_function("chrono", |b| {
        b.iter(|| {
            let _ = chrono::DateTime::parse_from_rfc3339(input);
        })
    });

    group.bench_function("eeep", |b| {
        b.iter(|| {
            let _ = eeep::parse_from_timestamp_datetime(input);
        })
    });

    group.bench_function("speedate", |b| {
        b.iter(|| {
            let datetime = speedate::DateTime::parse_str_rfc3339(input).unwrap();
            let naive = NaiveDate::from_ymd_opt(
                datetime.date.year as i32,
                datetime.date.month as u32,
                datetime.date.day as u32,
            )
            .unwrap()
            .and_hms_nano_opt(
                datetime.time.hour as u32,
                datetime.time.minute as u32,
                datetime.time.second as u32,
                datetime.time.microsecond,
            )
            .unwrap();
            let offset = FixedOffset::east_opt(datetime.time.tz_offset.unwrap_or(0)).unwrap();
            let _ = offset.from_local_datetime(&naive);
        })
    });

    group.finish();
}

criterion_group!(benches, parse_timestamp);
criterion_main!(benches);
