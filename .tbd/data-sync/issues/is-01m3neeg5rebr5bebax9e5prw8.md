---
type: is
id: is-01m3neeg5rebr5bebax9e5prw8
title: Gates hard-code target/ despite AGENTS.md recommending a per-worktree CARGO_TARGET_DIR
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T02:03:39.064Z
updated_at: 2026-09-29T02:03:39.064Z
---
run-golden.mjs, the Makefile's path-independence and release-rehearse recipes, and the default binary paths in the correctness and terminal scripts assume target/. Honour CARGO_TARGET_DIR (cargo metadata --format-version 1 target_directory). Found in the 0.2.1 stability pass.
