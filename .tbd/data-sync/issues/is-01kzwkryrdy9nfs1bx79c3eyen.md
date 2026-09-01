---
type: is
id: is-01kzwkryrdy9nfs1bx79c3eyen
title: "H60: Build bootstrap indexes as worker-local subtrees and splice them"
kind: task
status: in_progress
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
labels:
  - performance
  - experiment
  - campaign-2
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
hold: null
hold_until: null
created_at: 2026-08-13T03:48:38.284Z
updated_at: 2026-09-01T16:55:02.790Z
started_at: 2026-09-01T15:20:04.382Z
---
Inspired by pdu's worker-local recursive aggregation and FDU's measured single-consumer residue: for cold bootstrap only, build disjoint subtree arenas in workers and structurally merge them plus one roll-up at region completion instead of sending one path operation per entry. Preserve deterministic EntryId/snapshot/query behavior, progressive publication requirements, errors, and the delta contract for post-bootstrap changes. Profile first; target cold-scan-index component/user CPU and channel allocation, with >=3% end-to-end wall and bounded RSS.

## Notes

2026-09-01 the prototype now consumes directory-shaped records concurrently, merges direct contributions during the walk, and uses one reverse directories-only roll-up pass. Naive post-join construction was rejected at roughly 410 ms versus 270 ms because it lost pipeline overlap. The pipelined controls-aware route improves controls-rich component time 47.43% and reaches historical cold-construction parity. Worker-local compact arenas and structural promotion remain open.
