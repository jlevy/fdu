---
type: is
id: is-01m3n284jpq1089gc8ph1pj9kn
title: Hard-link-aware unique allocated size (pdu -H)
kind: feature
status: open
priority: 2
version: 1
labels:
  - parity
dependencies: []
parent_id: is-01m3n282ey52kzkf7ctv7fyeq3
created_at: 2026-09-28T22:30:27.669Z
updated_at: 2026-09-28T22:30:27.669Z
---
pdu --deduplicate-hardlinks subtracts sizes shared by links within each subtree and reports shared details in JSON. fdu counts per path. Relates to fdu-579b and docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md (unique-file allocation measure); nlink is not retained today.
