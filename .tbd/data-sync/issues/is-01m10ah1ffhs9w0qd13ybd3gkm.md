---
type: is
id: is-01m10ah1ffhs9w0qd13ybd3gkm
title: "PR #48 review F8: Run admission-site gate in CI and cover all producers"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:41.294Z
updated_at: 2026-08-27T01:48:23.027Z
closed_at: 2026-08-27T01:48:23.027Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
Makefile:91,164-165; .github/workflows/ci.yml; scripts/check-admission-sites.mjs:11-12; crates/fdu-core/src/opened.rs:964. Wire the gate into CI and scan the full core source with explicit producer allowlisting, including opened.rs. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
