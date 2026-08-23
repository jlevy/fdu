---
type: is
id: is-01m0nvz7j00dm6w1snefmvw713
title: "Carry out the rename: fdu-core for the engine, fdu for the installed package"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/done/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T23:12:53.311Z
updated_at: 2026-08-23T00:05:56.691Z
closed_at: 2026-08-22T23:52:58.467Z
close_reason: "fdu is the package a user installs (bin plus a lib re-exporting the engine); fdu-core is the engine. Follows the dominant Rust convention -- the name a user types belongs to what they install, as with ripgrep, gitoxide, and ruff. Enforcement is unchanged: the command line depends on the engine as an external crate, so the compiler still decides whether the CLI invents anything. 129 goldens byte-identical."
---
Mechanical but wide. crates/fdu becomes crates/fdu-core with package name fdu-core; crates/fdu-cli becomes crates/fdu with package name fdu, keeping its bin named fdu and adding a lib that re-exports the engine plus run_process.

Touches roughly 111 fdu:: import sites across fdu-cli and fdu-py, both manifests, the workspace members, Makefile targets that name -p fdu, the CI build step, cargo-deny if it enumerates crates, and the README.

The proof is unchanged and must stay green: 129 goldens byte-identical, and make lib-only still failing if clap or anyhow reaches the engine.
