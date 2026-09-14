---
type: is
id: is-01m0tra5gw0ap6nbbzgt7egvr4
title: Controls-on reconcile re-reads every .gitignore even when its attrs are unchanged
kind: bug
status: open
priority: 1
version: 15
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
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
labels:
  - pr47-review
  - metabrowser
  - stack-followup
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
child_order_hints:
  - is-01m2exj373p2d2ma725h7pz393
created_at: 2026-08-24T20:45:09.518Z
updated_at: 2026-09-14T02:58:21.993Z
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

--- Round-six findings (d58d9c5) verified. No code changed. ---

Two of the three findings in the d58d9c5 review land on this bead, and both are real.

1. Deleting the last pruned control file retags without a clocked delta. Confirmed at
   index.rs:1688-1728: rebind_tag_rules calls adopt_tag_rules -- which retags every
   entry and rebuilds the planes -- and only then returns Ok(None) because the newly
   bound rules govern nothing. Removing the last .gitignore is exactly that shape: the
   old tags disappear, no clock advances, and no Retagged transition enters the
   journal. The early return's own comment ("changed nothing a consumer can observe")
   is true of a rebind that governs nothing and false of one that *stopped* governing
   something. The fix is the union of old and new governed directories, or an exact
   moved set, committed whenever tags or planes moved -- including to zero.

2. The test I shipped verifies the effect and not the notification. It polls tags_of
   through with_index, which reads the retagged state directly, so it passes while a
   consumer watching batches is told nothing. That is a real weakness and the reason
   the defect survived: assert the deletion through a WatchBatch at its terminal
   cursor, not a side read.

3. The live control-directory registry is monotonic. adopt_pruned_control_dirs only
   extends, sorts and deduplicates, and I documented the deletion path as deliberately
   retaining the directory forever, calling it "bounded by the directories that ever
   held one". For a long-lived watcher that is not a bound, and the reviewer is right
   to reject it. The save/load asymmetry they report -- load caps at one million while
   save writes any count, so churn can produce a snapshot fdu refuses on its own next
   open -- I have taken as reported and not independently verified.

2026-09-13 (PR #48 review CLASS-8, deferred here as the reduced residual of this bead): the bind walk this bead filed is fixed on #48, but two correct-yet-wasteful passes remain. Every snapshot load BFS-walks the whole tree, allocating a PathBuf per entry, even when the control table is empty (crates/fdu-core/src/index.rs, the control install walk after load, around the ControlTable install in the snapshot load path); and every warm reconcile re-reads every .gitignore regardless of unchanged attrs (crates/fdu-core/src/scan.rs, read_control_op called from the reconcile walk). Review fix: return early on an empty control table, and skip control files whose attributes are unchanged. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101

2026-09-14 (fixer, at afe0f89 on codex/opened-root-inventory-rewrite). The description and the first two note rounds describe code that is gone: rebind_tag_rules, adopt_tag_rules, adopt_pruned_control_dirs and MAX_CONTROL_DIRS have no match under crates/. Retitled to the one live residual.

Snapshot half: fixed in afe0f89. install_controls (crates/fdu-core/src/index.rs:1228-1250@afe0f89) returns before the reclassification walk when both the loaded table and the one it replaces are empty, since no ignored bit can move then. Test loading_a_snapshot_without_controls_skips_the_reclassification_walk counts walk visits through a test-only per-thread probe: 0 with the early-out and 65 with it removed (mutation run). A snapshot that does carry a control still walks and reclassifies.

Reconcile half: left open. Skipping read_control_op when the entry's attrs equal its baseline is not exact with the facts the index carries today. The table stores a content identity (ControlSource.identity, a hash of the bytes), not the attrs it was read at. So "attrs unchanged" proves the table is current only if every writer that commits a .gitignore entry's attrs also commits its content. Three writers do not:
1. A read error. The listing walk pushes the upsert, then reads (scan.rs:3617-3631@afe0f89). If the file became unreadable, the new attrs commit and the old source stays. Today the next reconcile reads again, reports the error, and stays partial. With the skip it would find the attrs unchanged, read nothing, report complete, and mark the path Fresh with the stale rules: a silent lie.
2. A batch split. The upsert can flush at the batch bound before the ControlUpsert is pushed. The ControlUpsert is conditioned on the pre-upsert baseline, so it is rejected as stale. The retry then sees unchanged attrs and would skip the very read the retry exists to perform.
3. A subtree rooted at the file. Reconciling a retained .gitignore as its own subtree never reads the control at all (fdu-uzzv, reproduced). The next full walk is what repairs that today; with the skip, nothing would.

Making the skip exact needs a design decision, not an early-out. Option (a): record in the index, through the commit path, the attrs at which each control source was admitted, and skip only when those equal the observed attrs. Option (b): make every writer commit a .gitignore's attrs and content atomically, and withhold the attrs when the read fails. Either removes all three cases. Measure the read cost on a real tree before choosing: it is one open plus one read per .gitignore, against one stat per entry.
