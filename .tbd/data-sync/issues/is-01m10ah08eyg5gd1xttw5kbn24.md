---
type: is
id: is-01m10ah08eyg5gd1xttw5kbn24
title: "PR #48 review F4: Prevent revalidate stat errors from becoming removals"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:40.045Z
updated_at: 2026-08-27T01:48:22.994Z
closed_at: 2026-08-27T01:48:22.994Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/scan.rs:3043-3128. Mark a listed name seen before fallible metadata, align Reject/ControlOnly removal handling with sibling walkers, and add regression coverage proving transient stat failure does not publish absence. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
