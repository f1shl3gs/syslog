# Syslog

A simple syslog parser for [RFC5424](https://tools.ietf.org/html/rfc5424) that aims to parse
syslog messages. The goal is to extract as much correct information from the message rather
than to be pedantically correct to the standard.

## Limitation
Only RFC5424 is supported, if you want parse RFC3164 contents, please try [syslog_loose](https://docs.rs/syslog_loose/).

## Benchmark
This implementation is very simple, so it is more efficient.

```text
parse/syslog_loose/with_structured_data
                        time:   [660.24 ns 661.48 ns 662.68 ns]
parse/syslog/with_structured_data
                        time:   [289.91 ns 290.64 ns 291.33 ns]
parse/syslog_loose/with_structured_data_long_msg
                        time:   [464.72 ns 468.42 ns 473.15 ns]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
parse/syslog/with_structured_data_long_msg
                        time:   [190.74 ns 191.62 ns 192.54 ns]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
parse/syslog_loose/without_structured_data
                        time:   [456.49 ns 457.94 ns 459.78 ns]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
parse/syslog/without_structured_data
                        time:   [192.59 ns 193.61 ns 194.58 ns]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
parse/syslog_loose/without_structured_data_long_msg
                        time:   [664.48 ns 670.22 ns 675.93 ns]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
parse/syslog/without_structured_data_long_msg
                        time:   [292.21 ns 292.86 ns 293.53 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

```
