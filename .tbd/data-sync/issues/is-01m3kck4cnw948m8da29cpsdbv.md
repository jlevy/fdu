---
type: is
id: is-01m3kck4cnw948m8da29cpsdbv
title: "watch: persistence rewrites the whole snapshot every interval while dirty"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T06:52:44.821Z
updated_at: 2026-09-28T06:52:44.821Z
---
Soak finding: with --interval 10s on a busy 476k-entry root the watch persisted 187 full snapshot rewrites (36.7 MB each, 77 B/entry) in 61 min: 6.9 GB written per hour (watch_session.rs persists after each batch/idle timeout when changes are pending, throttled only by --interval). Unchanged on the #139 branch (--watch still persists under --cache auto). A render interval should not set the durability cadence; persist on a separate longer cadence or append deltas (see the delta-only checkpoint store, fdu-uq1y). Especially wrong for a low-space diagnostic (fdu-hbjp).
