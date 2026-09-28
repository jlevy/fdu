---
type: is
id: is-01m3k8bys7tcybgmse3kdkjq28
title: Diagnose FSEventStreamStart failure for multi-path device-relative streams
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:55.398Z
updated_at: 2026-09-28T05:38:55.398Z
---
Rank 7. A device-relative stream with four relative paths returned false from FSEventStreamStart (2/2) while each path alone starts. Needed to pay one journal scan for several roots.
