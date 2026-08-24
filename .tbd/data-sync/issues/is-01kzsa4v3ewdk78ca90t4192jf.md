---
type: is
id: is-01kzsa4v3ewdk78ca90t4192jf
title: "PR#6 C5: provenance changes bypass the clocked delta/change-feed contract"
kind: bug
status: open
priority: 1
version: 5
labels: []
dependencies:
  - type: blocks
    target: is-01m0rw7bvxtw87tgde30emgs56
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.701Z
updated_at: 2026-08-24T23:00:20.356Z
---
types.rs:1-16, index.rs:602-689,1302-1313,732-760. Unchanged upsert mutates entry.source then returns false; finish_reconcile mutates verified directly. Neither advances Clock nor reaches AppliedDelta. Medium.

## Notes

EXACT-HEAD INTERACTION at a3960fb (2026-08-24). fdu-91ru now reads RunFacts and freshness under the same guard as projections, but refresh calls reconcile_subtree_handle, performs analysis, and only later calls set_run_facts under another write guard. set_run_facts changes neither Clock nor the journal. Therefore the same Cursor can identify before and after run-state envelopes, a read between the two commits can see new rows with prior run facts, and no state-only change reaches a consumer. Extend this bead's clocked trust transition to RunFacts and provider state; add a forced interleaving and assert that every answer-affecting state transition advances the returned version and reaches the lossless change batch.

POST-LANDING WATCH INTERACTION at 558461a (2026-08-24). Session.next_batch captures Batch.cursor before calling rebind_tags_for. rebind_tag_rules then adopt_tag_rules/retag mutates answer-affecting tag state without advancing Clock or journaling the transition. The batch's synthetic invalidations use the prior applied delta's clock, and dirty_rollups was computed before the governed paths were known. Fold retag/provider-state transitions into the same clocked commit and lossless batch; the returned cursor must follow that commit, not precede it.
