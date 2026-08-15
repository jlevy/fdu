---
type: is
id: is-01m01eaac9f07exaqb7erjzf0y
title: Expose adaptive-worker and macOS backend decisions in performance artifacts
kind: bug
status: in_progress
priority: 1
version: 16
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - instrumentation
  - fix
dependencies:
  - type: blocks
    target: is-01m01egyp43zd6yj43cjf1ge1d
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:27.944Z
updated_at: 2026-08-15T02:17:24.896Z
---
Add runtime-gated, low-overhead observability that can prove how automatic scheduling and the macOS backend behaved. Define a bounded, versioned policy trace containing available, initial, and maximum workers; calibration window boundaries and signals; every decision and entry ordinal; active and peak workers; ready and in-flight directory work; handoff-channel backlog/high-water observations where measurable; and macOS bulk versus portable-fallback directory counts. Carry it into perf-probe raw artifacts without changing stable human output. Decide and document whether the trace belongs in the public ScanReport API or a versioned internal probe contract; do not silently add a benchmark-only public API.

Acceptance: trace size is bounded by policy events rather than entries; unavailable observations serialize as null plus a reason and invalidate claims that require them; FDU_COUNTERS aggregates and trace fields cross-check; tests cover every current policy outcome, truncation/bounds, unsupported-platform fields, and fail-closed artifact validation; the incremental overhead of these new fields is measured with the established paired protocol; make check and make cross-lint pass for touched platform-gated code. The current handoff channel is unbounded, so this bead must not claim true consumer backpressure or change queue semantics; any bounded-channel behavior change requires separate evidence and scope.

## Notes

Partial progress on branch codex/epic-fdu-5rpt-adaptive-workers (no bead closed).

Landed: an `adaptive scan policy` counter group in `fdu::counters` recording calibration
chunks, entries, worker microseconds, reserve expansions, and walks left undecided.
Recording is runtime-gated by the existing FDU_COUNTERS toggle and the update happens
outside the DirectoryQueue lock, so a disabled counter cannot lengthen the critical
section it observes. Covered by a library test and by the shipped-binary group assertion
in tests/cli_exit.rs. make check and make cross-lint pass.

NOT done, and this bead must stay open for it:
- No bounded, versioned policy *trace*. The counters are aggregates only; there is no
  per-decision record carrying available/initial/maximum workers, window boundaries,
  entry ordinals, active/peak workers, or ready/in-flight work.
- No macOS bulk versus portable-fallback directory counts.
- Nothing carried into perf-probe raw artifacts.
- The ScanReport-API versus internal-probe-contract decision is undocumented.
- No truncation/bounds, unsupported-platform, or fail-closed artifact validation tests.
- Incremental overhead is unmeasured. That needs the established paired protocol on the
  target regime, which the Linux x86_64/ext4 VM this ran on cannot provide.
