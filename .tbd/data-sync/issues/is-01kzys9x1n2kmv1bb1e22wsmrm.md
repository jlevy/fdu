---
type: is
id: is-01kzys9x1n2kmv1bb1e22wsmrm
title: Controlled-cold regime for the tool-comparison harness
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T00:03:45.332Z
updated_at: 2026-08-14T00:03:45.332Z
---
compare_tools.py hardcodes os_cache: warm-steady and requires warmups, so it cannot produce controlled-cold product evidence. measure.py already has --purge (sync + drop_caches on Linux). Controlled-cold is the one regime where Linux measurement found fdu behind (diskus 22.8% ahead), so the open Linux product question is currently unmeasurable by the release-evidence harness.
