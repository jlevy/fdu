---
type: is
id: is-01m3q1rvwx3k16jhfe4kkh1hzz
title: "macOS: end-to-end release gate for the round on APFS (e5a71c8a vs final head, A/A arm)"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3q1rvcnwdw17kysjfag0f6g
created_at: 2026-09-29T17:00:36.125Z
updated_at: 2026-09-29T17:00:36.125Z
---
Release gate for #161 and 0.2.2: Q0 engine against the final head with a byte-identical Q0 placebo arm, on linux-v6.12-apfs, node-modules-dense-darwin, metabrowser-clone and macos-balanced-1m (screen), 20 pairs (12 on the screen); default tree and summary with and without .gitignore; non-inferiority (upper 95% bound at most +3%) with peak RSS as the second metric.
