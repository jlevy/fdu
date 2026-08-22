---
type: is
id: is-01m0nv9134ddskjzyam5v3hjx0
title: Decide the published crate names before fdu ships to crates.io
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T23:00:45.795Z
updated_at: 2026-08-22T23:01:23.177Z
---
Moving the command line into crates/fdu-cli changed what a user types to install it, and
that decision should be made deliberately rather than inherited from a refactor.

Today: fdu is the library and publishes no binary; fdu-cli carries the binary, which is
still named fdu. So the install command becomes "cargo install fdu-cli" while the program
is fdu.

That is a common Rust layout and it is defensible, but it is surprising: a user who knows
the tool is called fdu will type "cargo install fdu" and get a library with no binary --
which is exactly the error the README produced until this was fixed.

Two ends, both coherent:

1. Keep it. fdu is the library, fdu-cli installs the command. Document it once, in the
   README and the release notes, and accept that "cargo install fdu" fails with cargo's
   own "has no binaries" message, which is at least a clear error rather than a wrong
   result.

2. Swap the names. The binary crate takes fdu and the library becomes fdu-core or
   similar, so "cargo install fdu" works as a user expects. Costs a library rename before
   anything is published, which is the cheapest moment it will ever be.

Nothing is published yet, so this is free to decide now and expensive later: crates.io
names are permanent, and a published fdu library cannot become a binary crate.

Also update scripts/release/registry_state.py, which checks crates.io for fdu alone and
would not notice fdu-cli missing, and the release packaging spec, whose goal line says
"make cargo install fdu ... expose the same native command-line contract".
