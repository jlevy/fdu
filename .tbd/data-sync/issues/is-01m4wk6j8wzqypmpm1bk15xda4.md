---
type: is
id: is-01m4wk6j8wzqypmpm1bk15xda4
title: Release Python interpreter during native reports and watch startup
kind: task
status: open
priority: 2
version: 5
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T06:57:04.795Z
updated_at: 2026-10-14T16:58:16.197Z
started_at: 2026-10-14T16:22:59.739Z
---
Astra review A2: detach native report construction, rendering, and report_once pending.join; opened ReportProjection serializes/clones while attached. Preserve PyO3 borrow protections and native lock ordering. Add actual-boundary concurrency regressions; document synchronous asyncio adaptation. No stall durations measured. See Python/Rust architecture review dated 2026-10-13.

## Notes

PyIndex.watch clones retained inventory and calls Session::new while attached, including watcher setup and initial reconciliation. Also audit implicit final-reference destruction of OpenedIndex: explicit close detaches, but OpenedState::drop calls shutdown and joins workers, so ordinary Python destruction may block while attached. No deadlock or duration measured. Preserve independent session semantics, PyO3 borrow protection and native lock ordering; cover report work, startup, explicit close and implicit destruction with actual-boundary concurrency regressions.
