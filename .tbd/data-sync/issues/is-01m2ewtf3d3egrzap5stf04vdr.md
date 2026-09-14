---
type: is
id: is-01m2ewtf3d3egrzap5stf04vdr
title: Coverage stays Partial(Inaccessible) after a refresh or observer walk disproves the last boundary
kind: bug
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-09-14T02:44:28.396Z
updated_at: 2026-09-14T03:18:39.589Z
---
Found while addressing PR #48 verification review 5193206420 (FIX48-1, FIX48-3); not a finding of that review. Only the Watching transition re-derives Complete coverage from Partial(Inaccessible) (index.rs apply_opened_state, LIFE-3). Since 509b536 a complete reconciliation drops the retained issues it disproved, and since b803b8e it records completeness for the directories it listed. So on a root that is not watched -- or a watched root after the handoff -- a refresh or observer walk that reads the last directory discovery could not read leaves coverage Partial(Inaccessible) with no retained issue explaining it (previously the stale Permission issue stayed and explained a boundary that no longer existed). Re-deriving Complete from 'no boundary issue remains' is not safe: a per-entry metadata error is retained at the child's path while discovery left the parent directory incomplete, so dropping the child's issue does not prove the parent complete, and restoring Complete there would bring back the FIX48-1 contradiction (Unknown { Building } below an incomplete directory on a Complete root). Direction to decide: track incomplete directories (or a count of them) so coverage can be re-derived exactly at any complete commit, or give Partial(Inaccessible) with no retained cause a documented meaning. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420

## Notes

2026-09-14 (LIFE-9 fixer, fdu-jxuq): not fixed here; it does not fall out of the LIFE-9 change. LIFE-9 now (a) holds the published IndexState.freshness at Reconciling for the life of the observation handoff instead of publishing Fresh after the handoff's own full pass (Index::published_freshness in index.rs), and (b) closes each subtree of a multi-path refresh on its own walk's outcome (scan.rs reconcile_paths_target). Neither touches coverage. Two related observations for whoever decides this bead: (1) On a Ready root whose coverage is Partial(Inaccessible), DiscoveryTransition::Finish and ::Inaccessible set state.freshness = Partial directly, without a freshness mark, so a later complete refresh of any path recomputes published freshness from marks alone and publishes Fresh beside Partial(Inaccessible) coverage; the freshness side of this bead is the same shape as its coverage side and should be decided with it. (2) A per-entry metadata error is retained at the child's path while discovery left the parent directory incomplete, so 'no boundary issue remains' cannot re-derive Complete; tracking incomplete directories (a count suffices for the root-level answer) is the direction that keeps FIX48-1 fixed.
