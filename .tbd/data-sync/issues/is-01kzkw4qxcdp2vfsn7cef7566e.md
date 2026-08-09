---
type: is
id: is-01kzkw4qxcdp2vfsn7cef7566e
title: Preserve negative timestamps in newest-mtime rollups
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzkw4ddv9g9jry50tp4xzgtw
created_at: 2026-08-09T18:21:43.211Z
updated_at: 2026-08-09T18:31:47.266Z
closed_at: 2026-08-09T18:31:47.266Z
close_reason: Implemented with regression coverage; the complete local handoff gate passes.
---
RollUp uses zero for an empty tree and initializes max reduction at zero, so files with mtimes before the Unix epoch incorrectly aggregate to zero. Make file presence govern the reducer identity, preserve negative values through add/update/remove, and convert pre-epoch portable metadata to negative nanoseconds.
