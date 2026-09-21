---
type: is
id: is-01m2zpkzxm97et6bdxeac7eh6w
title: Performance evidence report prints a peak_rss_bytes primary metric as milliseconds
kind: bug
status: closed
priority: 3
version: 5
labels: []
dependencies: []
parent_id: is-01m31hc3p7wv76jeq5dhgv3bd8
created_at: 2026-09-20T15:23:10.124Z
updated_at: 2026-09-21T08:30:59.685Z
closed_at: 2026-09-21T08:20:31.402Z
close_reason: "Shipped on #104 (https://github.com/jlevy/fdu/pull/104) at 743d2b9c / 4d78558e vs main a290aedc. CI run 35576573158 green on ubuntu/macos/windows including Performance evidence. exp-116 change_pct is the paired −2.158%; commit quoted as 984e4618; digest/regime/errata corrected; ledger+report regenerated. R3 name negatives, valid-rename load control, and H138 sharing guard restored; apply timer starts at candidates.remove. peak_rss_bytes prints as bytes (exp-117 377.5→339.4 MiB). Local rust-test passed except the known parallel_equivalence flake (not fixed)."
---
From the independent pre-merge verification of PR #91 (2026-09-20). Pre-existing generator bug, first exposed by exp-117, the first accept whose primary metric is `peak_rss_bytes`.

`docs/project/reports/performance-evidence/index.html` prints the exp-117 row as `content-cache-hit | 396 ms | 356 ms | -10.1% | kept`. The values are 395,886,592 -> 355,868,672 bytes of peak RSS (377.5 -> 339.4 MiB); the real wall time for that cell is 1,111 -> 1,103 ms. The report generator formats the primary metric as milliseconds regardless of `primary_metric`. Format by metric unit, add a generator test with a bytes-metric record, then `make perf-report`.

## Notes

Context posted for Linux handoff on PR #94: https://github.com/jlevy/fdu/pull/94#issuecomment-5757234985
