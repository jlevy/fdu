---
type: is
id: is-01m3kdqg02e0e65fp35q7gt1zc
title: make check hard-codes ./target/debug/fdu and fails with CARGO_TARGET_DIR outside the checkout
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T07:12:36.340Z
updated_at: 2026-09-28T07:12:36.340Z
---
AGENTS.md says to give each worktree its own Cargo target directory, but several gate consumers ignore CARGO_TARGET_DIR: the golden runner (scripts/run-golden.mjs: 'the rust surface is not runnable at <checkout>/target/debug/fdu ... build it with make build'), PATH_INDEPENDENCE_ENV (Makefile ~L303 FDU_BIN=$(CURDIR)/target/debug/fdu), and scripts/content-selfcheck.mjs. So 'CARGO_TARGET_DIR=/elsewhere make check' builds into /elsewhere and then fails test-golden (observed 2026-09-28). Resolve the binary via cargo metadata's target_directory (or honor CARGO_TARGET_DIR) everywhere, or document that the per-worktree target must be the checkout's own ./target. Related: fdu-8whh, fdu-fihm.
