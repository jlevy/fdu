---
type: is
id: is-01kzsa4v3ewdk78ca90t4192jf
title: "PR#6 C5: provenance changes bypass the clocked delta/change-feed contract"
kind: bug
status: closed
priority: 1
version: 9
labels: []
dependencies:
  - type: blocks
    target: is-01m0rw7bvxtw87tgde30emgs56
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.701Z
updated_at: 2026-08-25T04:01:39.742Z
closed_at: 2026-08-25T04:01:39.741Z
close_reason: |
  Closed again. The reopen note is dated at `fad3d2f`; both remaining defects it names were
  fixed in `4fbb7d1`, which lands after it. Verified against the current head rather than
  assumed, claim by claim.

  1. "reconcile/reconcile_handle promise each effective committed delta, but reconcile_target
     commits begin/finish state without sending either AppliedDelta to the sink."
     Fixed. `begin_reconcile` and `finish_reconcile` return the `AppliedDelta` they committed,
     and `reconcile_target` sinks both -- on the completing path and on the error path, where
     the sweep still records `Partial(Failed)`. scan.rs:2951, 2955, 2967.

  2. "The unchanged-tree test accepts an empty delta vector and checks only the index clock,
     so it does not prove the documented callback."
     Fixed, and this was the sharper half of the finding: the test passed while the sink
     received nothing, because `all()` is vacuously true on an empty vector. It now asserts
     the two deltas, their transitions, and their consecutive clocks, through
     `assert_sweep_deltas` -- shared by the direct and shared paths so neither can quietly
     publish less than the other. scan.rs:5558, 5887.

  3. "Session::Batch flattens Vec<StateChange> away from each AppliedDelta clock."
     Fixed. `CommittedState { clock, change }` pairs them, so the flattening is
     unrepresentable rather than discouraged. watch_session.rs:157.

  4. "PyWatch stamps every transition with the terminal cursor even though StateChange.clock
     documents the actual commit."
     Fixed. fdu-py/src/lib.rs:2059 uses the transition's own clock. Mutation-checked through
     `since`: restoring the terminal stamp reports the sweep's announcement at clock 4 with
     the rows it precedes at clock 2 -- the ordering the announcement exists to produce,
     inverted.

  The rule this bead carried is now enforced structurally in three places rather than by
  review: `AppliedDelta` carries `state` beside `ops` so a transition cannot avoid the clock;
  `CommittedState` carries the clock so a carrier cannot drop it; and transitions are charged
  against the journal budget so a bounded history stays bounded.

  If something here is still wrong, the finding is about `4fbb7d1` or later rather than about
  `fad3d2f`, and I would rather have it named against that head.
resolution: null
duplicate_of: null
---
types.rs:1-16, index.rs:602-689,1302-1313,732-760. Unchanged upsert mutates entry.source then returns false; finish_reconcile mutates verified directly. Neither advances Clock nor reaches AppliedDelta. Medium.

## Notes

EXACT-HEAD INTERACTION at a3960fb (2026-08-24). fdu-91ru now reads RunFacts and freshness under the same guard as projections, but refresh calls reconcile_subtree_handle, performs analysis, and only later calls set_run_facts under another write guard. set_run_facts changes neither Clock nor the journal. Therefore the same Cursor can identify before and after run-state envelopes, a read between the two commits can see new rows with prior run facts, and no state-only change reaches a consumer. Extend this bead's clocked trust transition to RunFacts and provider state; add a forced interleaving and assert that every answer-affecting state transition advances the returned version and reaches the lossless change batch.

POST-LANDING WATCH INTERACTION at 558461a (2026-08-24). Session.next_batch captures Batch.cursor before calling rebind_tags_for. rebind_tag_rules then adopt_tag_rules/retag mutates answer-affecting tag state without advancing Clock or journaling the transition. The batch's synthetic invalidations use the prior applied delta's clock, and dirty_rollups was computed before the governed paths were known. Fold retag/provider-state transitions into the same clocked commit and lossless batch; the returned cursor must follow that commit, not precede it.

Reopened: Reopened at exact PR #47 head 278457a after reviewing ac38584. State is clocked in the index but is not delivered losslessly by the live reconciliation path. ReconcileTarget::begin_reconcile and finish_reconcile (crates/fdu-core/src/scan.rs:869-880) discard the AppliedDelta identity, and reconcile_target (2941-2955) never calls sink for those commits; only operation batches in flush_reconcile_batch (3619-3633) reach it. Session::next_batch builds Batch.state solely from that sink vector (watch_session.rs:246-304). An unchanged reconciliation therefore advances the index clock twice but yields no state/cursor to the live batch; a reconciliation with row deltas creates clock gaps and reset instead of carrying the reconciling/fresh/verified transitions. The scan test at 5518-5543 is vacuous about delivery: all() passes on its empty deltas vector and it asserts only the index clock. The carrier then flattens StateChange without its commit clock, and PyWatch stamps every transition with the terminal cursor (crates/fdu-py/src/lib.rs:2054-2062), contradicting StateChange.clock and losing state/op ordering. Return and sink each state AppliedDelta in commit order, or assemble the batch from IndexHandle::since(session_resume) under one guard; preserve each transition's actual clock. Test an unchanged live reconciliation, interleaved state and row commits, terminal cursor equality, and exact transition clocks.

EXACT-HEAD UPDATE at PR #47 fad3d2f (2026-08-25). Building Session batches from IndexHandle::since fixes the previously reported omission of reconcile begin/finish state from that one live path. Keep this bead open for the remaining public and clock-fidelity defects.

The public reconcile/reconcile_handle contract at scan.rs:2887-2926 still promises to publish each effective committed delta, but reconcile_target commits begin/finish state at 2941-2955 without sending either AppliedDelta to the supplied sink; only operation batches reach it at 3619-3633. The unchanged-tree test at 5518-5543 accepts an empty delta vector and checks only the index clock, so it does not prove the documented callback.

Session::Batch also flattens Vec<StateChange> away from each AppliedDelta clock at watch_session.rs:323-337, and PyWatch stamps every transition with the terminal cursor at fdu-py/src/lib.rs:2054-2062 even though StateChange.clock documents the actual commit. Preserve a clocked state value or the AppliedDelta sequence end to end, and assert interleaved state/op ordering plus the public reconcile sink.
