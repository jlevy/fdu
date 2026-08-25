---
type: is
id: is-01kzsa4v3ewdk78ca90t4192jf
title: "PR#6 C5: provenance changes bypass the clocked delta/change-feed contract"
kind: bug
status: closed
priority: 1
version: 6
labels: []
dependencies:
  - type: blocks
    target: is-01m0rw7bvxtw87tgde30emgs56
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.701Z
updated_at: 2026-08-25T00:59:33.421Z
closed_at: 2026-08-25T00:59:33.421Z
close_reason: |
  Shipped. `make check` green, parity holds.

  THE RULE, now enforced by the type. Every answer-affecting transition advances the clock
  and reaches the change feed at the same atomic boundary as the rows it describes.
  `AppliedDelta` gained `state: Vec<StateChange>` beside `ops`; a state-only delta is
  ordinary and is the whole point, since coverage moving with nothing else moving is still
  an answer changing. Four variants: `Verified{path}`, `Freshness{path, freshness, reason}`,
  `RunFacts`, `Retagged{directories}`.

  `RunFacts` deliberately carries no payload. The envelope is read under the same guard as
  the rows, so what the feed owes a consumer is *that* it moved -- which is what tells it to
  reread. A second copy in the delta could only agree with the first or be wrong about it.

  THE ORIGINAL FINDING, and why the fix is not where the bead pointed. An unchanged upsert
  promotes `entry.source` and returns false, so the promotion never reached a delta. Per
  entry it cannot: a sweep performs millions of those, and emitting millions of
  provenance-only changes is exactly what the interval representation exists to avoid. The
  sweep is the unit. `begin_reconcile` and `finish_reconcile` are now commits, so the
  promotion of everything beneath a subtree arrives as two transitions rather than none.

  CALL SITES. `begin_reconcile` -> Freshness{Reconciling}. `finish_reconcile` -> either
  Freshness{Partial, reason}, or Freshness{Fresh} + Verified in ONE commit, because
  splitting them would publish a moment at which the subtree read as fresh but unverified,
  which never happened. `set_run_facts` -> RunFacts, and only when the facts actually
  changed: an idle revalidation loop would otherwise advance the clock forever and evict
  real history to store the news that nothing happened. `rebind_tag_rules` -> Retagged.

  OPEN-TIME INITIALIZATION IS NOT A TRANSITION, and the API says so structurally.
  `Index::with_run_facts` is a consuming builder -- an owned index is one nobody is reading,
  so there is no answer these facts contradict and minting a clock would publish a change
  away from a state that was never visible.

  THE JOURNAL BUDGET. `AppliedDelta::len()` counts ops + transitions. A free transition is a
  retention bound a producer can walk past, and the bound exists precisely so a long-lived
  server's history cannot grow without limit. Mutation-checked: restoring `ops.len()` leaves
  `journal_ops` at 0 and the truncation signal never fires.

  A DEFECT THIS WORK WOULD HAVE CREATED, FIXED WITH IT. `Session::next_batch` took its
  cursor from the last delta it carried. Once the re-tag became a commit, a concurrent
  producer -- a caller refreshing against the same handle while the watch runs -- could
  commit between the watcher's deltas and the rebind, and the batch would name a position
  past a commit it never delivered. An index has one writer at a time but not one producer.
  `Session` now tracks its resume position and reports `reset` on a gap, which is exactly
  what reset means: the consumer's position cannot be advanced by replay.

  ORDERING. `rebind_tags_for` moved ahead of everything derived from the batch, and its
  delta is pushed onto `applied`. So the cursor follows the re-tag, `dirty_rollups` is
  computed after the governed directories are known, and the governed-directory escalations
  are emitted in the delta's own place in the sequence rather than ahead of changes with
  lower clocks.

  `dirty_rollups` learned about `Retagged` and about nothing else. Trust moving is not
  values moving: if a verified sweep dirtied every ancestor it touched, every reconciliation
  would look like a mutation of the numbers and a consumer would discard a cache that was
  right.

  SURFACES. `Batch.state` and `ChangeSet.state` on both Python surfaces, as a typed
  `StateChange` with a `Transition` enum; `since()` renders them per delta, a batch renders
  them at its own terminal position because a batch is delivered whole.

  TESTS, each mutation-checked.
  - `a_state_transition_advances_the_version_and_reaches_the_feed`: the sweep's two
    transitions arrive through `since`, with no op among them.
  - `a_run_envelope_commits_only_when_it_actually_moved`: both halves, separately mutated.
  - `state_transitions_are_charged_against_the_journal_budget`.
  - `a_commit_this_stream_did_not_deliver_makes_the_batch_a_reset`, with the control that an
    uninterrupted stream does *not* reset -- a signal that fires always means nothing.
  - `a_re_tag_commits_and_the_batch_cursor_follows_it`: mutated by taking the cursor from
    the path deltas only, which is the exact shape of the original defect.
  - Python: a refresh's transitions through `since`, and each naming the subtree it applies
    to.

  WHAT THIS UNBLOCKS. `fdu-fltq`'s "state-only transition" is delivered; its remaining three
  signals (dirty query kinds, and the all-dirty/reset labelling) sit on this carrier.
resolution: null
duplicate_of: null
---
types.rs:1-16, index.rs:602-689,1302-1313,732-760. Unchanged upsert mutates entry.source then returns false; finish_reconcile mutates verified directly. Neither advances Clock nor reaches AppliedDelta. Medium.

## Notes

EXACT-HEAD INTERACTION at a3960fb (2026-08-24). fdu-91ru now reads RunFacts and freshness under the same guard as projections, but refresh calls reconcile_subtree_handle, performs analysis, and only later calls set_run_facts under another write guard. set_run_facts changes neither Clock nor the journal. Therefore the same Cursor can identify before and after run-state envelopes, a read between the two commits can see new rows with prior run facts, and no state-only change reaches a consumer. Extend this bead's clocked trust transition to RunFacts and provider state; add a forced interleaving and assert that every answer-affecting state transition advances the returned version and reaches the lossless change batch.

POST-LANDING WATCH INTERACTION at 558461a (2026-08-24). Session.next_batch captures Batch.cursor before calling rebind_tags_for. rebind_tag_rules then adopt_tag_rules/retag mutates answer-affecting tag state without advancing Clock or journaling the transition. The batch's synthetic invalidations use the prior applied delta's clock, and dirty_rollups was computed before the governed paths were known. Fold retag/provider-state transitions into the same clocked commit and lossless batch; the returned cursor must follow that commit, not precede it.
