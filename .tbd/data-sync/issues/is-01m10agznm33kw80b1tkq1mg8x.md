---
type: is
id: is-01m10agznm33kw80b1tkq1mg8x
title: "PR #48 review F2: Match gitignore negation and ancestor semantics exactly"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:39.443Z
updated_at: 2026-08-27T01:48:22.959Z
closed_at: 2026-08-27T01:48:22.959Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/control/gitignore.rs:104-156. Make per-pattern matches strict, leave ancestor exclusion to ControlTable/index prefix propagation, cover !docs/ and whitelist idioms against git check-ignore, and bump IGNORE_RULES_FINGERPRINT. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
