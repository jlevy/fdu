---
type: is
id: is-01m01eaac9f07exaqb7erjzf0y
title: Expose adaptive-worker and macOS backend decisions in performance artifacts
kind: bug
status: open
priority: 1
version: 15
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
updated_at: 2026-08-15T01:17:12.009Z
---
Add runtime-gated, low-overhead observability that can prove how automatic scheduling and the macOS backend behaved. Define a bounded, versioned policy trace containing available, initial, and maximum workers; calibration window boundaries and signals; every decision and entry ordinal; active and peak workers; ready and in-flight directory work; handoff-channel backlog/high-water observations where measurable; and macOS bulk versus portable-fallback directory counts. Carry it into perf-probe raw artifacts without changing stable human output. Decide and document whether the trace belongs in the public ScanReport API or a versioned internal probe contract; do not silently add a benchmark-only public API.

Acceptance: trace size is bounded by policy events rather than entries; unavailable observations serialize as null plus a reason and invalidate claims that require them; FDU_COUNTERS aggregates and trace fields cross-check; tests cover every current policy outcome, truncation/bounds, unsupported-platform fields, and fail-closed artifact validation; the incremental overhead of these new fields is measured with the established paired protocol; make check and make cross-lint pass for touched platform-gated code. The current handoff channel is unbounded, so this bead must not claim true consumer backpressure or change queue semantics; any bounded-channel behavior change requires separate evidence and scope.
