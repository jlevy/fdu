---
type: is
id: is-01m10ah4891y04ygz4wc5qext0
title: "PR #48 review S3: Document the one-time type-rule fingerprint invalidation"
kind: task
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m10afnckpkf0m2fnhgw3sx1d
created_at: 2026-08-27T00:39:44.136Z
updated_at: 2026-08-27T01:48:23.084Z
closed_at: 2026-08-27T01:48:23.084Z
close_reason: "Completed in b3cb609: all review findings and suggestions were implemented, the five opened-root session goldens and coverage gate landed, make check passed on the exact tree, and macOS/Windows cross-lint passed."
resolution: null
duplicate_of: null
---
Suggestion from PR #48 implementation review. Add a concise changelog entry that TYPE_RULE_FINGERPRINT now hashes parsed manifest values and intentionally invalidates prior snapshots/content sidecars once. Review: https://github.com/jlevy/fdu/pull/48#issuecomment-5432784958
