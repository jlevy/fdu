---
type: is
id: is-01m3kck52bs4kdtp3bmaa4n2gd
title: "watch: startup runs a ~37 s serial revalidation walk after a 7 s parallel cold scan"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T06:52:45.513Z
updated_at: 2026-09-28T06:52:45.513Z
---
Soak finding on a 476k-entry root: 60.5 s to the initial report = parallel cold scan ~7 s (21.5 CPU s) + snapshot save ~4 s + a serial revalidation walk ~37 s + rendering a 225 MB initial files view (RSS to 836 MB). Check whether the revalidation after a fresh cold scan is required by the observation-before-discovery handoff, and whether it can be parallel or scoped to the buffered hints.
