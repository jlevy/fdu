---
type: is
id: is-01m0ns0h494z78b7d42fhby8ge
title: "Prove parity: 129 goldens byte-identical, none regenerated"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies: []
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:21:10.152Z
updated_at: 2026-08-22T22:55:50.251Z
closed_at: 2026-08-22T22:55:20.565Z
close_reason: "crates/fdu-cli depends on fdu as an ordinary crate, so the boundary is enforced by the compiler: cli.rs has zero crate:: paths, the library has no cli module or feature and no clap or anyhow, and make lib-only fails if either returns. All 129 goldens byte-identical, none regenerated."
---
The corpus is the parity test and needs no new harness: scripts/run-golden.mjs already selects a surface by path, and the new binary builds to the same target/debug/fdu.

Byte-identical for all 129 is the pass condition. Zero differences is the right answer here, unlike the Python surface where zero would mean the shim never ran -- the difference is that this IS the command line rather than a second one impersonating it.

A regenerated golden in this work is a bug, not a result. If output changed, the move changed behaviour and the change is the finding.

Also run: make check with every feature combination it covers, including --no-default-features; and the Python parity harness, which links the library rather than the CLI and must be unaffected.
