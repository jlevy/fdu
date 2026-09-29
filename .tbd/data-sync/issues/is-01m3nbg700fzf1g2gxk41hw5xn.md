---
type: is
id: is-01m3nbg700fzf1g2gxk41hw5xn
title: Resolve exp-167's in-progress verdict now that exp-190 decided H159 on Linux
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-29T01:12:09.471Z
updated_at: 2026-09-29T06:05:31.245Z
closed_at: 2026-09-29T06:05:31.245Z
close_reason: "Resolved in cb968b78, PR #158 (https://github.com/jlevy/fdu/pull/158)."
resolution: null
duplicate_of: null
---
exp-167 still reads in-progress, waiting on the Linux decision exp-190 has since made (H159 accepted on node_modules-dense, 8.5 entries/dir). Decide appended resolution vs decision change; regenerate ledger/report. From the evidence-report refresh (672c2188).
