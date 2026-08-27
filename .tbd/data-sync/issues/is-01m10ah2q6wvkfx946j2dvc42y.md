---
type: is
id: is-01m10ah2q6wvkfx946j2dvc42y
title: "PR #48 review F12: Support leading closing bracket in gitignore classes"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:42.565Z
updated_at: 2026-08-27T01:48:23.053Z
closed_at: 2026-08-27T01:48:23.053Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/control/gitignore.rs:249-253. Match []] according to git wildmatch or explicitly document a deliberate incompatibility; prefer conformance and add regression coverage. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
