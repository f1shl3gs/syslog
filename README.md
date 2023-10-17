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
                        time:   [673.58 ns 676.44 ns 679.38 ns]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
parse/syslog/with_structured_data
                        time:   [369.50 ns 369.83 ns 370.22 ns]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
parse/syslog_loose/with_structured_data_long_msg
                        time:   [447.76 ns 449.75 ns 451.99 ns]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
parse/syslog/with_structured_data_long_msg
                        time:   [290.58 ns 292.16 ns 294.07 ns]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
parse/syslog_loose/without_structured_data
                        time:   [448.36 ns 450.89 ns 453.84 ns]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
parse/syslog/without_structured_data
                        time:   [280.17 ns 280.82 ns 281.49 ns]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
parse/syslog_loose/without_structured_data_long_msg
                        time:   [665.61 ns 667.14 ns 668.50 ns]
parse/syslog/without_structured_data_long_msg
                        time:   [385.84 ns 387.43 ns 389.40 ns]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high severe
```
