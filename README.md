# Syslog

A simple syslog parser for [RFC5424](https://tools.ietf.org/html/rfc5424) that aims to parse
syslog messages. The goal is to extract as much correct information from the message rather
than to be pedantically correct to the standard.

## Limitation
Only RFC5424 is supported, if you want parse RFC3164 contents, please try [syslog_loose](https://docs.rs/syslog_loose/).

## Benchmark
This implementation is very simple, so it is more efficient.

```text
test parse/syslog_loose/with_structured_data ... bench:         418 ns/iter (+/- 3)
test parse/syslog/with_structured_data ... bench:         184 ns/iter (+/- 4)
test parse/syslog_loose/with_structured_data_long_msg ... bench:         420 ns/iter (+/- 6)
test parse/syslog/with_structured_data_long_msg ... bench:         183 ns/iter (+/- 4)
test parse/syslog_loose/without_structured_data ... bench:         262 ns/iter (+/- 3)
test parse/syslog/without_structured_data ... bench:         120 ns/iter (+/- 1)
test parse/syslog_loose/without_structured_data_long_msg ... bench:         262 ns/iter (+/- 5)
test parse/syslog/without_structured_data_long_msg ... bench:         118 ns/iter (+/- 2)
```
