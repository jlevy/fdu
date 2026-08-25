---
type: is
id: is-01m0tra5gw0ap6nbbzgt7egvr4
title: "[bug] Gitignore bind walks the whole tree at open, even cache-only"
kind: bug
status: open
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:50.764Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:12.297Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5021835489
    at: 2026-08-25T17:17:54.573Z
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T20:45:09.518Z
updated_at: 2026-08-25T17:17:54.575Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
At PR #47 exact head d58d9c5036818f33fe390c31453eb7548ba7abfa, pruned control-file events now reach Session, but the rebind is still not lossless or bounded. IndexHandle::rebind_tag_rules computes governed directories from the newly bound rules, calls adopt_tag_rules (which retags and rebuilds planes), then returns without a commit when the new governed set is empty. Deleting the last .gitignore therefore changes tags and rollups without advancing Clock or entering AppliedDelta; the new integration test observes the handle directly and never asserts a WatchBatch, cursor, or dirty set, so it passes this defect. Capture the union of old and new governed directories (or have retag return the exact moved set) and commit whenever answer-affecting bits/planes moved, including deletion to zero. In addition, adopt_pruned_control_dirs only extends/sorts/deduplicates and deletion deliberately retains history forever. A long-lived watch can grow this set without bound; snapshot load caps it at MAX_CONTROL_DIRS=1,000,000 while snapshot save writes any count, so the engine can write a snapshot it later refuses. Maintain the current control-file set with removal-aware updates, enforce one shared bound at mutation/save/load, and test last-file deletion through exact batch/cursor/dirty state plus create-delete churn and snapshot self-roundtrip at the bound.

## Notes

Answered at the branch head after the fourth review round.

Hidden pruning gives .gitignore no row, and the live admission rule rewrites its
upsert to a removal of a path that was never there, which commits nothing. A
session watching the delta alone therefore saw an idle tree while every gitignore
tag under it went on describing a file that had changed -- the one case where an
answer-affecting change leaves nothing in the delta to notice it by.

The row was never the obstacle: a tag rule reads its control file from disk by
path. What was missing was being told, in two places.

1. watch::admitted records every op naming a control file the scope excludes,
   before anything rewrites it, and WatchApplyReport::pruned_control_files carries
   it. Deliberately not folded into the `outside` decision that drives the rewrite:
   a *removal* is never outside -- removing what is not there is harmless and
   always allowed -- yet deleting a pruned control file is exactly as
   answer-affecting as creating one. Keying on the rewritten upsert covers create
   and edit and misses delete, which is what the first implementation did and what
   the test caught.
2. Session::rebind_tags_for deposits the *directory* through
   IndexHandle::adopt_pruned_control_dirs before rebinding. Binding looks in the
   index and in the directories a *walk* pruned a control file from, and a control
   file created after that walk is in neither -- so the live rule deposits the same
   fact the walk would have. A directory stays on the list once it is on it,
   including after the file is deleted: binding reads from disk by path, so a stale
   entry finds nothing, and the list is bounded by the directories that ever held
   one, which is the bound the walk's own list has.

Test: a_pruned_control_file_still_rebinds_the_tags_it_governs, over a pruning
scope with a promoted gitignore rule. All three lifecycle events, because they
fail differently -- create has no prior row, edit has no row but a real prior
effect, and delete is the one a rewritten-upsert rule misses. It asserts the file
is still outside the index throughout, so the fix is not accidentally admitting it.

Five mutations, all caught: never recording, recording only the rewritten upsert
(delete), the session ignoring the signal, never depositing the directory, and
depositing the path instead of its directory.
