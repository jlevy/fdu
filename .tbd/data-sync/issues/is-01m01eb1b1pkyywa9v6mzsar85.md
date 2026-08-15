---
type: is
id: is-01m01eb1b1pkyywa9v6mzsar85
title: Model heterogeneous completion order in adaptive-scheduler tests
kind: bug
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - testing
  - fix
dependencies:
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:51.456Z
updated_at: 2026-08-15T02:17:37.613Z
---
Build a deterministic pure controller/scheduler model that injects identical total work in different completion sequences, including slow in-flight censorship, fast-prefix/slow-suffix, slow-prefix/fast-suffix, alternating phases, narrow frontiers, late activation, and delayed handoff consumption. Use it to encode the current one-shot policy’s false-negative and false-positive signatures, then test candidate policies against behavioral invariants without wall-clock sleeps or assumptions about filesystem enumeration order.

Acceptance: the normal suite passes while a legacy-policy fixture explicitly demonstrates the observed violation rather than leaving a failing test; candidate tests assert bounded workers, liveness, traversal/exactness independence, no late scale-up without useful ready work, bounded reaction to reordered completions, and stable shutdown/disconnect behavior; the model’s cases trace back to the profile and remain portable across platforms.

## Notes

Partial progress on branch codex/epic-fdu-5rpt-adaptive-workers (bead stays open).

Landed: `scan::tests::completion_order`, a deterministic completion-order model that
drives the shipped WorkerCalibration directly rather than restating its arithmetic, so
the model cannot drift from the policy it characterizes. No wall-clock sleeps and no
assumption about filesystem enumeration order; it is portable and runs in CI everywhere.

It encodes the legacy policy's violation as a passing fixture, not a failing test:
- completion order alone flips the decision. One tree, same multiset of chunks, 46 us
  per entry either way, and the shipped policy holds the pool in one order and scales up
  in the other. Both walks are latency-bound by its own 30 us threshold, so it answers
  one of them wrongly by its own criterion.
- a slow phase after the window closes is never reconsidered: 1% of the walk decides the
  policy for the other 99% at 89 us per entry.
- a walk shorter than the window is Undecided, distinct from Held.
A trailing-window candidate is screened in the same module and kept out of the shipped
walker, so no rejected prototype ships.

NOT done, and this bead must stay open for it:
- Missing scenarios: slow in-flight censorship, slow-prefix/fast-suffix, narrow
  frontiers, late activation, delayed handoff consumption.
- Candidate invariants not asserted: bounded workers, liveness, traversal/exactness
  independence, no late scale-up without useful ready work, bounded reaction to
  reordered completions, stable shutdown/disconnect.
- Acceptance requires the model's cases trace back to the profile. fdu-ileg is the
  profile and it is blocked on Apple Silicon hardware, so that link cannot be made yet
  and the scenario list above is derived analytically from the field report instead.
