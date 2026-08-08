---
type: is
id: is-01kzg4c6vnh98mqrpkzw7ydne0
title: "Publishing: crates.io, PyPI abi3 wheels, and a name re-verification gate"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:38.772Z
updated_at: 2026-08-08T07:28:38.772Z
---
Ship both artifacts from one workspace.
- crates.io: fdu, with cli as a default feature so 'cargo install fdu' just works. Library consumers write default-features = false; that trade-off is accepted and must be one documented line in the README.
- PyPI: abi3 wheels via maturin, one wheel per OS/arch. uv builds and consumes maturin projects natively.
- fdu-py stays publish = false on crates.io; it is a binding artifact, not a library.
- Release workflow, CHANGELOG discipline, and cargo-semver-checks on PRs once there is a public API worth promising.

RE-VERIFY NAME AVAILABILITY IMMEDIATELY BEFORE FIRST PUBLISH — availability is a race. Names were free on PyPI, crates.io (including similarity blockers f-du, f_du, fd-u, fd_u, FDU), and Homebrew as of 2026-08-07.

Methodological trap for whoever re-checks: https://pypi.org/project/<name>/ can return HTTP 200 with an anti-bot interstitial (<title>Client Challenge</title>) for names that do not exist. Use the Simple index (PEP 503) or the JSON API, and calibrate against a known-present package, or the check reports false positives.

Two prior uses of the name exist, neither blocking: an npm package 'fdu' (disk usage flame graph, last published 2022) and a dormant GitHub script nicollet/fdu. Neither is on PyPI, crates.io, or Homebrew.
