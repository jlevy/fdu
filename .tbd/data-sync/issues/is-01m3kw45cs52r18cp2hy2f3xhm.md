---
type: is
id: is-01m3kw45cs52r18cp2hy2f3xhm
title: "watch: skip the case-only membership listing on case-sensitive volumes"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T11:24:11.544Z
updated_at: 2026-09-28T11:24:11.544Z
---
fdu-822y review finding 4: the parent-listing membership check runs for every present renamed name, including sticky-flag writes (watch.rs:1040, 1180-1182); a 100k-entry flat directory with constant atomic saves is listed every <=1.6 s. Probe case sensitivity once per root and skip the listing where it cannot matter. Also add tests: rename chain, rename-over-existing, unlistable parent (None branch), sticky flag on a directory; entries() should compare file mtime.
