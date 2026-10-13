---
type: is
id: is-01m4t5mgqyj857ap8je2mzsbss
title: Audit and remove completed fdu worktrees and generated data
kind: chore
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-13T08:21:33.050Z
updated_at: 2026-10-13T08:22:01.088Z
started_at: 2026-10-13T08:21:54.543Z
closed_at: 2026-10-13T08:22:01.087Z
close_reason: null
resolution: null
duplicate_of: null
---
Completed user-authorized permanent cleanup after fresh remote and activity checks: six clean merged external worktrees, seven reproducible environment/cache directories, and 195 fdu test/cache directories. Preserved primary source, tbd worktree, release/media evidence, installed fdu 0.5.0, and the small performance checkout with two unpushed commits. Approximately 1.3 GiB allocated removed this pass; external free space about 12 GiB. No other-project data removed. Detailed private manifests retained outside scratch.
