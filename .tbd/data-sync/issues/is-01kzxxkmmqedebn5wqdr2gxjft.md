---
type: is
id: is-01kzxxkmmqedebn5wqdr2gxjft
title: Align real-tree allocated-byte oracle on non-POSIX hosts
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - correctness
  - windows
dependencies: []
parent_id: is-01kzxwah348yq9sg1em0cqv2k4
created_at: 2026-08-13T15:59:44.278Z
updated_at: 2026-08-13T16:10:40.018Z
closed_at: 2026-08-13T16:10:40.017Z
close_reason: Aligned the real-tree allocated-byte aggregate with FDU's non-POSIX apparent-size fallback, tested both POSIX and non-POSIX contracts independent of host OS, documented PEV-38, and passed make test-performance plus the complete make check gate.
---
The new real-process probe integration test exposed that benchmarks/realtree/tree.py reports aggregate allocated_bytes as zero when st_blocks is unavailable, while FDU and the same oracle engine digest use apparent size as the honest non-POSIX fallback. Centralize the platform rule, test the no-block case, update the review ledger, and restore the Windows CI gate.
