---
type: is
id: is-01m3k8bv5dqnr0phk1a24we46n
title: "Experiment: APFS dir-stats pruned refresh at scale vs walk oracle"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:51.691Z
updated_at: 2026-09-28T05:38:51.691Z
---
Rank 1 in the review. Go if: zero oracle misses across randomized workloads incl. open writers; refresh <=10% of walk at >=200k entries; write overhead <=10% at realistic depth; clean removal path; works unprivileged on the internal volume. Private marking API (apfs.util -M / fsctl 0xC1104A71) and persistent inherited flag are the risks; owned fixtures and disk images only.
