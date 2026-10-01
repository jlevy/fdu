---
type: is
id: is-01m3tr05r13ydjt9hza3nksarb
title: Run the stability pass's gates stage through the driver for real before a release relies on it
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-01T03:26:47.551Z
updated_at: 2026-10-01T03:26:47.551Z
---
PR #170's driver (scripts/release/stability_pass.py) has run make release-rehearse, the candidate build, the correctness passes, both breaks, the terminal tests and the peer self-test for real (macOS arm64, 2026-09-30, at 9e53c996), and the QA stage on Linux before the review fixes. make check, make cross-lint and make semver-check have run only against the tests' fake host: the macOS host lacked the reviewed cargo-semver-checks 0.50.0 and three of the five CROSS_TARGETS. Before the 0.3.1 stability pass, run 'make release-stability ARGS="--only gates,candidate"' on a host with both, and the harness, real-tree peer run and pty probe with the review fixes in. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (suggestion 4).
