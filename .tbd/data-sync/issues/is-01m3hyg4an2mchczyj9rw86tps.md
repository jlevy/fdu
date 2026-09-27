---
type: is
id: is-01m3hyg4an2mchczyj9rw86tps
title: Clean completed chat build outputs while preserving release evidence
kind: task
status: closed
priority: 2
version: 3
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-27T17:27:11.945Z
updated_at: 2026-09-27T17:29:33.774Z
started_at: 2026-09-27T17:29:30.873Z
closed_at: 2026-09-27T17:29:33.770Z
close_reason: Staged three inactive generated debug directories from completed chat builds using trash only. Individually measured allocated sizes total 22.61 GiB. No matching processes or open handles; preserved all source, Git state, release packages/wheels, QA/research evidence, environments/shared caches and other tasks builds. Exact source-to-Trash mapping and before/after df are in /Volumes/spud-ext1/agent-scratch/fdu-cleanup-20260927/manifest.json. Trash left unemptied; staging does not reclaim physical space.
resolution: null
duplicate_of: null
---
User-authorized cleanup of inactive generated Cargo targets from completed release and PR gates. Audit processes and open files; preserve source, history, release artifacts, QA/research logs, active/shared caches and other tasks. Use Trash only and retain exact path/size/destination manifest in external scratch.
