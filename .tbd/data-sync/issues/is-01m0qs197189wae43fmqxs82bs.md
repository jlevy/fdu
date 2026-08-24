---
type: is
id: is-01m0qs197189wae43fmqxs82bs
title: Counter relations as a golden-visible cost oracle
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T17:00:03.681Z
updated_at: 2026-08-23T22:25:09.527Z
closed_at: 2026-08-23T22:25:09.527Z
close_reason: "Counts::to_json plus a __FDU_COUNTERS__= payload on stderr, following FDU_SCAN_DIAGNOSTICS's pattern: versioned, outside the report envelope. A new cli-cost golden asserts relations rather than absolutes — stats equal entries plus the root on a cold walk, dir_opens equal directories plus the root, cache-only touches nothing at all (checked in syscalls rather than by reading source off the report), and analysis is the only thing that opens file bodies while walking identically. Never wall-clock. The package reports no counters because the allocator tier is process-global and an extension cannot install one for CPython; the scripts emit one stable line saying so and the parity harness records it as the process-instrumentation class."
resolution: null
duplicate_of: null
---
counters.rs already records twenty counters behind FDU_COUNTERS=1, and FDU_SCAN_DIAGNOSTICS already demonstrates the emission pattern: a versioned payload on stderr behind the __FDU_SCAN_DIAGNOSTICS__= sentinel, outside the report envelope, tested in crates/fdu/tests/cli_exit.rs. Counters give an axis no golden covers today — not what a run answered but what work it did. Absolute counts are platform-dependent; RELATIONS are not: stats == entries on a cold walk; stats == 0 under --cache only; an idle watch does zero filesystem work (a property the design principles already say is 'asserted by test rather than described'); a read concurrent with a write triggers no rescan. These catch the regression where output is unchanged but cost exploded. Never absolute, never wall-clock — a timing gate on a shared runner measures the runner.
