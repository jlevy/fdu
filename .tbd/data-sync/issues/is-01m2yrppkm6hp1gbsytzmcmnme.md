---
type: is
id: is-01m2yrppkm6hp1gbsytzmcmnme
title: Reconciliation closure overwrites budget and removes metadata-error entries
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T06:40:21.618Z
updated_at: 2026-09-20T06:52:16.641Z
closed_at: 2026-09-20T06:52:16.636Z
close_reason: Fixed and regression-tested in release-state transition slice
resolution: null
duplicate_of: null
---
At release integration base 7de72376, opened budget-stop tests ended as Partial(Inaccessible) because finish_reconcile overwrote the already-committed Partial(Budget) state when the filesystem walk completed but application hit the file budget. Reconciliation also rebuilt unbounded Issue vectors and retained prior-pass omission counts. Preserve the specific incompleteness cause, bound retained issue conversion, replace prior pass failures while preserving concurrently newer issues, and prove warm metadata-failure answers match cold partial scans by dropping unverified entries.

## Notes

Implemented in the isolated release-state worktree. Reconciliation closure now changes coverage to Inaccessible only for actual walk/terminal errors, so apply-only resource refusal preserves Partial(Budget). It consumes borrowed errors directly into the bounded issue store, replaces scoped older failures and the prior root-pass omitted count, and preserves issue epochs published after the pass began. The legacy metadata-error test now verifies the intended cold-equivalence contract: neither warm nor cold retains attributes metadata could not verify. Added regressions for repeated 66-error passes and concurrent newer issues. Validation: fdu-core lib 708 passed/1 ignored; four state transition integration regressions passed outside sandbox; no-default check and formatting passed. Full clippy has no findings in owned scan/index/query_status files and remains blocked by known findings in watch/content/report files owned by other integration slices.
