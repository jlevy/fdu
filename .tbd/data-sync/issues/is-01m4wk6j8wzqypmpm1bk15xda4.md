---
type: is
id: is-01m4wk6j8wzqypmpm1bk15xda4
title: Release Python interpreter during native report work
kind: task
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T06:57:04.795Z
updated_at: 2026-10-14T16:22:59.742Z
started_at: 2026-10-14T16:22:59.739Z
---
Astra review A2: detach native report construction, rendering, and report_once pending.join; opened ReportProjection serializes/clones while attached. Preserve PyO3 borrow protections and native lock ordering. Add actual-boundary concurrency regressions; document synchronous asyncio adaptation. No stall durations measured. See Python/Rust architecture review dated 2026-10-13.
