---
type: is
id: is-01kzky75mq2zkhzvgzs9c95cts
title: Restore and enforce the 14-day executable-dependency cool-off
kind: bug
status: open
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - supply-chain
  - ci
dependencies:
  - type: blocks
    target: is-01kzky7aq9m5j7r8a33tj0tx38
  - type: blocks
    target: is-01kzky7fjvk5f7758cav879nhs
  - type: blocks
    target: is-01kzky8bctckj3kk8gwntbg8tn
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01kzkzms7gmpjb0smwfc0c74wr
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:57:59.958Z
updated_at: 2026-08-09T19:25:49.998Z
---
Audit evidence on 2026-08-09: Cargo.lock resolves clap 4.6.6 and clap_builder from 2026-08-06, the PyO3 0.29.2 family from 2026-08-05, and thiserror 2.0.20 plus its derive crate from 2026-08-08. The pinned rust-cache and Rust-toolchain action commits are also only three to four days old. Replace them with exact reviewed pins that cleared the repository 14-day policy. Add a tested fail-closed cool-off and provenance gate covering Cargo, uv, npm, GitHub Actions, and bootstrap downloads, with narrow recorded exceptions only. Set workflow permissions to contents read, disable checkout credential persistence, and prevent pull-request jobs from saving reusable caches. Begin with tests that reproduce every current violation and missing-provenance case; review source and lock diffs before accepting replacements.

## Notes

tbd doctor also reports stale AGENTS.md and Codex managed surfaces. A dry run says setup would remove two legacy hooks and refresh Codex hooks. Reconcile this only under an explicit diff review: preserve the repository rule that session startup performs no default software installation or network-routing change, and do not overwrite the opt-in checksum-verified bootstrap blindly.
