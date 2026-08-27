---
type: is
id: is-01m10ah15q864ga3ptfvt5qg86
title: "PR #48 review F7: Prevent control-file open from blocking on FIFO replacement"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:40.982Z
updated_at: 2026-08-27T01:48:23.015Z
closed_at: 2026-08-27T01:48:23.015Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/scan.rs:2288-2301. Close the lstat-to-open race with a platform-safe nonblocking/open-handle type check and tests; never let FIFO/device replacement hang workers. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
