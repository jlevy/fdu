---
type: is
id: is-01kzw3te81j66eehy48rx2djv5
title: Iterate on measured high-impact live-scan bottlenecks
kind: task
status: closed
priority: 1
version: 12
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
  - is-01kzxp0fjq8f0q545tag20ffwf
created_at: 2026-08-12T23:09:49.696Z
updated_at: 2026-08-13T13:54:54.851Z
closed_at: 2026-08-13T13:54:54.850Z
close_reason: "This measured iteration phase is complete: accepted gains were retained, weak experiments reverted and recorded, PR #5 integration was semantically checked, a full near-million competitor comparison was published, goldens/harness/full checks pass, and every next architectural experiment remains tracked in its own open bead."
---
Use the established profile-measure-record loop on the frozen workspace. Revisit only hypotheses made newly plausible by the breadth-first, adaptive-worker, bulk-metadata, or heterogeneous-tree evidence; keep substantial composable gains and record/revert weak or losing variants.

## Notes

Completed exp-033 through exp-044, including post-composable-CLI semantic/performance validation, BFS-sensitive retests, accepted exact transient rich summary, rejected deeper summary/worker/selected-total specializations, and definitive v3 multi-tool comparison on the self-contained 901,963-entry tree. Current branch is 0 commits behind origin/main, output-equivalent to the post-CLI control across human/JSON/YAML/filtered paths, and passes make check. Continue future iterations in open child beads fdu-prph (compact index), fdu-r9he (portable stat chunks), fdu-weey (worker-local subtree splice), and fdu-f67r (dense base/overlay); Linux matrix is fdu-nffc.
