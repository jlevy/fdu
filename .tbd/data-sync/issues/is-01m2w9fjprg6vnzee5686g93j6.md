---
type: is
id: is-01m2w9fjprg6vnzee5686g93j6
title: "H113 quiet confirmatory: skip second analysis_candidates walk after H115"
kind: task
status: open
priority: 1
version: 10
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
updated_at: 2026-09-19T16:28:54.789Z
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

Quiet confirmatory of H113 after H115 landed. New experiment id (not a top-up of exp-110). Do not retry the file-count shortcut on an uncontrolled cell.

## Pre-registered accept rule (before any engine change)

- Hypothesis: H113. Experiment id if measured: exp-113.
- Claim: with H115 in (one bottom-up roll-up after restore), skipping the second analysis_candidates walk on a complete cache-only hit is a real wall win, only if a quiet cell can hold.
- Metric: content-cache-hit wall_ns on nominated metabrowser-clone (same deciding subject as exp-108..117).
- Direction: down.
- Accept: median at least 3% faster and the 95% paired interval entirely below zero; content digest identical to control; incomplete sidecar still refused.
- Control: current branch HEAD with H115 and H120 in. Claim-grade pair with FDU_COUNTERS unset.
- Regime: PERF_HOST_REGIME=quiet only. If the start gate fails (CPU busy > 25%), record the gate failure, skip to H122, and stop. Do not run another uncontrolled H113. Do not lower the 25% bar.
- Trials: 12 interleaved pairs. No RAM disk.
- Change: same class as exp-110. Cache-only completeness compares hits to a known file count instead of walking analysis_candidates.
- README: do not raise 200K files/s or 4M cached lines/s from this analyze-hit cell.

## Notes

Still open. Needs quiet host. Morning is the intended cell. Do not run uncontrolled.

2026-09-19 overnight: start gate refused at 46.7% busy. Later incomplete quiet cells (thermal fair = 0 pairs; then 3 pairs / 16 invalid; then 9 pairs / 4 invalid) are not a verdict. File-count shortcut reverted; not in the engine. exp-113 reserved.
