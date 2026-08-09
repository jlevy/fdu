---
type: is
id: is-01kzg4c6h9v2dzand7t090p278
title: Publish Phase 1 performance matrix and dut/gdu evidence report
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01kzg4d256qmchmtyvttnpvn4y
  - type: blocks
    target: is-01kzg4d2saym31t884vf6me2p7
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-08T07:28:38.441Z
updated_at: 2026-08-09T20:37:10.563Z
---
Execute and publish the complete Phase 1 performance evidence after engine and harness blockers land. Run the validated 10k/100k/500k/1M scale and state matrix; report scan-producer versus scan-index, snapshot/open/revalidation, bounded human versus complete JSON, memory, parallelism, and pinned dut/gdu adapters. Use at least ten valid interleaved trials for headline scenarios, preserve every raw sample and capability caveat, satisfy or explicitly revise the cold full-stat, warm 500k, memory, and snapshot UX gates, generate the reviewed report, and allow README claims only when they link to its reproduction manifest. Generic hosted CI results cannot satisfy this bead.
