---
type: is
id: is-01m1dtqrgwd4fn6ekn7dq8a6tg
title: Make detached application skip exact commit consequences
kind: feature
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - design
dependencies:
  - type: blocks
    target: is-01m1dtqxh815zb3zz6m3g11cx6
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:06.331Z
updated_at: 2026-09-01T06:33:11.463Z
---
Introduce one private batch-level consequence boundary shared by the reducer. Detached baseline construction retains facts, roll-ups, scope, issues, provenance, and ApplyStats but allocates no effective-change paths, Impact, Commit, journal entry, or AppliedDelta. Remove the unreleased eager compatibility projection and prove detached and exact fact digests agree.
