# Linux H111 Floor Scoreboard (2026-09-20)

Machine dump from `benchmarks.realtree.floor` for exp-141 / H111. Judgment is in
[exp-141](../experiments/exp-141-h111-linux-floor-and-rss-gates-fail-on-current-engine.md).
Do not quote these milliseconds as a product claim.

Host: Linux x86_64, 4 logical CPUs.
Every instrument ran a fixed pool of 4 workers.
Recorded 2026-09-20T01:29:34Z from commit bf260c74.

Regime: **uncontrolled** (quiet was requested, and 121 measured trials breached it:
quiet-host load/core exceeded 0.250 after the sample; quiet-host load/core exceeded
0.250 before the sample; this table is screening-grade).
30 trials, 3 warmups, interleaved.

## linux-450k — 450,001 entries (450,000 directories and files)

| Instrument | Role | Median | ×floor | ns/(dir+file) | spread | p95/median | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 88.91 ms | **0.49** | 198 | 1.10 | 1.049 | 25 MiB |
| `parfloor-stat` | floor | 180.92 ms | **1.00** | 402 | 1.13 | 1.05 | 25 MiB |
| `arena-spike` | ceiling | 195.40 ms | **1.08** | 434 | 1.11 | 1.049 | 30 MiB |
| `aggregate` | tier | 263.84 ms | **1.46** ✗>1.25 | 586 | 1.17 | 1.085 | 25 MiB |
| `index` | tier | 321.76 ms | **1.78** ✗>1.4 | 715 | 1.06 | 1.018 | 158 MiB |

## linux-v6.12 — 92,474 entries (92,411 directories and files)

| Instrument | Role | Median | ×floor | ns/(dir+file) | spread | p95/median | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 9.76 ms | **0.43** | 106 | 1.20 | 1.113 | 25 MiB |
| `parfloor-stat` | floor | 22.86 ms | **1.00** | 247 | 1.51 | 1.286 | 25 MiB |
| `aggregate` | tier | 36.24 ms | **1.59** ✗>1.25 | 392 | 1.31 | 1.188 | 25 MiB |
| `arena-spike` | ceiling | 41.90 ms | **1.83** | 453 | 2.65⚠ | 1.399 | 25 MiB |
| `index` | tier | 423.23 ms | **18.52** ✗>1.4 | 4580 | 1.10 | 1.062 | 36 MiB |

⚠ max/min at or past 2×: the samples span more than the median can stand for -- a second
mode or an outlier; the spread cannot say which.
Read that row as a range, and a tier marked ? as neither closed nor open, or re-run
where the cause can be separated.

## usr-prefix — 198,150 entries (166,503 directories and files)

| Instrument | Role | Median | ×floor | ns/(dir+file) | spread | p95/median | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 22.16 ms | **0.38** | 133 | 1.16 | 1.077 | 25 MiB |
| `parfloor-stat` | floor | 58.52 ms | **1.00** | 352 | 1.27 | 1.083 | 25 MiB |
| `aggregate` | tier | 108.67 ms | **1.86** ✗>1.25 | 653 | 1.40 | 1.083 | 25 MiB |
| `arena-spike` | ceiling | 215.85 ms | **3.69** | 1296 | 1.23 | 1.099 | 25 MiB |
| `index` | tier | 313.31 ms | **5.35** ✗>1.4 | 1882 | 1.23 | 1.078 | 65 MiB |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
