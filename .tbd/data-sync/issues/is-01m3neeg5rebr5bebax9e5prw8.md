---
type: is
id: is-01m3neeg5rebr5bebax9e5prw8
title: Gates hard-code target/ despite AGENTS.md recommending a per-worktree CARGO_TARGET_DIR
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-29T02:03:39.064Z
updated_at: 2026-09-30T02:45:13.211Z
closed_at: 2026-09-30T02:45:13.211Z
close_reason: "Fixed in 9c3db79c with fdu-dfbu: Makefile CARGO_TARGET asks cargo metadata once (covers CARGO_TARGET_DIR and build.target-dir); PATH_INDEPENDENCE_ENV, release-rehearse's package copy, test-terminal and test-performance use it; PERF_TARGET_DIR reuses it; run-golden/content-selfcheck/check-yaml share scripts/cargo-target.mjs; Python defaults (terminal, correctness, path_independence runner) honour CARGO_TARGET_DIR. scripts/cargo-target.test.mjs (make supply-chain) dry-runs the consumers under a moved CARGO_TARGET_DIR and bans literal target/{debug,release,profiling,package} in the Makefile and gate scripts; fails before, passes after. Verified: run-golden and content-selfcheck report /elsewhere paths under CARGO_TARGET_DIR=/elsewhere; make perf-test and test-path-independence pass. The full CARGO_TARGET_DIR=/elsewhere make check still needs the coordinator's run."
resolution: null
duplicate_of: null
---
run-golden.mjs, the Makefile's path-independence and release-rehearse recipes, and the default binary paths in the correctness and terminal scripts assume target/. Honour CARGO_TARGET_DIR (cargo metadata --format-version 1 target_directory). Found in the 0.2.1 stability pass.
