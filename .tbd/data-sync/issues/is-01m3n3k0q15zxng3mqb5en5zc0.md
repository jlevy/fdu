---
type: is
id: is-01m3n3k0q15zxng3mqb5en5zc0
title: "Peer-agreement script: account for directories' own blocks on non-APFS filesystems"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T22:53:52.737Z
updated_at: 2026-09-28T22:53:52.737Z
---
scripts/qa_peer_agreement.py's top-level comparison assumes APFS, where directories allocate no blocks; on ext4 each directory's own block makes rows differ, so the self-test exits 1 on Linux even when the totals agree. Found while fixing fdu-djz0.
