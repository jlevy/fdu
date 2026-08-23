---
type: is
id: is-01m0qs197189wae43fmqxs82bs
title: Counter relations as a golden-visible cost oracle
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T17:00:03.681Z
updated_at: 2026-08-23T17:00:03.681Z
---
counters.rs already records twenty counters behind FDU_COUNTERS=1, and FDU_SCAN_DIAGNOSTICS already demonstrates the emission pattern: a versioned payload on stderr behind the __FDU_SCAN_DIAGNOSTICS__= sentinel, outside the report envelope, tested in crates/fdu/tests/cli_exit.rs. Counters give an axis no golden covers today — not what a run answered but what work it did. Absolute counts are platform-dependent; RELATIONS are not: stats == entries on a cold walk; stats == 0 under --cache only; an idle watch does zero filesystem work (a property the design principles already say is 'asserted by test rather than described'); a read concurrent with a write triggers no rescan. These catch the regression where output is unchanged but cost exploded. Never absolute, never wall-clock — a timing gate on a shared runner measures the runner.
