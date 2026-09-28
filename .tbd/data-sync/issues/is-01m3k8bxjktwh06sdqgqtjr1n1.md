---
type: is
id: is-01m3k8bxjktwh06sdqgqtjr1n1
title: Measure writer-list coverage over three live refreshes
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:54.161Z
updated_at: 2026-09-28T05:38:54.161Z
---
Rank 5 (see fdu-vhrb). For each file changed between refreshes on a live agent root, classify as in the event set E, the libproc open-writer set W, or neither. Go if >= 99% in E u W and the remainder is attributable to other users' processes. Ordering: replay to HistoryDone, then enumerate writers, re-stat E u W, next cursor = HistoryDone id.
