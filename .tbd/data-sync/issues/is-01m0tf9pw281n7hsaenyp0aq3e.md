---
type: is
id: is-01m0tf9pw281n7hsaenyp0aq3e
title: Bounded tree remainders drop non-file leaves
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T18:07:37.337Z
updated_at: 2026-08-24T19:22:01.962Z
closed_at: 2026-08-24T19:22:01.962Z
close_reason: |
  Confirmed and fixed. The finding was correct: `fdu-or38` gave `TreeNode` an `others`
  dimension and left `Remainder` without one, so a node could truthfully report a hundred
  non-file leaves, have the one child holding them withheld by a bound, and return a
  remainder accounting for none of them. Emitted rows plus remainder stopped being the node.

  That is not a missing field, it is silent truncation in one dimension -- which is exactly
  what the remainder exists to prevent. "Truncate freely, never silently" has to hold per
  dimension, not only for the dimensions that happened to exist when the remainder was
  written.

  `others` now rides on `Remainder` in Rust, in the JSON and YAML shapes, in the PyO3 dict,
  and on the Python `Remainder` dataclass and its parser. `ChildRemainder` -- the listing
  page's -- already carried it from `fdu-5hip` and needed nothing.

  Both aggregation paths were wrong, and both are fixed: `Remainder::absorb`, which folds
  withheld rows one at a time under a row limit, and `withheld_children`, which sums every
  directory child at once under a depth bound.

  TEST. `a_bound_accounts_for_withheld_non_file_leaves_in_both_aggregation_paths` asserts
  `sum(emitted child.others) + remainder.others == node.others` under each bound kind, over
  a fixture with symlinks in three ranked subtrees so a bound of one has something to
  withhold whichever way it ranks them. It also asserts the node's own total does not move
  under a bound, since a bound narrows the rendering and never the totals.

  Mutation-checked rather than assumed: removing the two `others +=` lines makes it fail
  with `left: 2, right: 4`, so it is testing the arithmetic rather than agreeing with
  whatever the code produced.

  `make check` green.
resolution: null
duplicate_of: null
---
At PR 47 head 5012069, TreeNode gains others but Remainder still carries only rows, files, dirs, bytes, and allocated. Remainder::absorb and withheld_children omit others, and the JSON, YAML, and Python remainder shapes cannot carry it. A bounded tree whose omitted directory contains only symlinks says the full node has non-file leaves but cannot account for them in its machine-readable remainder, violating truncate freely never silently and the partition relation. Fix: add others across Remainder, absorb, withheld_children, serializers, and the Python model and conversion. Tests under both limit and depth bounds must assert emitted child others plus remainder.others equals node.others. Review finding FDU47-R9.
