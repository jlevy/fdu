---
type: is
id: is-01m392b6spkxq6mc5at92byx58
title: Watch start shows no progress while its first report is built
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies: []
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-24T06:41:15.061Z
updated_at: 2026-09-30T10:03:34.466Z
closed_at: 2026-09-30T10:03:34.465Z
close_reason: "Fixed in 6d6814bb. fdu --watch stopped its progress line when Session::start returned, before the first answer was built; run_watch now starts the session and builds the first answer under one ticker (Cli::start_watch) and stops the line once the answer exists, before the startup save's warning or the answer is written, as the plan's Watch bullet says. Engine entry point Session::changed_report_with_progress (public, additive) enters Summarizing as the one-shot build does. Tests: watch_session::tests::the_first_answer_is_built_under_the_summarizing_phase and cli::tests::a_watch_start_draws_through_its_first_answer_and_stops_before_writing_it; clippy clean; CHANGELOG Fixed entry and plan entry-point paragraph updated."
resolution: null
duplicate_of: null
---
Delta review E-2 of 2f210706: fdu --watch stops the progress line when Session::start_with_progress returns, before session.report() builds the first answer (crates/fdu/src/cli.rs run_watch). For heavy views (--view full) on a large tree that build takes seconds with no line. Either give the watch's first report build the Summarizing phase through an engine entry point, or correct the plan's Watch bullet (it says the line stops when the first report paints).
