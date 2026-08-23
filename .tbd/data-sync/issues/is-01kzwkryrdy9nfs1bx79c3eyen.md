---
type: is
id: is-01kzwkryrdy9nfs1bx79c3eyen
title: "H60: Build bootstrap indexes as worker-local subtrees and splice them"
kind: task
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - experiment
  - campaign-2
dependencies: []
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
created_at: 2026-08-13T03:48:38.284Z
updated_at: 2026-08-23T09:09:06.202Z
---
Inspired by pdu's worker-local recursive aggregation and FDU's measured single-consumer residue: for cold bootstrap only, build disjoint subtree arenas in workers and structurally merge them plus one roll-up at region completion instead of sending one path operation per entry. Preserve deterministic EntryId/snapshot/query behavior, progressive publication requirements, errors, and the delta contract for post-bootstrap changes. Profile first; target cold-scan-index component/user CPU and channel allocation, with >=3% end-to-end wall and bounded RSS.
