---
type: is
id: is-01m01mrd8z9d4chrsv4frjbav4
title: "H91: bound the observation channel; measure queue occupancy first"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
dependencies: []
created_at: 2026-08-15T02:42:01.118Z
updated_at: 2026-08-15T02:42:01.118Z
---
std::sync::mpsc is unbounded while producers outrun the consumer ~4x on Linux, so queued Observation batches are an unmeasured store at peak RSS. Step 1: peak-occupancy counter (FDU_COUNTERS). Step 2: bounded channel if occupancy is a measurable RSS share. Pre-registered signal: peak_rss_bytes down on 450k+ with wall unchanged; superseded entirely if H86 removes the channel.
