---
type: is
id: is-01m392b6spkxq6mc5at92byx58
title: Watch start shows no progress while its first report is built
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies: []
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-24T06:41:15.061Z
updated_at: 2026-09-24T06:41:15.061Z
---
Delta review E-2 of 2f210706: fdu --watch stops the progress line when Session::start_with_progress returns, before session.report() builds the first answer (crates/fdu/src/cli.rs run_watch). For heavy views (--view full) on a large tree that build takes seconds with no line. Either give the watch's first report build the Summarizing phase through an engine entry point, or correct the plan's Watch bullet (it says the line stops when the first report paints).
