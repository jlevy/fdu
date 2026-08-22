---
type: is
id: is-01m0ns0gsj8g96exsc3ndt45bg
title: Delete fdu::cli and the cli feature
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0ns0heyrk7hf6qh7bxzng2p
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:21:09.809Z
updated_at: 2026-08-22T22:55:50.251Z
closed_at: 2026-08-22T22:55:20.559Z
close_reason: "crates/fdu-cli depends on fdu as an ordinary crate, so the boundary is enforced by the compiler: cli.rs has zero crate:: paths, the library has no cli module or feature and no clap or anyhow, and make lib-only fails if either returns. All 129 goldens byte-identical, none regenerated."
---
The point of the move is that the old one goes. Two copies of the command line is the thing this work exists to prevent, and leaving the module behind 'for now' guarantees they diverge.

The cli feature gated clap, anyhow, and the [[bin]]; all three now live in the new crate, so the feature gates nothing the library uses and its removal is mechanical.

Verify: cargo tree -p fdu shows no clap and no anyhow. That is the check that the dependency actually left rather than moving to a different line of the same manifest.

Files: crates/fdu/src/lib.rs (mod cli), crates/fdu/Cargo.toml [features].
