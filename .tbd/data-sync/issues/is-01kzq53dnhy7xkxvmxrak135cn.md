---
type: is
id: is-01kzq53dnhy7xkxvmxrak135cn
title: "Snapshot format v3: reserved journal-cursor section"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53dw7bawahp6m77sx9gd4
created_at: 2026-08-11T00:56:00.432Z
updated_at: 2026-08-11T00:56:07.216Z
---
Phase 1 of the FSEvents-scoped revalidation plan. Bump FORMAT_VERSION to 3 with an optional journal-cursor section (tag byte, volume UUID, event ID, capture time), save-side stub writing 'none' on all platforms, load-side decode with corrupt-cursor failing closed, v2 loading as cursor-absent. Includes the platform-neutral journal/mod.rs: cursor types, the pure gate decision function (G1-G10 in the spec), changed-set normalization, and exhaustive unit tests for every gate row. Mergeable alone; also unblocks the block-format spike fdu-1vd0 from breaking the format later.
