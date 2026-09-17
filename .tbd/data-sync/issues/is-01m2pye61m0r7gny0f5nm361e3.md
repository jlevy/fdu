---
type: is
id: is-01m2pye61m0r7gny0f5nm361e3
title: "P1.4.2: Record walk failures per path; TreeStatus::of with bounded, path-ordered errors"
kind: task
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye6byr0vp0t4v8k2nk8vb
  - type: blocks
    target: is-01m2pye70x3nc8p6t827sywb23
parent_id: is-01m2pmra8yqrcxg27kc6ezg9vd
created_at: 2026-09-17T05:46:38.515Z
updated_at: 2026-09-17T20:20:00.014Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", commit 2. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `scan.rs`: `consolidate_detached_index` (`:3739-3755`) and the error branches in `reconcile_target_inner` (for example `:4352-4355`) call `index.record_walk(&errors, started_at)`, which marks each failed path `Partial` and retains its issue.
- `index.rs`: `set_initial_freshness` (`:2671`), `begin_reconcile` (`:2692`), `finish_reconcile` (`:2728`, `:2776`) mark failed paths rather than the pass root.
- `TreeStatus::of(index, request)` and `TreeStatus::of_walk(scan)`.
- Order of errors: `errors` holds the 64 failures smallest by path and `errors_omitted` counts the rest. One-shot scan errors are already sorted (`scan.rs:2414`), while retained issues are capped at insertion in walk order (`index.rs:2486`), so `record_walk` keeps retained failures in path order regardless of walk order.

**Tests**

- `TreeStatus::of` names each failed path on a partial one-shot index.
- A tree with more than 64 failed paths gives equal cold and warm `errors` and `errors_omitted`.
- Review `index.rs:8524-8608`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: issues are capped at 64 while scan errors are not, so `errors` becomes bounded with `errors_omitted`.

## Notes

Acceptance for the sidecar (PR #82 review F1, fdu-iiy5): layer 3 holds the partial-pass sidecar write behind this bead, because a partial pass marks the pass root Partial, so provenance_of promotes nothing and content_record_writable answers false for every unchanged file. When failed paths are marked per path, re-enable the write in content_tier_writable and pin it: after a warm partial pass with one changed file over a tree with an unlistable directory, the sidecar holds a record for every file the pass stat'd unchanged and none under the unlistable directory (the reviewer measured 7 records, not 1).
