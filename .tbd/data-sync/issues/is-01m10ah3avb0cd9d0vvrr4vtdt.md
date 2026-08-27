---
type: is
id: is-01m10ah3avb0cd9d0vvrr4vtdt
title: "PR #48 review F15: Remove review-identified magic constants and drift risks"
kind: task
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:43.194Z
updated_at: 2026-08-27T01:48:23.066Z
closed_at: 2026-08-27T01:48:23.066Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
control.rs:44; scan.rs:2304; snapshot.rs:611; scan.rs:3192-3258; classify.rs:261. Share CONTROL_SOURCE_OVERHEAD, remove or test ReconcilePathsReport.walked, and add a MANIFEST_FAMILIES/family_from_name bijection guard. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
