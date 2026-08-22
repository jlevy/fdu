---
type: is
id: is-01m0nrzw3n05db3memm9sg4hwh
title: "Move cli.rs into the new crate, rewriting crate:: to fdu::"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0nrzwe28fz1esdk2nnxd0eg
  - type: blocks
    target: is-01m0ns0gsj8g96exsc3ndt45bg
  - type: blocks
    target: is-01m0ns0h494z78b7d42fhby8ge
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:20:48.628Z
updated_at: 2026-08-22T22:55:20.545Z
closed_at: 2026-08-22T22:55:20.545Z
close_reason: "crates/fdu-cli depends on fdu as an ordinary crate, so the boundary is enforced by the compiler: cli.rs has zero crate:: paths, the library has no cli module or feature and no clap or anyhow, and make lib-only fails if either returns. All 129 goldens byte-identical, none regenerated."
---
A move, not a rewrite. Every crate:: path becomes fdu::, and the compiler enumerates anything that does not resolve -- which is the audit, run for real rather than by grep.

The audit predicts three failures, all handled by fdu-6hr2 and fdu-fmn1: human_bytes, human_count, prepare_report_with_scan_diagnostics. Everything else already resolves to a public item. If the compiler names a fourth, that is a finding worth recording rather than a nuisance to route around.

Do NOT split cli.rs into modules here. Worth doing, but it would hide the move inside a refactor and make the diff unreviewable.

Files: crates/fdu/src/cli.rs -> crates/fdu-cli/src/cli.rs.
