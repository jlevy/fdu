---
type: is
id: is-01m3gqwpvr0n844y9pj0gj3cwj
title: Avoid duplicate aggregate watch output after invisible metadata changes
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-27T06:12:29.678Z
updated_at: 2026-09-27T06:12:29.678Z
---
Observed on macOS at c5e2f6ed: run fdu on a one-file fixture with --watch --cache off --interval 200ms --color never. Idle produces no output. Touching the file without changing its 4096-byte size emits a new timestamp separator and the identical 4.0 KiB / 1 file tree. Growing it to 16384 bytes correctly emits 16 KiB. Session::next_batch marks any effective commit dirty before selection filtering; CLI run_watch reevaluates and writes the report for every dirty batch. Desired behavior: preserve native-event processing and index correctness, suppress unchanged aggregate presentation, and investigate query-aware invalidation to avoid unnecessary evaluation. Define output identity explicitly without volatile generated timestamps; retain meaningful tree-status, freshness, and invalidation changes and never deduplicate required file change-stream records. Add tests for idle, mtime-only default-tree changes, filtered changes, visible changes, and invalidation. This is a diagnosis only; no watch code changed in PR #132.
