---
type: is
id: is-01m3stdvsbm5f4adzh7cxptp25
title: Run make check, cross-lint and semver-check through make release-stability before 0.3.1
kind: task
status: open
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-30T18:49:58.827Z
updated_at: 2026-10-05T01:16:37.328Z
---
PR #170's follow-up (comment 5924110534) ran most of the command for real on macOS at 9e53c996: release-rehearse, the wheel build after target-owner, the candidate, the correctness breaks and the terminal tests all passed, and the run found and fixed fdf46d71. Three gates have still run only against the fake host: make check, make cross-lint and make semver-check. That host lacked the reviewed cargo-semver-checks and three of the five cross targets. Before the 0.3.1 stability pass relies on the command, run --only check,cross-lint,semver-check for real on a host with the reviewed cargo-semver-checks and every CROSS_TARGETS target installed. Check each gate's log and verdict, the cross-lint target list in its detail, and cleanup, and fix anything that differs.
