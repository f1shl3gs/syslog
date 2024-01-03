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
                        time:   [685.84 ns 686.84 ns 687.87 ns]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
parse/syslog/with_structured_data
                        time:   [316.41 ns 317.09 ns 317.70 ns]
parse/syslog_loose/with_structured_data_long_msg
                        time:   [474.28 ns 475.58 ns 476.99 ns]
parse/syslog/with_structured_data_long_msg
                        time:   [212.63 ns 212.99 ns 213.38 ns]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
parse/syslog_loose/without_structured_data
                        time:   [467.73 ns 469.26 ns 470.90 ns]
parse/syslog/without_structured_data
                        time:   [214.45 ns 214.69 ns 214.99 ns]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
parse/syslog_loose/without_structured_data_long_msg
                        time:   [670.14 ns 671.58 ns 673.21 ns]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
parse/syslog/without_structured_data_long_msg
                        time:   [316.62 ns 317.59 ns 318.74 ns]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
```
