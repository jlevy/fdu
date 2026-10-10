---
type: is
id: is-01m4kkw2gzm232vd6qgmyqab5p
title: Settle delayed allocation before the stale-bytes reader test compares listings
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-10T19:15:39.679Z
updated_at: 2026-10-10T19:15:39.679Z
---
scan::linux_dents::tests::stale_bytes_in_the_buffer_never_reach_a_listing failed once in CI on #194 (Test library build-feature boundaries, job 114287447167) at an engine identical to 0d6011da, where it passed; it passes locally in isolation (40 runs) and in make check. Probable cause: it writes 3,000 one-byte files and compares two consecutive listings field by field, allocated (st_blocks) included, and ext4 writeback of freshly written files under delayed allocation can move st_blocks between two stats. Fix: run sync (through the coreutils command, as the module's mkfifo helper does) after writing the fixture and before the first listing. Proposed patch in https://github.com/jlevy/fdu/pull/194#issuecomment-6101211939.
