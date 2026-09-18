---
type: is
id: is-01m2s0txht8tn9whp5ttcnr31n
title: Simulate a new-user install from packaged artifacts before publish
kind: task
status: closed
priority: 0
version: 6
labels:
  - release
  - testing
dependencies: []
parent_id: is-01m2s0tq4ppsygrs129nw1m86n
created_at: 2026-09-18T01:07:01.818Z
updated_at: 2026-09-18T01:10:25.868Z
closed_at: 2026-09-18T01:10:25.867Z
close_reason: Packaged-artifact stranger path exercised on 98379c76; results in bead notes. Watch/Python/Rust/CLI succeeded; wheel SIGINT and crates.io README remain known blockers.
---
Exercise cargo package + host wheel the way a stranger would after crates.io/PyPI exist: install the packaged CLI, run help/docs/tree/watch, import the Python module, and compile a small Rust consumer. Record gaps as beads. Does not contact either registry.

## Notes

Pre-publish new-user simulation on origin/main 98379c76 (2026-09-18), packaged artifacts only.

Passed:
- cargo package -p fdu-core -p fdu; smoke_crate.py install reports fdu 0.1.0
- fdu --help, --docs, default tree, --view summary
- host abi3 wheel via uv tool install --no-index; same version
- import fdu; open(); report schema fdu.report/5; complete True
- path-crate Rust consumer: open + total()
- crate --watch --view files --format jsonl --interval 1s: fdu.stream/1 upsert for a created file; SIGINT exits
- Python index.watch() saw the created file

Failed / confirmed known:
- --interval 0.2 / 0.2s / 200ms are usage errors (new bead)
- uv-tool wheel --watch ignores SIGINT (fdu-18vk reproduced)
- crates/fdu readme = ../../README.md; root README has ~20 relative links that crates.io would resolve under crates/fdu (fdu-i142 / #77)

Not simulated: manylinux2014 / macOS / Windows wheels (host wheel only), cargo install from crates.io, uvx from PyPI, docs.rs, GitHub release assets.
