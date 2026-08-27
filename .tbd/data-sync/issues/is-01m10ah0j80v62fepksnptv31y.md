---
type: is
id: is-01m10ah0j80v62fepksnptv31y
title: "PR #48 review F5: Decide inaccessible-directory observation handoff semantics"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:40.359Z
updated_at: 2026-08-27T01:48:23.000Z
closed_at: 2026-08-27T01:48:23.000Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/opened.rs:1197-1309 and scan.rs:360-362. Decide whether known inaccessible boundaries permit Watching with Partial(Inaccessible) or remain terminal; retain exact causes and add deterministic coverage/documentation for the chosen contract. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
