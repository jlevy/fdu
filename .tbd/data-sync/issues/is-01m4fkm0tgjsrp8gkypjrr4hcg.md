---
type: is
id: is-01m4fkm0tgjsrp8gkypjrr4hcg
title: "PR #189 C2: test_stability_pass assumes FDU_QA_LARGE unset; exported FDU_QA_* fail release-test in make check and release-rehearse"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m3tr05r13ydjt9hza3nksarb
hold: null
hold_until: null
created_at: 2026-10-09T05:54:18.063Z
updated_at: 2026-10-09T06:13:00.641Z
started_at: 2026-10-09T05:54:19.033Z
closed_at: 2026-10-09T06:13:00.640Z
close_reason: "fixed in f405067d (PR #189): Scratch sets aside FDU_QA_*, the pass's identity, and make's variables; AmbientEnvironmentTests runs the driver tests under a maintainer's exports; CI 37891213321"
resolution: null
duplicate_of: null
---
Coordinator-found: gate-check.log:5740 test_a_harness_that_exits_0_after_leaving_a_phase_out_is_skipped fails with FDU_QA_LARGE exported as the guide tells maintainers to. Fix: tests set aside FDU_QA_* and the pass's other environment inputs.
