---
type: is
id: is-01m2esgndtpzgepd8x9vsqqw8y
title: One-shot and shared reconcile still re-queue unclosable invalidations after LIFE-2
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T01:46:41.465Z
updated_at: 2026-09-14T01:46:41.465Z
---
Scope gap left by PR #48 review LIFE-2, which was fixed for opened roots only (fdu-e6vi, f276cb5). Recorded by the fixer. The disposition map says "The one-shot and shared reconcile APIs keep their documented retry."

**Defect the review found.** An invalidation whose subtree cannot be read (EACCES) reconciles as incomplete and is restored to the pending queue. A caller that drains the queue after every event then re-walks the same unreadable subtree on every unrelated event, forever. After a root escalation (a rescan, a notify error, an unpaired rename; routine on FSEvents) with one `0700` directory anywhere below, that is a full-tree walk per event.

**What the fix covered.** `ReconcileTarget::retries_incomplete` (`crates/fdu-core/src/scan.rs:919` on codex/opened-root-inventory-rewrite at f917cb7) now splits by target:
- `Controlled` (opened root): restores only for `stale > 0 || resource_refused > 0`.
- `Direct | Shared`: still `!report.is_complete()`, so a scan error re-queues.

**Where the old behaviour is still reachable.**
- `Watcher::apply_next` (`crates/fdu-core/src/watch.rs:356`) calls `apply_intent`, which calls `scan::reconcile_pending_handle` (`watch.rs:416`), which is the `Shared` target. That is the per-event drain pattern the review described. Whether `fdu --watch` and the Python watch session reach it that way is still to be confirmed.
- `scan::reconcile_pending` (`Direct`) is used by one-shot library callers, which drain when they choose.

**Why it was not changed.** Existing tests pin the retry for those targets: `partial_pending_reconciliation_remains_queued_for_retry` (`scan.rs:6905`, a mode-000 subtree stays queued) and `failed_pending_reconciliation_remains_queued_for_retry` (`scan.rs:6886`).

**To do.**
1. Decide whether the documented retry is right for any caller that drains after every event, as `Watcher::apply_next` does. If it is not, give `Shared` the opened root's rule, retaining scan errors as issues, and rewrite the two tests to pin the new contract.
2. If the retry stays for `Direct`, make sure no per-event driver uses it, and document the difference on `reconcile_pending` and `reconcile_pending_handle`.
3. Add a watch-level regression: one rescan plus N unrelated creates over a tree with one unreadable directory reconciles that directory once, not N+1 times.

Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101
