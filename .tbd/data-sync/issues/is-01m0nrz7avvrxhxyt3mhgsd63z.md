---
type: is
id: is-01m0nrz7avvrxhxyt3mhgsd63z
title: "Decide prepare_report_with_scan_diagnostics: promote or relocate"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies: []
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:20:27.354Z
updated_at: 2026-08-22T22:55:50.251Z
closed_at: 2026-08-22T22:31:17.230Z
close_reason: human_count moved into report_format (the library was calling into the CLI to format a number), human_bytes promoted, prepare_report_with_scan_diagnostics promoted as a real capability, and report_format ungated by depending on anstyle directly instead of taking its colour types from clap. clap is now absent from library builds and a library consumer can render. 129 goldens unchanged.
---
The third and last production dependency cli.rs has on a non-public item. It exists for repository-controlled installed-command measurement, and its doc says so: 'deliberately separate from prepare_report so ordinary CLI and library callers incur neither trace collection nor serialization work'.

So it is not obviously general API. Two defensible ends: promote it beside prepare_report, or keep it internal and move its single caller (the SCAN_DIAGNOSTICS_PREFIX path in cli.rs) behind whatever mechanism the measurement harness actually needs.

Decide deliberately and record the reason; do not promote it just to make the move compile.

Files: crates/fdu/src/execution.rs:191, crates/fdu/src/cli.rs:578.
