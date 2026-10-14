---
type: is
id: is-01m4ws23t454s8tnh8gpm3c1v7
title: Measure and reduce full-index copying in legacy watch reports
kind: task
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T08:39:30.365Z
updated_at: 2026-10-14T16:18:24.132Z
started_at: 2026-10-14T16:18:24.125Z
---
Copy audit for PR205: Index.watch clones native Index into independent IndexHandle; Session.report calls IndexHandle.snapshot (Index.clone) before projection, even for small reports. persist_due may snapshot for safe persistence. Measure startup, repeated report, and persistence clone time/RSS against OpenedIndex observe/read. Reduce avoidable full-index clones through coherent native projection or shared immutable ownership where justified, preserving independent watcher lifecycle, report snapshot identity, concurrency/lock ordering, and stable persistence semantics. Do not describe retaining a Rust handle as eliminating all native copies.
