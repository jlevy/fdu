# Linux Tree-Tier Floor Scoreboard (2026-10-10)

Machine dump from `benchmarks.realtree.floor` for `fdu-088k` (H193): the tier-by-floor
scoreboard re-derived after the age column made the default tree describe every
directory and symlink again, with the `tree` tier this branch adds.
Judgment is in
[exp-217](../experiments/exp-217-linux-h193-the-age-column-costs-the-dense-tree-s-default-com.md).
Do not quote these milliseconds as a product claim.

The harness ran from `843bf2ab` (this branch); the probe scored is the 0.5.0 stack’s
head, `0d6011da` (sha256 `d93d5a43`).

Host: Linux x86_64, 4 logical CPUs.
Every instrument ran a fixed pool of 4 workers.
Recorded 2026-10-10T18:08:14Z from commit 843bf2ab.

Regime: **uncontrolled** (quiet was requested, and 1 measured trial breached it:
quiet-host CPU busy exceeded 25.0% after the sample; this table is screening-grade).
30 trials, 3 warmups, interleaved.

## node-modules-dense — 79,957 entries (79,849 directories and files)

| Instrument | Role | Median | ×floor | ns/(dir+file) | spread | p95/median | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 37.66 ms | **0.39** | 472 | 1.58 | 1.201 | 26 MiB |
| `parfloor-stat` | floor | 97.26 ms | **1.00** | 1218 | 1.36 | 1.167 | 26 MiB |
| `aggregate` | tier | 110.75 ms | **1.14** ?≤1.25 | 1387 | 1.27 | 1.1 | 26 MiB |
| `tree` | tier | 119.31 ms | **1.23** | 1494 | 1.41 | 1.186 | 26 MiB |
| `index` | tier | 130.97 ms | **1.35** ?≤1.4 | 1640 | 1.66 | 1.239 | 31 MiB |
| `arena-spike` | ceiling | 357.15 ms | **3.67** | 4473 | 3.75⚠ | 1.215 | 26 MiB |

⚠ max/min at or past 2×: the samples span more than the median can stand for -- a second
mode or an outlier; the spread cannot say which.
Read that row as a range, and a tier marked ? as neither closed nor open, or re-run
where the cause can be separated.

## linux-v6.12 — 92,474 entries (92,411 directories and files)

| Instrument | Role | Median | ×floor | ns/(dir+file) | spread | p95/median | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 24.20 ms | **0.26** | 262 | 1.48 | 1.167 | 26 MiB |
| `parfloor-stat` | floor | 91.58 ms | **1.00** | 991 | 1.33 | 1.166 | 26 MiB |
| `aggregate` | tier | 103.53 ms | **1.13** ?≤1.25 | 1120 | 1.26 | 1.087 | 26 MiB |
| `tree` | tier | 115.80 ms | **1.26** | 1253 | 1.30 | 1.117 | 26 MiB |
| `index` | tier | 128.21 ms | **1.40** ?≤1.4 | 1387 | 1.40 | 1.171 | 31 MiB |
| `arena-spike` | ceiling | 197.90 ms | **2.16** | 2142 | 1.87 | 1.132 | 26 MiB |

## linux-balanced-1m — 1,000,001 entries (1,000,000 directories and files)

| Instrument | Role | Median | ×floor | ns/(dir+file) | spread | p95/median | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 528.79 ms | **0.47** | 529 | 1.52 | 1.135 | 26 MiB |
| `aggregate` | tier | 1104.35 ms | **0.97** ?≤1.25 | 1104 | 1.30 | 1.131 | 26 MiB |
| `parfloor-stat` | floor | 1133.08 ms | **1.00** | 1133 | 1.28 | 1.074 | 26 MiB |
| `arena-spike` | ceiling | 1136.00 ms | **1.00** | 1136 | 3.83⚠ | 2.245 | 66 MiB |
| `tree` | tier | 1371.19 ms | **1.21** | 1371 | 1.29 | 1.079 | 68 MiB |
| `index` | tier | 1416.42 ms | **1.25** ?≤1.4 | 1416 | 1.37 | 1.198 | 306 MiB |

⚠ max/min at or past 2×: the samples span more than the median can stand for -- a second
mode or an outlier; the spread cannot say which.
Read that row as a range, and a tier marked ? as neither closed nor open, or re-run
where the cause can be separated.
