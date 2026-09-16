---
type: is
id: is-01m2nnne8pq16fh5m76b8h2gcr
title: "PR #68 review PR68-2: Preserve the comparison reproduction artifact"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2nnn53byv4wgyjv0t0szynf
created_at: 2026-09-16T17:54:04.690Z
updated_at: 2026-09-16T17:54:04.690Z
---
Formal review 5226316777, PR68-2. report-2026-09-16-fdu-live-tool-comparison.md:97-113 omits exact commands, comparator versions/binary hashes, raw paired samples, and bootstrap inputs, so the published intervals and work classes cannot be independently audited or rerun. The preserved final raw harness artifact is run-rc-tree-v2.json; commit a redacted durable artifact and link it from the report.
