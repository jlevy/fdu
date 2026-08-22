---
type: is
id: is-01m0nrzvs7hkqmgn3wmxh303zx
title: Create crates/fdu-cli and move the binary target to it
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0nrzw3n05db3memm9sg4hwh
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:20:48.294Z
updated_at: 2026-08-22T22:55:20.531Z
closed_at: 2026-08-22T22:55:20.512Z
close_reason: "crates/fdu-cli depends on fdu as an ordinary crate, so the boundary is enforced by the compiler: cli.rs has zero crate:: paths, the library has no cli module or feature and no clap or anyhow, and make lib-only fails if either returns. All 129 goldens byte-identical, none regenerated."
---
The crate boundary is the whole mechanism: inside one crate, 'the CLI invents nothing' is a claim a reviewer checks; across a crate boundary it is a fact the compiler checks on every build, and any future private need becomes a visible act of making something public.

crates/fdu-cli/
  Cargo.toml    depends on fdu; owns clap and anyhow, which move out of the library
  src/main.rs   the entry point the fdu binary builds from

The binary stays named fdu. The library keeps no [[bin]].

Files: new crate; crates/fdu/Cargo.toml loses clap, anyhow, and its [[bin]]; workspace members gain the crate.
