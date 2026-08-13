---
type: is
id: is-01kzxsmcabr3shfgh9644tbdtg
title: "H66: Experiment with an exact directory-only transient tree"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T14:50:14.218Z
updated_at: 2026-08-13T14:58:47.500Z
---
For an unfiltered cache-off tree-only request, scan the complete tree but fold file metadata directly into directory roll-ups and retain only directory topology needed for byte-identical tree output. Do not use the path when a snapshot, files/types/summary composition, selection, or reusable index is required. Compare current indexed-tree and candidate on the 60k and near-million subjects with exact output semantics, paired wall/CPU/RSS/fault evidence, and a dut rendered-tree calibration on Linux. Keep only a material wall improvement or a large memory reduction without a meaningful latency regression.

## Notes

Derived from the 2026-08-13 dut refresh. This is a requirement-derived execution plan, not a CLI fast mode: scan every entry, preserve exact partial/error semantics, retain every directory and exact roll-up required by the tree view, and omit file records only when no requested/current/future consumer can observe them. Gate on byte-identical output and paired wall/CPU/RSS/fault evidence at roughly 60k and near-million scale; use dut only as a Linux rendered-tree calibration.
