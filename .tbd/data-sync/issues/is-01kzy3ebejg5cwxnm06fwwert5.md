---
type: is
id: is-01kzy3ebejg5cwxnm06fwwert5
title: Benchmark contracts fdu-index-summary and fdu-transient-summary have identical argv
kind: chore
status: closed
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-08-13T17:41:42.481Z
updated_at: 2026-08-13T18:14:51.752Z
closed_at: 2026-08-13T18:14:51.751Z
close_reason: "CONTRACTS comment landed on PR #13 (0ede2e4)"
---
In benchmarks/realtree/compare_tools.py the two contracts differ only in name, work_class, and description; their argv is byte-identical, so they are distinguished solely by which binary the operator points at. Nothing in the harness can verify that the binary labelled index-summary actually retains an index. Worth a comment on CONTRACTS saying so, so a later reader does not assume the harness enforces the work class it reports.
