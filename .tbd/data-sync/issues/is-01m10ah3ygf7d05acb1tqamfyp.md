---
type: is
id: is-01m10ah3ygf7d05acb1tqamfyp
title: "PR #48 review S2: Add a git-derived gitignore conformance corpus"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:43.823Z
updated_at: 2026-08-27T01:48:23.079Z
closed_at: 2026-08-27T01:48:23.079Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
Suggestion from PR #48 implementation review and supporting oracle for F1/F2/F10/F11/F12. Build table-driven cases against recorded git check-ignore semantics, including negated ancestors, whitelist idioms, trailing double-star, []], all-slash, and documented case sensitivity. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
