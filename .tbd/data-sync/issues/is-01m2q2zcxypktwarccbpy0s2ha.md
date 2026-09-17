---
type: is
id: is-01m2q2zcxypktwarccbpy0s2ha
title: "PR #79 review H1: Rerooting replaces the harness's path spelling, not fdu's canonical root"
kind: task
status: closed
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2q2zbmwp3dd406y9dwbm3q5
created_at: 2026-09-17T07:05:56.925Z
updated_at: 2026-09-17T15:25:27.842Z
closed_at: 2026-09-17T15:25:27.841Z
close_reason: "Addressed in PR #79 (785d3d18, 02e98968, 873e34c0, 0a987d55); CI and the full matrix pass on Linux, macOS, and Windows"
resolution: null
duplicate_of: null
---
PR #79 review (https://github.com/jlevy/fdu/pull/79#issuecomment-5710424165), finding H1.

Normalize the root from the answer itself; failures by placeholder. Windows prints \\?\ paths.
