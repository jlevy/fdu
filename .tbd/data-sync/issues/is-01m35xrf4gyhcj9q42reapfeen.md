---
type: is
id: is-01m35xrf4gyhcj9q42reapfeen
title: Arbitrate overlapping reconciliation verification even when metadata is unchanged
kind: bug
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T01:23:23.407Z
updated_at: 2026-09-23T01:33:17.049Z
---
Recovered state at abc35f96 protects newer omitted issues but an older finishing pass can reinsert errors disproved by a newer clean pass; its conditional facts can also overwrite newer unchanged verification because entry revisions do not move. Track scoped supersession only for active passes, reject stale overlapping observations, suppress only error evidence covered by a newer verification, and preserve disjoint sibling evidence. Deterministic same-scope and ancestor/child regressions plus bounded ownership review required.

## Notes

2026-09-22 recovered-baseline red proof: older_pass_cannot_publish_errors_after_newer_clean_verification fails at 47dbb51c because an older pass resurrects a cause after newer clean verification. New isolated implementation uses active scope/version evidence. Each pass budgets at most one normalized shadow scope per retained entry at begin (len includes root, so root-only is one; index has no zero-entry state). Descendant scopes covered by an ancestor collapse. Distinct absent child verifications demonstrate no bound exists from current retained paths alone, so exceeding this derived budget discards proof and enters explicit Retry, never widens verification. All future batches from that pass are refused; closing preserves prior issues and newer facts, publishes partial state and an interruption issue, then returns a private retry flag. Scan uses its existing incomplete/retry protocol, without a new public error. Full newer ancestor proof can supersede an interrupted pass. Tests cover same-scope stale causes, unchanged newer facts, disjoint scopes, ancestor/child sibling cause preservation, absent-child bounded ownership, and publication before retry result; validation ongoing. Draft old worktree remains untouched.
