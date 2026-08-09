---
type: is
id: is-01kzky75mq2zkhzvgzs9c95cts
title: Restore and enforce the 14-day executable-dependency cool-off
kind: bug
status: closed
priority: 0
version: 14
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - supply-chain
  - ci
  - merge-blocker
dependencies:
  - type: blocks
    target: is-01kzky7aq9m5j7r8a33tj0tx38
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01kzkzms7gmpjb0smwfc0c74wr
  - type: blocks
    target: is-01kzm3t12dcq5h7n92xztnhcyd
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:57:59.958Z
updated_at: 2026-08-09T21:40:37.279Z
closed_at: 2026-08-09T21:40:37.278Z
close_reason: The 14-day policy is restored and enforced before executable CI jobs. All locked ecosystems and bootstraps now have authoritative fail-closed provenance, fresh inputs were rolled back or narrowly documented, CI trust boundaries are hardened, managed hook drift is resolved without startup installation, and all ecosystem/runtime validation passed.
---
Audit evidence on 2026-08-09: Cargo.lock resolves clap 4.6.6 and clap_builder from 2026-08-06, the PyO3 0.29.2 family from 2026-08-05, and thiserror 2.0.20 plus its derive crate from 2026-08-08. The pinned rust-cache and Rust-toolchain action commits are also only three to four days old. Replace them with exact reviewed pins that cleared the repository 14-day policy. Add a tested fail-closed cool-off and provenance gate covering Cargo, uv, npm, GitHub Actions, and bootstrap downloads, with narrow recorded exceptions only. Set workflow permissions to contents read, disable checkout credential persistence, and prevent pull-request jobs from saving reusable caches. Begin with tests that reproduce every current violation and missing-provenance case; review source and lock diffs before accepting replacements.

## Notes

Implemented and verified 2026-08-09. Added a zero-dependency fail-closed provenance/cool-off validator with nine unit tests covering Cargo, npm, PyPI/uv artifacts, exact verified GitHub Action commits, official Rust and Node manifests, GitHub release assets, the first-party tbd bootstrap exception, shell downloader inventory, transient-only retries, and pull-request trust controls. Replaced every fresh Cargo resolution with an older reviewed lock entry; pinned Rust 1.97.1 and Node 24.18.0; replaced fresh rust-toolchain/rust-cache inputs with an aged exact action commit and no reusable Rust cache; set contents:read, disabled checkout credential persistence, lifecycle scripts, and uv cache; gated all jobs on provenance. Refreshed tbd surfaces with use_gh_cli:false, removed automatic session-start gh installers, retained one opt-in checksum-verified bootstrap, and made tbd doctor clean. Evidence: online validator verified 66 Cargo, 31 npm, 2 PyPI packages, 21 action uses, and all bootstrap pins; nine validator tests pass; npm audit and audit signatures pass; cargo deny reports advisories/bans/licenses/sources all ok; cargo test --locked --workspace --all-features passes 123 library tests, CLI/integration tests, and doctests on Rust 1.97.1; the downgraded PyO3 0.29.0 abi3 wheel builds, installs, and smoke-tests on macOS CPython 3.14.6.
