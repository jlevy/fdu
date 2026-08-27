---
type: is
id: is-01m10agzaa4a3kb2s7f12pd670
title: "PR #48 review F1: Bound gitignore double-star matching"
kind: bug
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:39.080Z
updated_at: 2026-08-27T01:48:22.944Z
closed_at: 2026-08-27T01:48:22.934Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/control/gitignore.rs:164-175. Collapse adjacent DoubleStar segments, memoize segment/path states to O(segments × components), add an explicit pattern bound if justified, add regression and git-derived conformance coverage, and bump IGNORE_RULES_FINGERPRINT. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
