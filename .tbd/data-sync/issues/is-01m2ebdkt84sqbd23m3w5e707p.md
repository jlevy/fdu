---
type: is
id: is-01m2ebdkt84sqbd23m3w5e707p
title: "PR #48 review LIFE-9: Fresh published mid-handoff; one failed subtree downgrades a multi-path refresh"
kind: bug
status: closed
priority: 3
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:21.447Z
updated_at: 2026-09-14T15:33:22.209Z
closed_at: 2026-09-14T15:33:22.208Z
close_reason: "c9b3754: Index::published_freshness holds published freshness at Reconciling through the observation handoff (Stale/Partial still show); reconcile_paths_target closes each subtree of a multi-path refresh on its own walk. Golden journal-and-observation-recovery loses the Clock(6) Reconciling/Fresh transition. Test: readable and unreadable directory refreshed together, run 8x. fdu-08aj left open with analysis."
resolution: null
duplicate_of: null
---
Low. index.rs:1802-1848; scan.rs:3323-3329. The handoff publishes freshness Fresh while the phase is still Reconciling, and reconcile_paths_target applies one completion flag to every subtree of a multi-path refresh, so one failed subtree downgrades verified siblings to Partial. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).

## Notes

2026-09-14 (fix wave, w-life): fix written, NOT yet compiled or tested -- host below the 6 GiB disk floor before any build. Local branch w-life in worktree agent-a365bdbfabb1cd605, off baf6c00. What b803b8e already did: finish_reconcile records DirectoryComplete for directories a complete pass listed; it did not touch either half of this finding. Change (a): Index::published_freshness (index.rs) derives IndexState.freshness from the marks but holds Reconciling while phase == Reconciling and the marks say Fresh; used at the three sites that recomputed state.freshness (invalidation, begin_reconcile, finish_reconcile). Stale/Partial still show through. Effect on the golden journal-and-observation-recovery: the handoff's Clock(6) commit loses its IndexState{Reconciling/Reconciling -> Reconciling/Fresh} transition and Clock(7) becomes Reconciling/Reconciling -> Watching/Fresh; golden must be regenerated with FDU_UPDATE_OPENED_ROOT_GOLDEN and the diff read. Change (b): scan.rs reconcile_paths_target keeps one completion flag per walked subtree (its own report.is_complete(); false for a subtree not walked after an Err) and passes it to that subtree's finish_reconcile; listed_incomplete stays merged because finish_reconcile filters by starts_with(path). Test: opened.rs multi_path_refresh_closes_each_subtree_on_its_own_walk (unix, skipped when permission bits are not enforced): refresh([readable, blocked]) with blocked chmod 000 -> readable Fresh with a Verified{readable} transition, blocked Partial, one issue. fdu-08aj does not fall out of this; its note records why.
