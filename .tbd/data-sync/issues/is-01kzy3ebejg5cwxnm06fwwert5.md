---
type: is
id: is-01kzy3ebejg5cwxnm06fwwert5
title: Benchmark contracts fdu-index-summary and fdu-transient-summary have identical argv
kind: chore
status: closed
priority: 3
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzy4tve6eej9e0jhxfqmqqmz
created_at: 2026-08-13T17:41:42.481Z
updated_at: 2026-08-13T19:10:10.234Z
closed_at: 2026-08-13T19:10:10.234Z
close_reason: "Addressed on main by the #8 stabilization commit 51917f8, which documents the identical-argv contracts in compare_tools.py directly"
---
In benchmarks/realtree/compare_tools.py the two contracts differ only in name, work_class, and description; their argv is byte-identical, so they are distinguished solely by which binary the operator points at. Nothing in the harness can verify that the binary labelled index-summary actually retains an index. Worth a comment on CONTRACTS saying so, so a later reader does not assume the harness enforces the work class it reports.
