---
type: is
id: is-01m12xe7z1vh1739cmpc4k2f7z
title: The joint contract does not state the row orders fdu is required to traverse
kind: task
status: closed
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-28T00:48:41.440Z
updated_at: 2026-08-28T02:03:00.965Z
closed_at: 2026-08-28T02:03:00.964Z
close_reason: null
resolution: null
duplicate_of: null
---
Checkpoint 3C's fdu row in the plan says: 'opened/read.rs tree and flat projections | Traverse the maintained structures in the two exact contract orders and resume without a full selection pass.'

The contract does not state those orders. At MetaBrowser 45266a8, inventory_engine/contract.py contains no documentation of tree, directory, or recent row order; the only 'ordered' reference in the file is the change stream. The orders exist in three places that do not agree:

1. The fdu plan states them in prose: 'a tree page is parent-first. Within each directory, directories precede nondirectories, and each partition is ordered by canonical component UTF-8 bytes.'
2. The MetaBrowser reference provider implements them incidentally.
3. fdu's maintained serving structures impose their own total orders.

The specific ambiguity blocking render depth: 'parent-first' does not distinguish breadth-first level order from depth-first pre-order. Both satisfy the plan's prose, and they emit different row sequences for any tree deeper than one level. _directory_rows in providers/python_inventory.py implements breadth-first level order with an explicit frontier, which appears to be an implementation choice rather than a stated contract. fdu's Tree projection currently returns one directory's children, so the question has not yet arisen; adding max_depth is exactly when it does, and the choice also fixes what a continuation cursor means.

This is the same class of gap as fdu-z01g (recent tie-break). Both are invisible until the cross-provider replay under fdu-xu27, which is the failure mode the rewrite exists to prevent: one concept with two owners and no single definition.

Required before the remaining 3C projection work proceeds:

- state the tree/directory row order in the joint contract, breadth-first level order versus depth-first pre-order, with the intra-directory rule and the byte-level comparison basis;
- state whether include_ignored=false prunes the subtree or only the row, since the reference skips before adding to the next frontier and therefore prunes;
- state the resume cursor's meaning in that order, so a continuation advances without rescanning;
- add the order to the conformance registry so a provider that disagrees fails there rather than in production.

Do not implement max_depth in fdu until this is settled. Blocks fdu-a0cf's remaining tree work.
