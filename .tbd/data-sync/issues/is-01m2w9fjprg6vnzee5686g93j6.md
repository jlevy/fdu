---
type: is
id: is-01m2w9fjprg6vnzee5686g93j6
title: "H113 quiet confirmatory: skip second analysis_candidates walk after H115"
kind: task
status: open
priority: 1
version: 3
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
updated_at: 2026-09-19T07:41:31.254Z
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

Pre-registered 2026-09-19 before any engine change.

Metric: content-cache-hit wall_ns on nominated metabrowser-clone.
Direction: down.
Accept: median at least 3% faster and 95% paired interval entirely below zero;
content digest identical; incomplete sidecar still refused.
Control: branch HEAD with H115 in (4370c6c0 / engine 7798fdc1). FDU_COUNTERS unset.
Regime: PERF_HOST_REGIME=quiet only. 12 interleaved pairs. No RAM disk.
Change: same class as exp-110 — analysis_candidate_count vs walking analysis_candidates.

2026-09-19 official quiet start gate refused: CPU busy 46.7% > 25.0%.
No pair ran. Engine patch reverted unmeasured. exp-113 not consumed.
H113 still needs a quiet host. Do not retry uncontrolled.
