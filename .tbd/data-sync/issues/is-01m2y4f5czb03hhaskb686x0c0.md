---
type: is
id: is-01m2y4f5czb03hhaskb686x0c0
title: "H140: Linux walk leftover after current engine"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - H140
dependencies: []
parent_id: is-01m2y4f4g34vdbgxf0jcvt8dw3
created_at: 2026-09-20T00:46:43.102Z
updated_at: 2026-09-20T01:11:09.983Z
closed_at: 2026-09-20T01:11:09.983Z
close_reason: "exp-139: walk still 95.7-96.1% of default-tree component; leftover is getdents64+statx floor; no userspace cut; do not retry H71"
---
default-tree leftover on Linux. Expected getdents64+statx floor. Do not compile a walk trim. Do not retry H71.

## Notes

PREDICT (2026-09-20, Linux KVM)

Hypothesis: H140 / exp-139
Job: default-tree leftover profile (+ same-binary pair as attachment)
Subject: linux-v6.12 (92,474 entries), same frozen reconstructible clone as exp-138
Determination: walk share >=90% of component, leftover is getdents64+statx (expected) or a userspace stage >=3% Darwin did not see.
Do not compile a walk trim. Do not retry H71.
Regime: quiet first. Do not lower the 25% bar.
