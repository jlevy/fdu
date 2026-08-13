---
type: is
id: is-01kzwet2mwgg1f1ddkydaw9r9q
title: Investigate any alternative that outperforms fdu
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
dependencies: []
parent_id: is-01kzw3t92p7d4512h8vn6ktch1
created_at: 2026-08-13T02:21:52.155Z
updated_at: 2026-08-13T05:51:08.766Z
closed_at: 2026-08-13T05:51:08.765Z
close_reason: Investigated pinned sources for dust, gdu, pdu, ncdu, dua, diskus, and dumac. Explained dumac's narrower scalar-only advantage and queued H58-H61 plus H19-H22 with design gates.
---
For every benchmarked alternative materially faster than fdu, inspect implementation and measurement semantics, profile as needed, explain the gap from first principles, and turn actionable high-impact findings into dependent beads.

## Notes

Pinned source audit: dust v1.2.4 fabe19b (Apache-2.0), gdu v5.36.1 8d64b4f (MIT), pdu 0.24.0 4e19260 (Apache-2.0), dua v2.41.1 90a59e1 (MIT), diskus v0.9.0 d8a77db (MIT/Apache-2.0), dumac 1ffbe3c (no declared license; inspect-only/executable input). Transferable queue: H58 dua wide-dir stat chunks, H59 pdu bounded retention design gate, H60 worker-local subtree splice, existing H19-H22 compact full index. H57 over-threading refuted.
