---
type: is
id: is-01kzxsmcabr3shfgh9644tbdtg
title: "H66: Experiment with an exact directory-only transient tree"
kind: task
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T14:50:14.218Z
updated_at: 2026-08-23T01:52:08.484Z
---
For an unfiltered cache-off tree-only request, scan the complete tree but fold file metadata directly into directory roll-ups and retain only directory topology needed for byte-identical tree output. Do not use the path when a snapshot, files/types/summary composition, selection, or reusable index is required. Compare current indexed-tree and candidate on the 60k and near-million subjects with exact output semantics, paired wall/CPU/RSS/fault evidence, and a dut rendered-tree calibration on Linux. Keep only a material wall improvement or a large memory reduction without a meaningful latency regression.

## Notes

Campaign 2: re-screen after H86 (Phase B) rather than running first. arena_spike holds
index-shaped records at 1.06x the floor, so the additional headroom of retaining only
directory topology may be under the gate once the representation lands; if it survives,
it becomes a retention flag on the new layout rather than a separate plan.
