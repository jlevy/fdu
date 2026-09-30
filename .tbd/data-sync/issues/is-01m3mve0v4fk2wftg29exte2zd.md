---
type: is
id: is-01m3mve0v4fk2wftg29exte2zd
title: "Linux spikes: fdu main vs H159 (PR 150) vs pdu vs diskus on balanced 1M"
kind: task
status: closed
priority: 1
version: 3
delegate: claude-code@vm
labels:
  - research
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
hold: null
hold_until: null
created_at: 2026-09-28T20:31:20.420Z
updated_at: 2026-09-29T22:00:34.671Z
started_at: 2026-09-28T20:31:31.018Z
closed_at: 2026-09-29T22:00:34.670Z
close_reason: "Superseded by the 2026-09-29 quiet peer run on linux-balanced-1m (report-2026-09-27-fdu-linux-tool-comparison.md, dated section; PR #162) and exp-194's standing cells: fdu 1.088 s, pdu --max-depth 2 1.061 s, pdu default 1.135 s, diskus 1.162 s, 12 pairs each. H159 itself was decided by exp-188..exp-190."
resolution: null
duplicate_of: null
---
Same host class as the 2026-09-28 Linux report. Screens only (hyperfine, strace -c, user/sys split, thread sweeps, allocator LD_PRELOAD, pdu depth/thread variants). Record figures in the research brief as screens, not claims.
