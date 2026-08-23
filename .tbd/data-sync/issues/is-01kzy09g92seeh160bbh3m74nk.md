---
type: is
id: is-01kzy09g92seeh160bbh3m74nk
title: "H69: Pipeline macOS directory opens ahead of bulk enumeration"
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - experiment
  - macos
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T16:46:37.857Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-13T18:23:47.706Z
close_reason: "Superseded by H70/exp-046: the corrected pairwise-helper screen was inconclusive under host noise, no production code was retained, and the shared-pool design owns confirmation."
---
The H67 exact-binary profile places about 96% of FDU worker samples in open or getattrlistbulk, with open about 40% and no consumer bottleneck. Prototype a macOS-only bounded opener stage that keeps the accepted six scan/parser workers but allows at most two directory opens to overlap active bulk enumeration. This is distinct from rejected 8-16 full-worker pools and parent-relative openat: it spends extra threads only on the open phase. Preserve breadth-first region ordering, strict bulk fallback, one-filesystem/depth/error/partial semantics, and the exact oracle. Screen on the immutable 901,963-entry APFS tree, confirm any >=3% paired wall gain on an independent large topology, and reject if CPU/context-switch cost is disproportionate or the interval crosses zero.

## Notes

Experimental only; no production code is retained in PR #8. Exact current FDU/dumac profiles localized the warm APFS floor to open plus getattrlistbulk. The first paired-helper prototype showed -4.47% in five noisy pairs with CI [-31.04%, +33.91%]; a shared-opener follow-up is tracked separately in H70. Resume after the stable macOS performance PR lands, under a single concurrency budget.
