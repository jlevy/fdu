---
type: is
id: is-01m3k8by8ghsmwr4mjxfykq0p8
title: Test whether fseventsd keeps processing abandoned replays
kind: task
status: closed
priority: 3
version: 2
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:54.862Z
updated_at: 2026-09-28T05:41:55.204Z
closed_at: 2026-09-28T05:41:55.201Z
close_reason: "Done in the 2026-09-27 change-source review: abandoning an 8 h replay after 1 s (clean stop, _exit, or SIGKILL) left fseventsd idle within ~1 s (+0.09-0.18 s CPU over the next 21 s), and the next 15-min replay ran at baseline (0.56-0.64 s vs 0.47-0.61 s). No backlog. Variance came from contention: concurrent replays on the same volume ran 1.6x slower, cross-volume replays 2x, and host state alone 1.8x."
resolution: null
duplicate_of: null
---
Rank 6. Start a long (8-12 h) replay, abandon it (clean stop and SIGKILL variants), then time a short replay and watch fseventsd CPU. Explains the 32 s -> 92.5 s day-old variance and decides whether deadlines must cancel cleanly.
