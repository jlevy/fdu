---
type: is
id: is-01m0nrz70pggw1yya4tkhh40t3
title: Promote the formatting helpers the CLI needs
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0nrzvs7hkqmgn3wmxh303zx
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:20:27.029Z
updated_at: 2026-08-22T22:31:17.214Z
closed_at: 2026-08-22T22:31:17.214Z
close_reason: human_count moved into report_format (the library was calling into the CLI to format a number), human_bytes promoted, prepare_report_with_scan_diagnostics promoted as a real capability, and report_format ungated by depending on anstyle directly instead of taking its colour types from clap. clap is now absent from library builds and a library consumer can render. 129 goldens unchanged.
---
report_format::human_bytes and human_count are pub(crate) and used in production by cli.rs. A caller formatting fdu's numbers should not reimplement its unit rules -- that is how two spellings of the same quantity appear in one tool.

Promote both, with doc comments saying why a caller wants them rather than merely that they exist.

Files: crates/fdu/src/report_format.rs (both fns), crates/fdu/src/lib.rs if re-exported.
Verify: cargo check --no-default-features still clean.
