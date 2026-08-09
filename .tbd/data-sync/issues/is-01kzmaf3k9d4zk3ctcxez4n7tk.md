---
type: is
id: is-01kzmaf3k9d4zk3ctcxez4n7tk
title: Keep cfg-disabled integration test crates documented cross-platform
kind: bug
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - ci
  - windows
  - merge-blocker
dependencies:
  - type: blocks
    target: is-01kzm3t12dcq5h7n92xztnhcyd
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T22:32:02.920Z
updated_at: 2026-08-09T22:38:41.270Z
closed_at: 2026-08-09T22:38:41.269Z
close_reason: "Fixed in 8a539a2 by keeping crate documentation before #![cfg(unix)]. The exact Rust 1.97.1 x86_64-pc-windows-msvc all-feature test-target check passes with warnings denied, the complete local make check passes, and fresh GitHub Actions run 31339731585 is green on Windows and every other required job."
---
Windows CI compiles the Unix-only cli_exit integration-test target as an empty cfg-disabled crate under RUSTFLAGS=-D warnings. Because #![cfg(unix)] preceded the //! crate documentation, rustc removed the docs before enforcing workspace missing_docs and failed the Windows job. Put crate docs before the cfg attribute, retain Unix-only execution, and verify the complete local and fresh cross-platform gates.

## Notes

CI run 31339430092 failed only on Windows before tests: workspace -D missing-docs saw the cfg-disabled cli_exit integration crate without docs because #![cfg(unix)] preceded //! documentation. Reordered those crate attributes. RUSTFLAGS=-D warnings cargo +1.97.1 check --locked -p fdu --all-features --tests --target x86_64-pc-windows-msvc passes after installing the exact target, and the complete local make check passes. Fresh CI remains.
