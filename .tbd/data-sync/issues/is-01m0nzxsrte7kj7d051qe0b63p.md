---
type: is
id: is-01m0nzxsrte7kj7d051qe0b63p
title: "PR #42 review R1: moved stack-safety test filters on the old module path and runs zero tests"
kind: bug
status: closed
priority: 0
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:00.730Z
updated_at: 2026-08-23T00:39:53.289Z
closed_at: 2026-08-23T00:39:53.288Z
close_reason: "Fixed. The child filter is derived from module_path!() instead of written down, and the parent now requires the child's stdout to report '1 passed' -- a filter matching nothing is a zero-test run that exits 0. Verified red-green: renaming the constant makes the test fail; the passing run went from 0.00s to 0.24s, which is the renderer actually running."
---
crates/fdu-core/src/report_format.rs:2184. The child process is spawned with --exact cli::tests::deep_rendering_is_stack_safe, but the test now lives at report_format::tests::deep_rendering_is_stack_safe. libtest runs 0 tests and exits 0, so assert!(output.status.success()) passes vacuously and deep-render coverage is gone. Fix: correct the filter and assert the child's stdout reports 1 passed.
