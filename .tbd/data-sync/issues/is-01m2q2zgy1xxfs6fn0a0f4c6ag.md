---
type: is
id: is-01m2q2zgy1xxfs6fn0a0f4c6ag
title: "PR #79 review H4: copytree drops symlink mtimes on Windows; fixture test ignores link mtimes"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2q2zbmwp3dd406y9dwbm3q5
created_at: 2026-09-17T07:06:01.024Z
updated_at: 2026-09-17T07:06:01.024Z
---
PR #79 review (https://github.com/jlevy/fdu/pull/79#issuecomment-5710424165), finding H4.

Re-stamp links after copy; include link mtimes in the determinism test.
