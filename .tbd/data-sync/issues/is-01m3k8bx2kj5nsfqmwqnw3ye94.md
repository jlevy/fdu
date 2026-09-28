---
type: is
id: is-01m3k8bx2kj5nsfqmwqnw3ye94
title: Prototype a delta-only per-directory roll-up checkpoint log
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:53.650Z
updated_at: 2026-09-28T05:38:53.650Z
---
Rank 4. Walk capture appends only directories whose roll-up changed (closed under ancestors); key by volume UUID + firmlink-free path; per-path allocated measure; retain nlink. Measure capture cost vs flat snapshot at 450k and 1.5M entries, query time for now-vs-T, and daily state growth. Go if capture <= walk + 5%, query <= 0.2 s, growth bounded. Feeds fdu-8ybz.
