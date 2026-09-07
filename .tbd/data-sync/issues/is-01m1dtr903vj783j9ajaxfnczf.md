---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 21
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies:
  - type: blocks
    target: is-01m1x444q4jz0680n8a057r5z8
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
child_order_hints:
  - is-01m1edc4xady6k86e0hsbzfsk1
  - is-01m1eek06tcb89yygyc1xz2yz5
  - is-01m1egf3aa4wt4kc2z5qmhspqp
  - is-01m1egxbrdj757jr3bk8bhv1ce
  - is-01m1ejqfv4khft8mbkfw7f3q0f
  - is-01m1ekg6ewkj2mr9wf1xs9g01y
  - is-01m1xahn7a7m85y4xqd76xk3x4
hold: null
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-07T07:39:30.422Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

64c6e61 is pushed in formal stack #53 and passes full isolated make check, cross-lint and all 19 CI checks. Scope/profile fixes fdu-ht5q and fdu-ttpf are closed. Corrected candidate 64c6e61, unchanged historical b75, allocation-only 2010fa3 and structural control 1981747 verify in claim-grade manifest bfa3f00322999b3f3d73718121738e1b641f6ab42195059ff17d66134444aff5. Scoped allocation checks on both nominated subjects match all compared summary fields/digests. Default allocation ratios versus historical are 0.6463 Rust and 0.6674 source; byte ratios 0.9472 and 0.9146. Cold and opened allocation/reallocation/byte ceilings also pass this instrumented check, not a timing verdict. Timing remains unsampled: two original attempts refused before trials; a later read-only pressure snapshot was 53.97% busy. Another project workload and indexing are active; neither was interrupted. fdu-0q6w owns the confirmed public preflight hotspot before final-binary timing. Preserve fixed 12 pairs, 3 warmups and all original final quiet-host/CI thresholds.
