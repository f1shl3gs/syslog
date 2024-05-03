# Syslog

A simple and high-performance syslog parser for [RFC5424](https://tools.ietf.org/html/rfc5424) that aims to parse
syslog messages. The goal is to extract as much correct information from the message rather
than to be pedantically correct to the standard.

## Limitation
Only RFC5424 is supported, if you want parse [RFC3164](https://datatracker.ietf.org/doc/html/rfc3164) contents, please try [syslog_loose](https://docs.rs/syslog_loose/).

## Benchmark
This implementation is very simple, so it is more efficient.

```text
test parse/syslog_loose/with_structured_data ... bench:         412 ns/iter (+/- 12)
test parse/rfc5424/with_structured_data ... bench:         106 ns/iter (+/- 4)
test parse/syslog_loose/with_structured_data_long_msg ... bench:         414 ns/iter (+/- 10)
test parse/rfc5424/with_structured_data_long_msg ... bench:         106 ns/iter (+/- 2)
test parse/syslog_loose/without_structured_data ... bench:         278 ns/iter (+/- 2)
test parse/rfc5424/without_structured_data ... bench:          61 ns/iter (+/- 1)
test parse/syslog_loose/without_structured_data_long_msg ... bench:         277 ns/iter (+/- 2)
test parse/rfc5424/without_structured_data_long_msg ... bench:          60 ns/iter (+/- 0)

```
