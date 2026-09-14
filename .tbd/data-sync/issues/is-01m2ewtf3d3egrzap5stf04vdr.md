---
type: is
id: is-01m2ewtf3d3egrzap5stf04vdr
title: Coverage stays Partial(Inaccessible) after a refresh or observer walk disproves the last boundary
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-09-14T02:44:28.396Z
updated_at: 2026-09-14T02:44:28.396Z
---
Found while addressing PR #48 verification review 5193206420 (FIX48-1, FIX48-3); not a finding of that review. Only the Watching transition re-derives Complete coverage from Partial(Inaccessible) (index.rs apply_opened_state, LIFE-3). Since 509b536 a complete reconciliation drops the retained issues it disproved, and since b803b8e it records completeness for the directories it listed. So on a root that is not watched -- or a watched root after the handoff -- a refresh or observer walk that reads the last directory discovery could not read leaves coverage Partial(Inaccessible) with no retained issue explaining it (previously the stale Permission issue stayed and explained a boundary that no longer existed). Re-deriving Complete from 'no boundary issue remains' is not safe: a per-entry metadata error is retained at the child's path while discovery left the parent directory incomplete, so dropping the child's issue does not prove the parent complete, and restoring Complete there would bring back the FIX48-1 contradiction (Unknown { Building } below an incomplete directory on a Complete root). Direction to decide: track incomplete directories (or a count of them) so coverage can be re-derived exactly at any complete commit, or give Partial(Inaccessible) with no retained cause a documented meaning. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
