---
type: is
id: is-01m2eeg5j9pvsn2vq5sjwyj653
title: "PR #52 review PERF-5: documented probe build recipes omit --features gitignore"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:34:10.887Z
updated_at: 2026-09-13T22:58:51.669Z
closed_at: 2026-09-13T22:58:41.820Z
close_reason: "Fixed in the PR #52 recipe commit: explorations/benchmarks/README.md spells out the perf-probe-release build with --features gitignore, and performance-loop.md runs make perf-probe-release instead of building -p fdu."
resolution: null
duplicate_of: null
---
PR #52 review PERF-5 (Low). explorations/benchmarks/README.md:79 and docs/project/guides/performance-loop.md:171 at afbb2ee. The README documents a release-probe build without --features gitignore, and the guide builds -p fdu. Both contradict the Makefile and the policy test_provenance.py enforces, so an operator following the README builds a control the record treats as having no control semantics. Leftover of fdu-by5y's recipe correction. Fix: update both recipes.

## Notes

Fixed in 0c7099d on PR #52.
