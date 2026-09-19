---
type: is
id: is-01m2w9fjprg6vnzee5686g93j6
title: "H113 quiet confirmatory: skip second analysis_candidates walk after H115"
kind: task
status: in_progress
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: unknown@spud10
labels:
  - campaign-2
  - macos-agenda
  - performance
dependencies: []
parent_id: is-01kzysa79temyc45zjn2v98kpw
hold: null
hold_until: null
created_at: 2026-09-19T07:35:50.742Z
updated_at: 2026-09-19T19:59:55.070Z
started_at: 2026-09-19T07:35:59.554Z
---
Quiet confirmatory of H113 after H115 landed. New experiment id (not a top-up of exp-110). Do not retry the file-count shortcut on an uncontrolled cell.

## Pre-registered accept rule (before any engine change)

- Hypothesis: H113. Experiment id if measured: exp-113.
- Claim: with H115 in (one bottom-up roll-up after restore), skipping the second `analysis_candidates` walk on a complete cache-only hit is a real wall win, only if a quiet cell can hold.
- Metric: `content-cache-hit` wall_ns on nominated metabrowser-clone (same deciding subject as exp-108..112).
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero; content digest identical to control; incomplete sidecar still refused (`cache_only_analysis_fails_closed_when_the_sidecar_is_incomplete`).
- Control: current branch HEAD with H115 in (`4370c6c0` / engine `7798fdc1`). Claim-grade pair with `FDU_COUNTERS` unset.
- Regime: `PERF_HOST_REGIME=quiet` only. If the start gate fails (CPU busy > 25%), record the gate failure, update standing (H113 still needs a quiet host), and stop. Do not run another uncontrolled H113. Do not lower the 25% bar.
- Trials: 12 interleaved pairs. No RAM disk.
- Change: same class as exp-110 — `Index::analysis_candidate_count` returns the root regular-file total (`analysis_candidates` capacity); cache-only completeness compares `hits` to that instead of walking. Incomplete-sidecar fail-closed test already kept.
- README: do not raise 200K files/s or 4M cached lines/s from this analyze-hit cell.

## Notes

2026-09-19 stacked PR #92: quiet start gates refused at 69.4% then 43.79% CPU busy. No pair either time. File-count shortcut not compiled on the 43.79% attempt. exp-113 unused. Do not run uncontrolled. Next free experiment after leftover exp-122 is still exp-113 reserved, then exp-123.
