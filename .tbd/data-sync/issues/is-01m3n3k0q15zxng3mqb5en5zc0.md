---
type: is
id: is-01m3n3k0q15zxng3mqb5en5zc0
title: "Peer-agreement script: account for directories' own blocks on non-APFS filesystems"
kind: task
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
hold: null
hold_until: null
created_at: 2026-09-28T22:53:52.737Z
updated_at: 2026-10-01T03:27:21.437Z
started_at: 2026-10-01T03:27:20.646Z
closed_at: 2026-10-01T03:27:21.436Z
close_reason: "Merged in PR #170 (main 7f2f268b): the top-level check adds each subtree's own directory and symbolic-link blocks, so the script passes on ext4; a row one block off still fails."
resolution: null
duplicate_of: null
---
scripts/qa_peer_agreement.py's top-level comparison assumes APFS, where directories allocate no blocks; on ext4 each directory's own block makes rows differ, so the self-test exits 1 on Linux even when the totals agree. Found while fixing fdu-djz0.
