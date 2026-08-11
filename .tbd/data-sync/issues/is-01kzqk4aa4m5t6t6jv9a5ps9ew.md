---
type: is
id: is-01kzqk4aa4m5t6t6jv9a5ps9ew
title: "PR #3 review R9: report two-sided significance and valid process metrics"
kind: bug
status: closed
priority: 2
version: 4
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzqk4awe65tvmt9c6ynnb5zc
  - type: blocks
    target: is-01kzqk493tkcy6nwws6vf9md7f
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:09.828Z
updated_at: 2026-08-11T05:34:06.441Z
closed_at: 2026-08-11T05:34:06.440Z
close_reason: Statistics now record direction, CI-excludes-zero, and significant-improvement separately; render clear regressions as regressions; null blocked_ns for parallel process-CPU jobs; omit invalid historical blocked estimates; and require separate CPU/RSS guardrails or an explicit waiver for accepted latency wins. Schema, ledger, and tests updated.
---
FDU-PR3-R9. benchmarks/realtree/measure.py and summary.py. The current significant flag recognizes only improvements, labels clear regressions n.s., and interprets wall minus summed multithread CPU as blocked time. Add ci_excludes_zero and direction, preserve a separate acceptance signal, render regressions, null unsupported blocked time, and make resource tradeoffs explicit with tests. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
