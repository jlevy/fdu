---
type: is
id: is-01m2nnne8pq16fh5m76b8h2gcr
title: "PR #68 review PR68-2: Preserve the comparison reproduction artifact"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2nnn53byv4wgyjv0t0szynf
hold: null
hold_until: null
created_at: 2026-09-16T17:54:04.690Z
updated_at: 2026-09-16T17:58:27.314Z
started_at: 2026-09-16T17:54:33.496Z
closed_at: 2026-09-16T17:58:27.314Z
close_reason: "Fixed in PR #68 commit 45e688c: narrowed the README/report to exploratory synthetic-corpus calibration with no release qualification or portable ordering, and committed the complete redacted harness result with exact commands, binary identities, raw pairs, resources, validity data and bootstrap intervals."
resolution: null
duplicate_of: null
---
Formal review 5226316777, PR68-2. report-2026-09-16-fdu-live-tool-comparison.md:97-113 omits exact commands, comparator versions/binary hashes, raw paired samples, and bootstrap inputs, so the published intervals and work classes cannot be independently audited or rerun. The preserved final raw harness artifact is run-rc-tree-v2.json; commit a redacted durable artifact and link it from the report.
