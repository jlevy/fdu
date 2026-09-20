---
type: is
id: is-01m2y63j1zr2ybptec39sgwe9r
title: "PR #91 review S1: describe rebuild_rollups without a strict linear bound"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2y58e5s7yzpstkcqmheymy5
created_at: 2026-09-20T01:15:19.997Z
updated_at: 2026-09-20T01:15:26.222Z
closed_at: 2026-09-20T01:15:26.220Z
close_reason: rebuild_rollups rustdoc no longer claims a strict O(files + dirs) bound.
resolution: null
duplicate_of: null
---
content_index.rs rustdoc promised O(files + dirs); rebuild still sorts directories and walks ancestors. Describe removal of repeated per-file propagation without claiming a strict linear bound.
