---
type: is
id: is-01kzky7aq9m5j7r8a33tj0tx38
title: Pin Rust tooling and prove every supported feature and MSRV contract
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - ci
  - compatibility
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01kzkzmsegmx4sfswka2084se6
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:05.160Z
updated_at: 2026-08-09T22:10:21.911Z
closed_at: 2026-08-09T22:10:21.910Z
close_reason: Pinned normal toolchain and complete locked normal/feature/MSRV matrix are implemented, aligned between make check and CI, and passing locally.
---
The repository declares Rust 1.85 as MSRV but has no rust-toolchain.toml; normal CI requests moving stable, so rustfmt, Clippy, code generation, and future benchmark evidence are not reproducible. Pin an exact normal toolchain and components only after the release clears the cool-off. Keep MSRV separate and run the supported core tests on 1.85, not only cargo check. Add the watch-only combination with no default features to make check and CI, alongside default, all-feature, and core-only coverage. Each matrix row must answer a documented compatibility question, use locked resolution, and keep local commands aligned with CI.

## Notes

Complete on the assembled PR branch. rust-toolchain.toml pins 1.97.1 with rustfmt and Clippy; normal CI uses that exact toolchain; make check and CI exercise all-feature, core-only, and watch-only questions; MSRV remains a separate exact 1.85.0 lane that compiles the all-feature workspace and runs 104 core tests plus 2 doctests. All resolution is locked. Local make msrv passed on 2026-08-09.
