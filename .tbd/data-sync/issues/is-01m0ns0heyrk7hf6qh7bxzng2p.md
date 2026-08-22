---
type: is
id: is-01m0ns0heyrk7hf6qh7bxzng2p
title: Update make check, CI, and release packaging for the new crate layout
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies: []
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:21:10.493Z
updated_at: 2026-08-22T22:55:20.572Z
closed_at: 2026-08-22T22:55:20.572Z
close_reason: "crates/fdu-cli depends on fdu as an ordinary crate, so the boundary is enforced by the compiler: cli.rs has zero crate:: paths, the library has no cli module or feature and no clap or anyhow, and make lib-only fails if either returns. All 129 goldens byte-identical, none regenerated."
---
Every target that names -p fdu for a CLI purpose, builds the binary, or assumes the workspace has one crate.

Known touch points: Makefile (build, test-golden's target/debug/fdu, lib-only, msrv, cross-lint), .github/workflows/ci.yml, crates/fdu-py (links the library -- verify it does not pick up the CLI), scripts/release, and cargo-deny if it enumerates crates.

The lib-only target matters most: it exists to exercise how library consumers build, and after this change that is the only way fdu builds.
