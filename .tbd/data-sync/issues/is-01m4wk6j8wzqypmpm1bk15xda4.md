---
type: is
id: is-01m4wk6j8wzqypmpm1bk15xda4
title: Release Python interpreter during native reports and watch startup
kind: task
status: in_progress
priority: 2
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T06:57:04.795Z
updated_at: 2026-10-14T16:57:55.407Z
started_at: 2026-10-14T16:22:59.739Z
---
Astra review A2: detach native report construction, rendering, and report_once pending.join; opened ReportProjection serializes/clones while attached. Preserve PyO3 borrow protections and native lock ordering. Add actual-boundary concurrency regressions; document synchronous asyncio adaptation. No stall durations measured. See Python/Rust architecture review dated 2026-10-13.

## Notes

Further source review: PyIndex.watch clones the retained inventory and calls Session::new while GIL-attached. Session::new performs watcher setup and initial reconciliation, so watch startup belongs in this responsiveness work as well as report building/rendering and pending-save joining. Preserve independent session semantics and PyO3 borrow protection. Source: crates/fdu-py/src/lib.rs watch and crates/fdu-core/src/watch_session.rs new_observed.
