---
type: is
id: is-01m3kdqg02e0e65fp35q7gt1zc
title: make check hard-codes ./target/debug/fdu and fails with CARGO_TARGET_DIR outside the checkout
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-28T07:12:36.340Z
updated_at: 2026-09-30T02:45:13.875Z
closed_at: 2026-09-30T02:45:13.875Z
close_reason: "Fixed in 9c3db79c (same change as fdu-bi9a): the golden runner, PATH_INDEPENDENCE_ENV and content-selfcheck now resolve the binary from cargo metadata's target_directory, so CARGO_TARGET_DIR outside the checkout no longer fails test-golden. Verified by dry runs and scripts/cargo-target.test.mjs; an end-to-end CARGO_TARGET_DIR=/elsewhere make check needs the coordinator's build."
resolution: null
duplicate_of: null
---
AGENTS.md says to give each worktree its own Cargo target directory, but several gate consumers ignore CARGO_TARGET_DIR: the golden runner (scripts/run-golden.mjs: 'the rust surface is not runnable at <checkout>/target/debug/fdu ... build it with make build'), PATH_INDEPENDENCE_ENV (Makefile ~L303 FDU_BIN=$(CURDIR)/target/debug/fdu), and scripts/content-selfcheck.mjs. So 'CARGO_TARGET_DIR=/elsewhere make check' builds into /elsewhere and then fails test-golden (observed 2026-09-28). Resolve the binary via cargo metadata's target_directory (or honor CARGO_TARGET_DIR) everywhere, or document that the per-worktree target must be the checkout's own ./target. Related: fdu-8whh, fdu-fihm.
