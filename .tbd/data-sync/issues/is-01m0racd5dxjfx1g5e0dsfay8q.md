---
type: is
id: is-01m0racd5dxjfx1g5e0dsfay8q
title: Roll-up leaf counts so empty is decidable from the aggregate
kind: feature
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
child_order_hints:
  - is-01m0rh6bhzbhf822jt62q14kvn
created_at: 2026-08-23T22:03:13.964Z
updated_at: 2026-08-24T00:03:01.229Z
closed_at: 2026-08-24T00:03:01.228Z
close_reason: |-
  A subtree of symlinks weighs nothing and is not nothing. The aggregate now says which.

  WHAT WAS TRUE: contribution() returned a default roll-up for Symlink and Other, so a
  directory holding a hundred symlinks was zero files, zero directories and zero bytes --
  arithmetically identical to one holding nothing. A listing greying out empty directories
  was greying out one with contents, or greying out none.

  WHAT LANDS:
  - InternedRollUp, RollUp and RollUpScalars gain `others`: descendant entries that are
    neither files nor directories. contribution() returns one for a symlink or a device,
    and merge/unmerge carry it like every other invertible reducer.
  - RollUp::entries() and ::is_empty(), same on RollUpScalars. bytes cannot answer the
    emptiness question -- an empty file, a symlink and nothing at all all weigh nothing.
  - ChildSnapshot::is_empty_subtree() -> Option<bool>, which is where "a partial subtree can
    never claim emptiness" lives. None for a non-directory, which has no subtree, and None
    for a Status::Partial roll-up, which has not accounted for one: zero there means
    "nothing found yet". Decided in the engine rather than left to the consumer, because
    deciding it needs the row's provenance as well as its counts.
  - ChildRemainder gains `others`, so the page partition property still holds over the new
    field. Python: RollUp.others/.entries/.is_empty, Child.empty, and Child.totals moves
    from SummaryRow to a dedicated DirectoryTotals -- a view's summary row answers "what
    does this query cover", this answers "how big is this child and is there anything in
    it", which is why one carries `others` and the other does not.

  NO SNAPSHOT BUMP, and the bead's assumption that one was needed is wrong in a way worth
  recording. The format persists kind, name and attrs; roll-ups are rebuilt on load through
  insert_loaded_child, so `others` is derived from data already stored. Bumping the version
  would have discarded every user's cache to gain nothing.

  TESTS. Three in the engine plus a Python smoke check. The direct one builds `links` (two
  symlinks) beside `hollow` (nothing), asserts both are zero files, zero dirs, zero bytes,
  and that only one of them is empty. Another drives a real partial subtree through
  begin_reconcile/finish_reconcile(complete=false) and pins that it declines to answer
  rather than claiming empty. The page partition test now covers `others` at every bound.

  GAP NAMED, NOT HIDDEN: the report views still cannot tell the two apart, because
  SummaryRow and TreeNode carry no such field. Left out deliberately -- adding a column to
  the text table is a command-line display decision and moves every golden -- and filed as
  fdu-or38 rather than left for someone to rediscover.

  make check passes.
resolution: null
duplicate_of: null
---
contribution() gives Symlink and Other a default rollup, so a subtree containing only symlinks is arithmetically indistinguishable from an empty one -- a listing cannot tell them apart. Maintain a non-directory leaf count (or per-kind counts) in rollup state so a complete subtree's emptiness is an exact fact from the aggregate. A partial subtree can never claim emptiness. Partition property extends to the new fields; snapshot version increments. Joins the maintained-state union priced by fdu-n4gn.
