---
type: is
id: is-01kzmmjc0yat2kf5c3616b2c91
title: Track tryscript update-mode preservation of named patterns
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels:
  - testing
  - upstream
dependencies: []
created_at: 2026-08-10T01:28:35.614Z
updated_at: 2026-08-10T01:29:46.887Z
closed_at: 2026-08-10T01:29:46.886Z
close_reason: Upstream issue 49 filed and the local workaround is documented in the linked golden-test spec.
---
Running tryscript 0.1.7 with --update on a failing block replaces the whole expected output and turns named patterns such as SCAN_PATH, MTIME_NS, and ALLOCATED into one run's literal values. The required subsequent comparison then fails on the next sandbox. File an upstream issue with a minimal reproduction and acceptance behavior. Keep fdu's named-pattern normalization exact and document the manual review workaround until upstream support exists.

## Notes

Filed upstream as https://github.com/jlevy/tryscript/issues/49 with a minimal mixed stable/unstable-output reproduction, source-level cause, and acceptance criteria. The completed golden plan documents the fail-safe local review: make golden-update reruns comparison, and named patterns must be restored before the suite can pass.
