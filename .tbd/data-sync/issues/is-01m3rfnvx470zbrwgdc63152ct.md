---
type: is
id: is-01m3rfnvx470zbrwgdc63152ct
title: Settle freshly written test fixtures before comparing allocated bytes across walks (ext4 writeback race)
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-30T06:22:52.323Z
updated_at: 2026-09-30T09:55:27.240Z
closed_at: 2026-09-30T09:55:27.240Z
close_reason: "Fixed in 42ef6c23. Root cause as the bead records: ext4 delayed allocation drops a fresh file's reserved block a moment before charging the allocated one, so a stat during writeback reads a one-block file as 0 blocks, and a differential walking one fresh fixture twice comes out 4096 apart. Ported test_support::settle_allocations (sync -f, syncfs, Linux only; no-op elsewhere) from e8f87e51 with the same name and signature, and settled every fixture a test walks twice while comparing allocated bytes or whole attributes: execution.rs (control cases, random trees, boundary trees, failed-listing tree, compact-vs-indexed summary), linux_dents.rs random_tree (both random-tree differentials), opened.rs progressive-discovery-vs-one-shot, and twelve scan.rs differential fixtures. Verified: cargo test -p fdu-core --all-features over execution, scan, linux_dents and the opened differential: 196 passed. Residual: the golden and Python parity replays walk fixtures the harness wrote moments earlier and are not settled; a one-block difference there would show as an unclassified parity difference on CI."
resolution: null
duplicate_of: null
---
Root-caused on PR #163 (2026-09-30): on ext4, a freshly written file's reserved blocks are released during writeback a moment before the real block is counted, so a statx in between reads a one-block file as 0 blocks. A test that walks the same fresh fixture twice and compares allocated bytes fails rarely by exactly 4096 per affected file. Evidence: a C probe read blocks=0 on fresh 3-byte files during syncfs 23/90 rounds (0/140 once synced); a fixture-shaped walk read 4923392 vs 4927488 in 6/2166 walks, 0/716 once settled. The race predates the pdu track (b1376507 is exposed too). #163 adds test_support::settle_allocations (sync --file-system on Linux, a no-op elsewhere) in e8f87e51 and settles the execution.rs fixtures. Still exposed: linux_dents::random_trees_read_the_same_natively_and_portably, summary_folds_agree_across_worker_counts_on_random_trees, and any other test comparing allocation across two reads of a fresh tree. Fix: cherry-pick e8f87e51 onto claude/stability-fixes and settle every such fixture; grep for tests comparing allocated/blocks across walks.
