---
type: is
id: is-01kzwxej9axvrpknyke9hz9m2k
title: Plan phased fast file content metrics
kind: task
status: closed
priority: 2
version: 11
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies: []
parent_id: is-01kzg4d256qmchmtyvttnpvn4y
created_at: 2026-08-13T06:37:43.593Z
updated_at: 2026-08-13T12:03:01.322Z
closed_at: 2026-08-13T07:50:12.245Z
close_reason: Implementation-ready plan approved, refined, validated, and decomposed into dependency-wired beads under fdu-j5ny.
---
Turn the completed fast file-content metrics research into an active implementation spec with independently useful phases, explicit metric semantics, API and cache boundaries, performance gates, and rollout criteria.

## Notes

Approved plan refined to a concrete file/function map against the current PR #8 code: OpenConfig/ScanConfig ownership, optional sparse ContentIndex, worker/delta/cache seams, report/CLI/Python changes, FlexDoc-compatible additive logical-word statistics, tryscript fixture, multilingual self-host invariants, and named performance jobs. Created implementation epic fdu-j5ny with 16 ordered child beads; every performance bead is blocked by its phase's semantic-lock bead. Full make check passed in the isolated worktree.
