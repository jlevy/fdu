---
type: is
id: is-01m74sny84wap88zdgvh79hxd1
title: "Design: detect or report change-time moves that come only from hard-link count changes"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T07:55:39.140Z
updated_at: 2026-11-11T07:55:39.140Z
---
From review G3 on jlevy/fdu#209 (https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811). A link count is inode metadata, so link() and unlink() set the status-change time (ctime) of the file, which every remaining name shares. Under the default clock, change time (the newer of mtime and ctime), making or removing a hard link anywhere therefore makes every other name of the file read as just changed. Package managers that install by linking from a shared store do this routinely: uv on Linux (and, by its documentation, Windows) hard-links from its cache, and pnpm does where the filesystem has no reflinks; cp -l and cloning a local git repository do too. Reviewer's measurement with uv 0.12.1 on ext4, where each installed file's link count rose from 2 to 3: after 'uv venv A && uv pip install --python A/bin/python idna certifi' and T=$(date +%s), 'fdu A --kind file --since @$T --view summary' gave 0 files; after installing the same packages into an unrelated environment B it gave 632 KiB in 26 files; with --time modified, 0 files; after 'rm -r B', 26 files again. So 'what was written since yesterday' reads a .venv or node_modules as written whenever any project sharing its packages was installed or deleted, and the staleness recipe (--kind dir --include .venv --before 30d) does not call it stale while that keeps happening. Two open questions. (1) Batch routes: can a report detect or say that a change time moved only by a link-count change (for example by comparing nlink across runs, or by noting files with nlink > 1 whose ctime is newer than their mtime), and should it? (2) Live routes: a watch or an observing opened root verifies only the names events carry, so a link made outside the tree sends no event, and one made inside updates only the new name; the file's other names keep their old change time until next read. Re-reading other in-tree names of a linked file would need an inode index the engine does not keep. The PR documents the limit (design principles' verification section, usage guide, skill, --docs, CHANGELOG, TimeBasis docs) and pins it with the watch_session test a_hard_link_moves_a_change_time_the_watch_reads_only_where_an_event_names_it, which holds the limit as it is. Whether the default clock should stay change time given this is a maintainer decision being asked separately. Done when the design is decided and either implemented with tests or recorded as a documented limit with its reasons.
