---
type: is
id: is-01m0vx6h8tfjyqjaxmt67nabrp
title: "Batched scoped refresh: many hint paths, one commit, one receipt"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T07:29:47.802Z
updated_at: 2026-08-25T09:29:21.861Z
closed_at: 2026-08-25T09:29:21.861Z
close_reason: "Batched scoped refresh shipped: scan::reconcile_paths / reconcile_paths_handle returning a RefreshReceipt, and Index.refresh_paths on the Python surface. One operation, one terminal position, overlapping hints folded into one walk, the whole batch announced before any of it is read, and typed per-path refusals. Nine engine tests plus a Python smoke check, mutation-checked three ways."
resolution: null
duplicate_of: null
---
The interactive-client contract's refresh takes 1-1024 observed paths in one request and
returns a receipt naming which were accepted and which rejected. fdu's `Index.refresh(path)`
takes one path and returns counts.

Adapter-side iteration is not equivalent, and the difference is the one the whole contract
exists to prevent: N calls are N reconciliations, N commits and N cursors, so a receipt
covering them describes a range rather than a boundary, and a consumer resuming from "the"
cursor cannot say which of the N it is past. It also costs N ancestor merges where one
reconciliation over the union would cost one.

Wanted: a scoped refresh taking a bounded set of paths, reconciling their union under one
guard, committing once, and returning per-path acceptance beside the counts. Rejection is a
real answer, not an error: a path outside the scope, pruned by admission, or not canonical
is rejected rather than silently reconciled as nothing.

Acceptance: refreshing N paths in one call commits exactly one delta and returns exactly one
cursor; the per-path result distinguishes accepted from rejected; and the union of N single
refreshes and one batched refresh of the same N leaves identical index state.

## Notes

CONFIRMED (MB74-D3, answered on PR #47 at head `eaae030`): the consumer contract stays
batched and atomic, and adapter-side iteration is explicitly forbidden. One refresh request
must produce one native fdu operation, one commit, one cursor, and one per-path receipt.
This bead is the implementation owner; no contract change is needed on either side.



SHIPPED. `scan::reconcile_paths` / `reconcile_paths_handle` returning a `RefreshReceipt`,
and `Index.refresh_paths(paths)` on the Python surface.

What one operation buys over N calls, in order of how much it matters:

1. **One terminal position.** N calls name N positions, so a receipt covering them
   describes a range and a caller cannot say which of the N its cursor is past.
2. **Overlapping hints cost one walk.** Paths are folded into their nearest accepted
   ancestor before anything is read, so `src`, `src/nested` and `src/top.txt` are one
   subtree. A hint at the root collapses the whole batch to one whole-tree walk.
3. **The batch is announced before any of it is read.** Announcing each subtree just
   before its own walk would let a consumer read an index where half the batch is marked
   reconciling and half still claims to be fresh, with nothing saying the second half is
   about to move.
4. **One scope validation, one classification pass, one merged report.**

Refusal is a first-class answer, typed: `OutsideRoot`, `BeyondDepth`, `NotAdmitted`,
`Bounded`. A receipt listing only what it did would make "reconciled, and nothing had
changed" and "never looked" the same answer, and a caller feeding its own watcher's hints
would re-send a path forever waiting for news that will never come. Every refusal is
decided from the request and the config alone, which is what lets the whole batch be
classified before any of it is announced.

`accepted` names every path that was reconciled, including one folded into an ancestor:
it *was* reconciled, and saying otherwise makes a caller re-send it. `walked` names the
subtrees actually read, and is the measure of what the batching bought.

An empty path set reads nothing, deliberately not the whole tree. Conflating them would
make a dropped hint list mean "re-read everything", which is the most expensive possible
response to having lost track of what changed. The whole-tree form stays `reconcile_handle`
/ `refresh()`.

Bounded at `MAX_REFRESH_PATHS` (1024, matching the consumer contract). Past the bound a
path is *refused* rather than erroring the call, so a caller that oversends still gets a
complete answer about what was done.

Tests: `crates/fdu-core/tests/batched_refresh.rs`, nine cases, plus
`public_smoke.py:check_a_batched_refresh_is_one_operation_not_a_loop`. Mutation-checked
three ways: removing the covering-set fold, interleaving announcements with walks, and
replacing the native call with adapter-side iteration in the Python layer.

One mutation *passed* and forced a fixture change: the announcement-ordering test ran
against an unchanged tree, so reconciliation emitted no row operations and the ordering
assertion had nothing to order against. It now writes into every subtree first and asserts
the operation count is `Some`, so it cannot go vacuous again.

Also fixed here, the same defect the walk budget exposed in the open path: the Python
`refresh` rebuilt its issue list from `report.scan.errors`, dropping every typed condition
without an I/O failure behind it. Both refresh shapes now ask `ScanReport::issues()`.

NOT DONE, and recorded rather than hidden: the CLI has no batched form. `--refresh a b c`
is not a shape a one-shot command line wants, and the rule the repository actually holds is
one-directional -- nothing may be reachable *only* by flag. A library-only capability is
allowed.
