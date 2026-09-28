---
type: is
id: is-01m3k8by8ghsmwr4mjxfykq0p8
title: Test whether fseventsd keeps processing abandoned replays
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:54.862Z
updated_at: 2026-09-28T05:38:54.862Z
---
Rank 6. Start a long (8-12 h) replay, abandon it (clean stop and SIGKILL variants), then time a short replay and watch fseventsd CPU. Explains the 32 s -> 92.5 s day-old variance and decides whether deadlines must cancel cleanly.
