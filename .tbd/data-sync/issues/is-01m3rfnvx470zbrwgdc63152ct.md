---
type: is
id: is-01m3rfnvx470zbrwgdc63152ct
title: Settle freshly written test fixtures before comparing allocated bytes across walks (ext4 writeback race)
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-30T06:22:52.323Z
updated_at: 2026-09-30T06:22:52.323Z
---
Root-caused on PR #163 (2026-09-30): on ext4, a freshly written file's reserved blocks are released during writeback a moment before the real block is counted, so a statx in between reads a one-block file as 0 blocks. A test that walks the same fresh fixture twice and compares allocated bytes fails rarely by exactly 4096 per affected file. Evidence: a C probe read blocks=0 on fresh 3-byte files during syncfs 23/90 rounds (0/140 once synced); a fixture-shaped walk read 4923392 vs 4927488 in 6/2166 walks, 0/716 once settled. The race predates the pdu track (b1376507 is exposed too). #163 adds test_support::settle_allocations (sync --file-system on Linux, a no-op elsewhere) in e8f87e51 and settles the execution.rs fixtures. Still exposed: linux_dents::random_trees_read_the_same_natively_and_portably, summary_folds_agree_across_worker_counts_on_random_trees, and any other test comparing allocation across two reads of a fresh tree. Fix: cherry-pick e8f87e51 onto claude/stability-fixes and settle every such fixture; grep for tests comparing allocated/blocks across walks.
