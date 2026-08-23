---
type: is
id: is-01m0nv9134ddskjzyam5v3hjx0
title: "Rename: fdu is the installed package, fdu-core is the engine"
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/done/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0nvz7j00dm6w1snefmvw713
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T23:00:45.795Z
updated_at: 2026-08-23T00:05:56.691Z
closed_at: 2026-08-22T23:52:58.481Z
close_reason: "fdu is the package a user installs (bin plus a lib re-exporting the engine); fdu-core is the engine. Follows the dominant Rust convention -- the name a user types belongs to what they install, as with ripgrep, gitoxide, and ruff. Enforcement is unchanged: the command line depends on the engine as an external crate, so the compiler still decides whether the CLI invents anything. 129 goldens byte-identical."
---
DECIDED: the package a user installs is named fdu; the engine becomes fdu-core.

What other Rust tools do, and why this follows them:

The dominant pattern for a tool that is mostly a CLI is ONE package that is both a
library and a binary, with the CLI's dependencies feature-gated -- bat, tokei, mdbook,
just, cbindgen all do this. That is what fdu was, and it is why "cargo install fdu"
worked.

Where a project has a substantial engine AND a CLI, the split is usually by role rather
than by suffix, and the name a user types belongs to the thing they install: ripgrep is
the binary while the engine is grep-searcher and friends; gitoxide is the binary while
the library is gix; ruff is the binary while the engine is ruff_* crates.

What is rare is publishing the library as the headline name and making users install
something else. "cargo install fdu" returning "there is nothing to install ... it has no
binaries" is a bad first contact with the tool, and that is exactly what the previous
layout produced.

The shape:

  fdu-core   the engine. Every type and function the API offers.
  fdu        bin + lib. The binary is the command. The lib re-exports fdu-core and
             run_process, so "cargo add fdu" still gives a library user the whole API and
             docs.rs/fdu is a useful page rather than a stub.
  fdu-py     the Python binding, unchanged in role.

Two packages, not more. The user's suggestion of fdu-rs was tempting for symmetry with
fdu-py, but on crates.io everything is Rust, so the suffix carries no information there;
fdu-core says what the crate is rather than what language it is in.

Why not one package with the CLI in the binary target: verified that a bin target sees
its own package's lib as an external crate, so private items are unreachable and the
boundary would hold. But fdu-py compiles run_process INTO the Python extension module, so
the command line has to be reachable as a library, and a bin-target-only CLI is not.

This keeps every property the split was for: the command line depends on the engine the
way any consumer does, so the compiler still decides whether "the CLI invents nothing" is
true on every build.
