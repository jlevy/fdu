---
type: is
id: is-01m3kck5jdzmkjbk9zzmb2pfkk
title: Design a lightweight resident dirty-directory recorder for whole-home scopes
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T06:52:46.028Z
updated_at: 2026-09-28T06:52:46.028Z
---
Soak finding: over one hour the busy root touched only 530 distinct parent directories (3,372 paths) plus ~70 open-for-write files per sample: tens of KB of state, versus a 476k-entry resident index (190 B/entry floor; ~1.3 GB for a 6.7M-entry home). An event-only watcher would miss 99.8% of in-place growth bytes (held-open files); re-stat of the libproc writer list recovers all of it. Design: live FSEvents (inotify on Linux) -> durable dirty-directory log + cursor; writer-list re-stat at checkpoint time; relist dirty dirs into the delta-only checkpoint store (fdu-uq1y); downtime recovered by replaying from the last applied event id under the budget rule, else a sweep. Requires an explicit decision to allow background monitoring.
