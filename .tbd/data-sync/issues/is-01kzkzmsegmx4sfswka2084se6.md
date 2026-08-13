---
type: is
id: is-01kzkzmsegmx4sfswka2084se6
title: Automate stable performance regressions and claim governance
kind: feature
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-09T19:22:54.799Z
updated_at: 2026-08-13T14:34:04.371Z
---
Add a tiny cross-platform harness smoke with no speed claim, establish a protected stable scheduled runner and compatible-baseline/noise policy, retain raw artifacts, triage material regressions, and generate the dedicated-host Phase 1 report. README and release claims must link to reviewed raw evidence and a reproduction manifest; generic hosted CI is never treated as a stable stopwatch.

## Notes

Claim governance must require explicit cache-state evidence. The real-tree tool comparator now records the independent fingerprint pass and minimum full-tree warmups and rejects zero-warmup warm-steady claims. Published warm-steady results must not imply full metadata residency or controlled-cold behavior; Linux and macOS cold matrices remain separately tracked under fdu-nffc and fdu-rjqx.
