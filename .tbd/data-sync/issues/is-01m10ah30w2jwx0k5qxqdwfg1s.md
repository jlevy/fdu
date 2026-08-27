---
type: is
id: is-01m10ah30w2jwx0k5qxqdwfg1s
title: "PR #48 review F14: Exercise retained-issue and all-dirty clamps"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:42.876Z
updated_at: 2026-08-27T01:48:23.059Z
closed_at: 2026-08-27T01:48:23.059Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/tests/reference_model.rs:216-222,594-611 and index.rs:1467. Add a named model/index case exceeding 64 retained issues and 256 dirty paths so raised as well as lowered bounds are guarded. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
