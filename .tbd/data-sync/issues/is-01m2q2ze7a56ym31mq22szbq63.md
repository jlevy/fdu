---
type: is
id: is-01m2q2ze7a56ym31mq22szbq63
title: "PR #79 review H2: Cache-only 'refused' accepts any failure, including panics and I/O errors"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2q2zbmwp3dd406y9dwbm3q5
created_at: 2026-09-17T07:05:58.248Z
updated_at: 2026-09-17T15:25:27.850Z
closed_at: 2026-09-17T15:25:27.850Z
close_reason: "Addressed in PR #79 (785d3d18, 02e98968, 873e34c0, 0a987d55); CI and the full matrix pass on Linux, macOS, and Windows"
resolution: null
duplicate_of: null
---
PR #79 review (https://github.com/jlevy/fdu/pull/79#issuecomment-5710424165), finding H2.

Require exit 1 and the 'snapshot is not usable' message on the CLI, and fdu's own error type in Python.
