---
type: is
id: is-01m10agzyrxw19sc3vm80df0xm
title: "PR #48 review F3: Remove untrusted-product allocation from glob matching"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:39.735Z
updated_at: 2026-08-27T01:48:22.986Z
closed_at: 2026-08-27T01:48:22.986Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/control/gitignore.rs:244-246. Replace pattern_len × text_len memo allocation with bounded/O(1)-space matching or a justified named cap and reuse; add adversarial tests. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
