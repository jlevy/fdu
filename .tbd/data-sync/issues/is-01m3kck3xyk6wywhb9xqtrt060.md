---
type: is
id: is-01m3kck3xyk6wywhb9xqtrt060
title: "watch: every macOS rename escalates to a full-root reconcile"
kind: bug
status: open
priority: 1
version: 5
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T06:52:44.338Z
updated_at: 2026-09-28T11:24:12.388Z
---
Found by the 2026-09-27 resident soak (explorations/change-sources/watch-soak/). notify 8.2.0's FSEvents backend reports every ItemRenamed as an unpaired Modify(Name(Any)); crates/fdu-core/src/watch.rs record() (~L816-834) escalates any unpaired rename to InvalidateReason::UnpairedRename on the whole root. FSEvents flags are sticky per path, so later events on the same path escalate too. On agent state B (~476k entries) 61.5% of raw events carried ItemRenamed (atomic temp+rename writes): 174 root reconciles in 61 min (one per 20 s, 9.9 CPU s each), 47.8% of a core (sys-dominated), RSS median 442 MB / peak 906 MB. Windows without a root reconcile cost ~1% of a core. Fix direction: scope a one-sided rename to the named path's parent (recursive when the named path is a directory), relying on FSEvents delivering both sides or a drop flag; keep root escalation for drop/overflow flags. Note the accidental benefit: the polling covered the open-writer gap, so a fix must pair with the libproc writer re-stat (fdu-vhrb).

## Notes

2026-09-28 (watch-fix agent): fix pushed as afd15ab0 on claude/watch-rename-scope (base 543df0a6 = PR #142 head; engine = a5c0ab46). Not closed: awaiting lead review.

Backend facts (notify 8.2.0 sources): FSEvents emits one Modify(Name(Any)) per ItemRenamed record, one per side, every record under the watched root; loss = MustScanSubDirs -> Flag::Rescan. inotify emits From and To for every rename (plus a cookie-paired Both), IN_MOVE_SELF only for the watch root; loss = IN_Q_OVERFLOW -> Rescan. Windows emits From/To (never paired); moves across the tree boundary arrive as REMOVED/ADDED. kqueue names only the old path. So the old code escalated EVERY rename to a root reconcile on macOS, Linux, and Windows alike.

Design (watch.rs record/verify_intent): each in-root side is a Verify{renamed} of its own path. Gone -> Remove (subtree). Present file -> Upsert. Present dir -> Upsert + InvalidateSubtree(dir, UnpairedRename), independent of relist_new_dirs. Renamed + present must also be listed byte-for-byte by its parent (case-/normalization-insensitive lookup kept both spellings of a case-only rename); a miss re-lists and re-stats, then reconciles the parent. Root escalation kept for Flag::Rescan, a rename naming the root, a rename naming no in-root path, and backends that may never name the new side (kqueue, via RecommendedWatcher::kind()). InvalidateReason::UnpairedRename kept (public enum, Python string) with its doc rewritten.

Tests: 12 rename tests in watch.rs, 2 of them replacing the old UnpairedRename tests (record shapes per backend, sticky flags, kept escalations, applying-driver cases asserting no root invalidation + cold-scan-equal index, case-only rename, listing refresh, real-backend rename convergence). Mutation: disabling the membership check or the rename relist fails 4 tests. fdu-core --all-features: 925 lib + integration + doctests pass; workspace clippy -D warnings clean.

Paired soak on the agent-state root (base a5c0ab46 vs fix, concurrent, 620 s after initial reports): base 24 root reconciles, 205.7 CPU s (33.2% of a core); fix 0 root reconciles, 4 subtree relists, 5.0 CPU s (0.81%). Peak footprint fix 1.09 GB vs base 0.84 GB, transient: vmmap showed ~533 MB of freed MALLOC_LARGE blocks retained by libmalloc mid-run, falling to ~101 MB; footprint at the end 455 vs 419 MB.

Follow-up coupling: the root reconciles were masking the open-writer gap; fdu-vhrb (writer re-stat) now matters for macOS accuracy.

Pending when handed back (timing lock held >100 min by a macOS measurement cell): the accuracy comparison of each watcher's persisted view against the two --cache off walks (O1/O2 sandwich, open writers enumerated every 30 s), and a post-lint re-run of the watch tests plus the CLI watch_controls/watch_persistence tests. Queued as a detached job in the watch-fix worktree (attic/soak/followup.sh); results land in attic/soak/compare.json (aggregate only), attic/soak/followup.log, and attic/gate3.log. The lint fixes were a parameter rename and slice::from_ref in tests; clippy compiled all targets after them.

2026-09-28 review (Fable, read-only): ship with docs changes. No unsound rename shape found per backend. Required before merge: document the macOS open-writer behavior change (CHANGELOG Changed bullet, README Live Updates, docs/usage.md ~450, architecture Native observation) — follow-up soak: fixed 4 stable misses, all open-for-write; base 1. Test rerun after lint fixes (attic/gate3.log) pending. Follow-ups: fdu-ek21 (Windows overflow), fdu-pcvk (fingerprint-gated relist), fdu-3n47 (case-sensitivity skip, test gaps). fdu-vhrb stays P1 for 0.2.x.
