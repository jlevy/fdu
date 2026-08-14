---
type: is
id: is-01m0129stsv3wn03hy6m3j4r4t
title: Add clean-consumer, CLI parity, typing, and downstream acceptance gates
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0129t2wsdsv20mt3bq7s0zh
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:19:28.088Z
updated_at: 2026-08-14T21:19:41.654Z
---
Test only extracted and installed artifacts outside the workspace: no-default Rust consumption, cargo install, wheel and sdist installs, uvx, strict BasedPyright, runtime/stub exports, Cargo-versus-wheel CLI parity, complete/stale and partial-result semantics, reusable multi-view queries, provenance, deltas, watch cleanup, native paths, and a representative downstream roll-up transformation.
