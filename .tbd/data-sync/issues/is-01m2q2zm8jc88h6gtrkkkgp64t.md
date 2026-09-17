---
type: is
id: is-01m2q2zm8jc88h6gtrkkkgp64t
title: "PR #79 review M3: Classify the Windows findings: onefs refusal order and samesize_keepmtime"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2q2zbmwp3dd406y9dwbm3q5
created_at: 2026-09-17T07:06:04.431Z
updated_at: 2026-09-17T15:25:27.894Z
closed_at: 2026-09-17T15:25:27.894Z
close_reason: "Addressed in PR #79 (785d3d18, 02e98968, 873e34c0, 0a987d55); CI and the full matrix pass on Linux, macOS, and Windows"
resolution: null
duplicate_of: null
---
PR #79 review (https://github.com/jlevy/fdu/pull/79#issuecomment-5710424165), finding M3.

onefs refused cold but reported as cache miss under only (P1.3); keepmtime serves pre-rewrite metrics where change time is not compared.
