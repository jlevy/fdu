---
type: is
id: is-01m3k8bwjpa8gbrk4ev4wh6m8f
title: "Experiment: home-filter FSEvents replay cost on the internal Data volume"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:53.141Z
updated_at: 2026-09-28T05:38:53.141Z
---
Rank 3. Replay 1 h and 24 h cursors with the home folder as the path filter (directory events and FileEvents) to size the ~10 us per-matching-record term. Go if replay + relist <= 25% of a home walk. Budget rule: 0.124 s/MB x journal MB behind cursor + 10 us x matching records, x2 for contention.
