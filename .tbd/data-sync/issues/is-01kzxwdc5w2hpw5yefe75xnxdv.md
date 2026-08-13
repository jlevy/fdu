---
type: is
id: is-01kzxwdc5w2hpw5yefe75xnxdv
title: Fix perf-probe newest-mtime oracle mismatch
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - correctness
dependencies: []
parent_id: is-01kzxwah348yq9sg1em0cqv2k4
created_at: 2026-08-13T15:38:50.424Z
updated_at: 2026-08-13T15:51:52.408Z
closed_at: 2026-08-13T15:51:52.395Z
close_reason: Reproduced with a red perf-probe unit test, emitted and self-validated newest_file_mtime_ns for producer and index summaries, added a real-process independent-tree oracle test, and made the normal performance test target run both harness suites. Focused tests and the complete make check gate pass.
---
Current benchmarks/realtree/tree.py requires newest_file_mtime_ns from perf_probe summaries, but crates/fdu/examples/perf_probe.rs neither records nor renders the field. Reproduce on a non-empty tree, add a focused test that fails before the fix, emit the exact newest regular-file mtime for producer and index summaries, and restore make perf-compare oracle validity.
