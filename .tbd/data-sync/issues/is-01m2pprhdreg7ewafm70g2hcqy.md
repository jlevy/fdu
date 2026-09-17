---
type: is
id: is-01m2pprhdreg7ewafm70g2hcqy
title: "PR #78 review R1: Invariant excludes complete/errors and has no rule for partial answers"
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2pprgq5m8mvkr5z1abt5ezr
created_at: 2026-09-17T03:32:29.239Z
updated_at: 2026-09-17T03:39:28.443Z
closed_at: 2026-09-17T03:39:28.442Z
close_reason: "Fixed in PR #78 commits 77da71bd and 4db9d3bd; disposition posted on the PR"
resolution: null
duplicate_of: null
---
PR #78 (https://github.com/jlevy/fdu/pull/78#issuecomment-5708037094), finding R1.

Plan :129-141, :202-206; principles :85-89. Split provenance into tree-describing (complete, errors, coverage) and delivery-describing fields; state the partial rule; add an unreadable-subtree mutation to the harness.
