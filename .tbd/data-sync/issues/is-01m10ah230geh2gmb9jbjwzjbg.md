---
type: is
id: is-01m10ah230geh2gmb9jbjwzjbg
title: "PR #48 review F10: Reject empty gitignore segment patterns"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:41.919Z
updated_at: 2026-08-27T01:48:23.040Z
closed_at: 2026-08-27T01:48:23.040Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/control/gitignore.rs:86-96. Ignore malformed all-slash patterns such as /// rather than matching the full tree; add git-derived regression coverage. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
