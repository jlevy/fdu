---
type: is
id: is-01kzw3te81j66eehy48rx2djv5
title: Iterate on measured high-impact live-scan bottlenecks
kind: task
status: in_progress
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
child_order_hints:
  - is-01kzwk202rpvey4gn0kkwz532c
  - is-01kzwk20bb97hagzjeegkxpd77
  - is-01kzwk20kyaxajq254tee8apts
  - is-01kzwk20wzb7qcahfa3hq6mn4f
  - is-01kzwkryrdy9nfs1bx79c3eyen
  - is-01kzwsr47nxr6arn4qbdz66949
created_at: 2026-08-12T23:09:49.696Z
updated_at: 2026-08-13T05:53:07.031Z
---
Use the established profile-measure-record loop on the frozen workspace. Revisit only hypotheses made newly plausible by the breadth-first, adaptive-worker, bulk-metadata, or heterogeneous-tree evidence; keep substantial composable gains and record/revert weak or losing variants.

## Notes

The post-PR-5 iteration completed exp-033 through exp-039 and the definitive cross-tool run. Continue only with queued high-impact work: H19-H22 compact entries, H58 portable stat chunks, H59 design-gated bounded retention, H60 worker-local subtree splice, and H61 dense immutable base plus sparse overlay. Linux evidence is tracked separately by fdu-nffc.
