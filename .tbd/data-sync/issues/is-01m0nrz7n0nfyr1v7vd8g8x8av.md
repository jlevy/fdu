---
type: is
id: is-01m0nrz7n0nfyr1v7vd8g8x8av
title: Ungate report_format and prepare_report from the cli feature
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0nrzvs7hkqmgn3wmxh303zx
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:20:27.679Z
updated_at: 2026-08-22T22:55:50.251Z
closed_at: 2026-08-22T22:31:17.238Z
close_reason: human_count moved into report_format (the library was calling into the CLI to format a number), human_bytes promoted, prepare_report_with_scan_diagnostics promoted as a real capability, and report_format ungated by depending on anstyle directly instead of taking its colour types from clap. clap is now absent from library builds and a library consumer can render. 129 goldens unchanged.
---
A consumer taking default-features = false cannot render a report at all: report_format is behind the cli feature, so the library can produce a Report and not print it, which is half an API. This already bit once on this branch -- a display note called a function configured out under --no-default-features and broke the library build.

prepare_report is gated with it for the same reason (fdu-z7sp). One-shot planning is an execution strategy, not a front end.

Neither gate survives the move anyway: once the CLI is a separate crate, a 'cli' feature on the library gates nothing the library itself uses.

Files: crates/fdu/src/lib.rs:71-72 and 98, crates/fdu/Cargo.toml [features].
Verify: cargo check --no-default-features, and a scratch consumer that renders a report without the cli feature.
