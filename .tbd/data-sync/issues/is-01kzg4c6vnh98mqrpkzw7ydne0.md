---
type: is
id: is-01kzg4c6vnh98mqrpkzw7ydne0
title: "Publishing: crates.io, PyPI abi3 wheels, and a name re-verification gate"
kind: task
status: open
priority: 2
version: 8
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4d2fb96erw3h1b5k0c6xy
  - type: blocks
    target: is-01kzg4d32s8s6g47686dpk8ddk
  - type: blocks
    target: is-01kzm3v6nndedpwk414enwysv3
  - type: blocks
    target: is-01kzg4d256qmchmtyvttnpvn4y
  - type: blocks
    target: is-01kzg4d2saym31t884vf6me2p7
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:38.772Z
updated_at: 2026-08-14T21:19:42.062Z
---
Ship both artifacts from one workspace.
- crates.io: fdu, with cli as a default feature so 'cargo install fdu' just works. Library consumers write default-features = false; that trade-off is accepted and must be one documented line in the README.
- PyPI: abi3 wheels via maturin, one wheel per OS/arch. uv builds and consumes maturin projects natively.
- fdu-py stays publish = false on crates.io; it is a binding artifact, not a library.
- Release workflow, CHANGELOG discipline, and cargo-semver-checks on PRs once there is a public API worth promising.

RE-VERIFY NAME AVAILABILITY IMMEDIATELY BEFORE FIRST PUBLISH — availability is a race. Names were free on PyPI, crates.io (including similarity blockers f-du, f_du, fd-u, fd_u, FDU), and Homebrew as of 2026-08-07.

Methodological trap for whoever re-checks: https://pypi.org/project/<name>/ can return HTTP 200 with an anti-bot interstitial (<title>Client Challenge</title>) for names that do not exist. Use the Simple index (PEP 503) or the JSON API, and calibrate against a known-present package, or the check reports false positives.

Two prior uses of the name exist, neither blocking: an npm package 'fdu' (disk usage flame graph, last published 2022) and a dormant GitHub script nicollet/fdu. Neither is on PyPI, crates.io, or Homebrew.

## Notes

The 2026-08-09 Rust release audit found that cargo package --list -p fdu omits the license file. Before first publish, inspect and smoke-test the packaged crate and every native artifact, include expected license/readme content, define one tag/version identity, use least-privilege trusted publishing, and document supported platforms, MSRV-change policy, deprecation policy, security reporting, rerun behavior, and incident recovery. This bead is blocked on cool-off-clean tooling, the pinned compatibility matrix, the minimal documented Rust API, and the lossless Python boundary.
