---
type: is
id: is-01kzg4c6vnh98mqrpkzw7ydne0
title: "Publish fdu 0.1.0 by hand: crates.io, PyPI, GitHub Release"
kind: epic
status: open
priority: 1
version: 19
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
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
  - type: blocks
    target: is-01m2pj0jfrvm78rpmv6rwc2ktv
  - type: blocks
    target: is-01m2pj0kq0s5533yhbzfetepbv
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
child_order_hints:
  - is-01m2phzkwcvz3fwk2nh150bdmj
  - is-01m2phzm7r5vw97jr9y4mtzxxs
  - is-01m2phzmjh2sk8fzqya61v47rd
  - is-01m2phzmxbwajz8eqrcv6rzfks
created_at: 2026-08-08T07:28:38.772Z
updated_at: 2026-09-17T02:09:30.847Z
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

Packaging prerequisites are implemented on codex/python-packaging-release-engineering and tracked by fdu-3d8c. This final publication bead now owns the remaining external/irreversible work: run and retain the five-platform rehearsal; configure the protected release environment and PyPI pending publisher; add narrowly scoped publisher, attestation, and GitHub Release jobs; create and verify the signed tag; use/remove the one-time crates.io token; upload and verify both registries; configure crates.io OIDC for later releases; then establish post-0.1 semver checks. Do not publish from the implementation PR.

2026-09-15 DECISIONS (user): the first release (0.1.0) is published by hand from the signed tag, following docs/project/guides/release-process.md. Workflow publish jobs, the release environment and trusted publishers are post-0.1.0.

2026-09-17: Moved under fdu-gjc2. The Phase-1 blockers fdu-oqoy, fdu-jej9, fdu-lka2 and fdu-ywu0 no longer
block publication (2026-09-15 decision: publish 0.1.0 by hand; they stay open as post-release work).
Blocked by the end-to-end verification, signing identity, and repository settings beads. Workflow
publish jobs, the protected environment and trusted publishers are tracked in the release automation epic.
