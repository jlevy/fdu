---
type: is
id: is-01m10ah3mvnn5kp4s0gz8kewx4
title: "PR #48 review S1: Make reference-model helper independence honest"
kind: task
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:43.514Z
updated_at: 2026-08-27T01:48:23.071Z
closed_at: 2026-08-27T01:48:23.071Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
Suggestion from PR #48 implementation review. Re-derive Commit::retained_cost and Commit::applied_delta in the independent reference model, or narrow its independence claim and explain why direct helper tests are sufficient. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
