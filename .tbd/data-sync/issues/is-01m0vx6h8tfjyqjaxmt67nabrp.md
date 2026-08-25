---
type: is
id: is-01m0vx6h8tfjyqjaxmt67nabrp
title: "Batched scoped refresh: many hint paths, one commit, one receipt"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T07:29:47.802Z
updated_at: 2026-08-25T08:06:09.674Z
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
