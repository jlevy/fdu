---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 17
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
hold: null
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-07T06:32:02.849Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

Final comparison preregistration before any new timing run: 12 measured interleaved pairs with 3 warmups in each quiet warm-steady cell, fixed N without optional stopping. Subjects are the full stable Rust installation (72,026 entries, naturally no .gitignore, dense) and the live public Metabrowser checkout (97,587 entries, control-rich, dense), with fresh independent fingerprints and per-artifact provenance. Compare final controls-enabled candidate to exact historical b75bf85 for default-tree/cold-scan-index, and to c6380f7 plus matched opened oracle (115ff4f) for default-tree/cold-scan-index/opened-discovery. Current remote main remains b75bf85, checked before the run. Historical scoped-allocation-only backport is 2010fa3; the historical timing binary is untouched. Record all cells, failed/inconclusive ones included, and apply the existing +3%, 1.05 allocation and structural 3%/20% RSS rules unchanged. Profile all five jobs before considering another production optimization. Earlier source/SDK/cache eligibility screens are retained locally; no timings were selected from them.
