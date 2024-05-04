# Syslog

A simple and high-performance syslog parser for [RFC5424](https://tools.ietf.org/html/rfc5424) that aims to parse
syslog messages. The goal is to extract as much correct information from the message rather
than to be pedantically correct to the standard.

## Limitation
Only RFC5424 is supported, if you want parse [RFC3164](https://datatracker.ietf.org/doc/html/rfc3164) contents, please try [syslog_loose](https://docs.rs/syslog_loose/).

## Benchmark
This implementation is very simple, so it is more efficient.

```text
test parse/syslog_loose/with_structured_data ... bench:         414 ns/iter (+/- 4)
test parse/rfc5424/with_structured_data ... bench:          94 ns/iter (+/- 0)
test parse/syslog_loose/with_structured_data_long_msg ... bench:         407 ns/iter (+/- 4)
test parse/rfc5424/with_structured_data_long_msg ... bench:          94 ns/iter (+/- 0)
test parse/syslog_loose/without_structured_data ... bench:         262 ns/iter (+/- 3)
test parse/rfc5424/without_structured_data ... bench:          60 ns/iter (+/- 2)
test parse/syslog_loose/without_structured_data_long_msg ... bench:         260 ns/iter (+/- 3)
test parse/rfc5424/without_structured_data_long_msg ... bench:          60 ns/iter (+/- 0)

```
