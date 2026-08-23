---
type: is
id: is-01m01mred1c08prxh1nk3m6fw6
title: Probe --no-oracle mode and engine-scoped counters
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-15T02:42:02.273Z
updated_at: 2026-08-23T09:09:38.194Z
---
Phase 0 instrument from the campaign-2 plan (cited there as fdu-9ydj, which was a duplicate and is closed). A --no-oracle probe mode and engine-phase counter scoping so attribution runs stop counting the harness: the oracle is ~39% of probe instructions and 46% of its allocation events. Platform-neutral; runnable on macOS.
