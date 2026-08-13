---
type: is
id: is-01kzy09g92seeh160bbh3m74nk
title: "H69: Pipeline macOS directory opens ahead of bulk enumeration"
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T16:46:37.857Z
updated_at: 2026-08-13T17:04:32.319Z
---
The H67 exact-binary profile places about 96% of FDU worker samples in open or getattrlistbulk, with open about 40% and no consumer bottleneck. Prototype a macOS-only bounded opener stage that keeps the accepted six scan/parser workers but allows at most two directory opens to overlap active bulk enumeration. This is distinct from rejected 8-16 full-worker pools and parent-relative openat: it spends extra threads only on the open phase. Preserve breadth-first region ordering, strict bulk fallback, one-filesystem/depth/error/partial semantics, and the exact oracle. Screen on the immutable 901,963-entry APFS tree, confirm any >=3% paired wall gain on an independent large topology, and reject if CPU/context-switch cost is disproportionate or the interval crosses zero.

## Notes

Corrected H69 screen completed with six scan/parser workers and two opener helpers: five adjacent pairs after two warmups, exact oracle, no mutation, wall point estimate -4.47% with 95% CI [-31.04%, +33.91%]. The first prototype incorrectly stacked opener helpers with the adaptive reserve and reached 18 threads; that run is excluded. No production code retained. Next gate is 12 quiet pairs plus independent large-tree confirmation; opener and reserve policies must share one concurrency budget.
