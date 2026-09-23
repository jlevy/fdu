---
type: is
id: is-01m36bm4k9grwp72hthb58bq0b
title: Measure the composed correctness stack against main before merge
kind: task
status: open
priority: 1
version: 2
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:41.608Z
updated_at: 2026-09-23T05:25:50.842Z
---
The alpha correctness plan makes no performance claim, and nobody has measured #117 against main, although #113 adds status/provenance computation and #115 reroutes execution.rs/scan.rs. Earlier stacks used the gate fdu PATH within 10% of main on a control-free and a control-rich tree (2026-09-14 decision). Run make perf-compare, interleaved and paired, on a real tree for fdu PATH, --view summary, a warm open and --format json; record with make perf-record. The README headline (fdu-y5xr, dumac +11.3%) must be re-measured again on the final release candidate after #94/#97/#105 compose.
