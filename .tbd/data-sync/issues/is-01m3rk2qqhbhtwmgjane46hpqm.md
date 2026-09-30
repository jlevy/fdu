---
type: is
id: is-01m3rk2qqhbhtwmgjane46hpqm
title: "Readable-but-unsearchable directory: the d_type skip answers differently from the full index (tree route H185, summary route H72)"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3rhbnwhvym3rmjbynhdc4he
hold: null
hold_until: null
created_at: 2026-09-30T07:22:19.761Z
updated_at: 2026-09-30T09:56:20.519Z
started_at: 2026-09-30T09:56:19.830Z
closed_at: 2026-09-30T09:56:20.519Z
close_reason: "Fixed in 0e59c387 on claude/pdu-uniform-lead (option 1): each listing carries a Searchability that starts unproven, every child is stated until one stat in the listing succeeds, and only then are directory and symlink kinds taken from d_type, on the native reader and the portable path (tree route and summary route). Fixtures: a 0400 directory holding a subdirectory, a file and a symlink in boundary_trees() and the summary control cases, under require_permission_bits(); native-reader search-denied test lists the same kinds; new latch unit test. Evidence: run as nobody (setpriv, no FDU_TEST_ALLOW_NO_PERMISSION_BITS) the differentials and the search-denied test fail without the latch and pass with it; strace -c statx on the release binary 70,416 -> 72,075 (node-modules-dense) and 87,006 -> 87,656 (linux-v6.12), so 83% and 89% of exp-197's saving is kept. Recorded in exp-197 and the H185 registry row; CHANGELOG Fixed line for the summary route's earlier answer."
resolution: null
duplicate_of: null
---
Review finding R163-1 on PR #163 (issuecomment-5906249640). Under skip_dir_symlink_stat, a directory with read but not search permission (0400) lists, but every child's stat fails with EACCES; the full index reports an error per child and no entries, while the folded tree (H185, native reader and musl) admits DT_DIR/DT_LNK children from d_type: one more dir counted and the symlink's error dropped. The summary route has answered this way since H72 (musl) and #161 (native reader). Fix (option 1, being implemented on claude/pdu-uniform-lead): stat each listing's children until the first stat succeeds, then take kinds from d_type; a failing first stat keeps stat'ing. Differential fixture under require_permission_bits(), proven as a non-root user.
