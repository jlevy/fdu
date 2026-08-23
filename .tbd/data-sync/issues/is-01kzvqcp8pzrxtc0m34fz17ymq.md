---
type: is
id: is-01kzvqcp8pzrxtc0m34fz17ymq
title: Golden the realistic default human report UX
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-12T19:32:36.245Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-13T05:51:07.666Z
close_reason: Added the realistic cli-overview tryscript golden and retained focused limit/ellipsis coverage; all 72 golden cases pass with compact ten-cell aligned bars and explicit-root help behavior.
---
Add an end-to-end tryscript session over a realistic nested fixture that pins the natural explicit-path human report: useful overview depth, compact aligned ten-cell bars, directory rollups, and omission markers only for genuinely hidden rows. Compare with post-#5 and prior/dust output and restore regressions without broadening PR #8 beyond behavior preservation.

## Notes

Applied tbd golden-testing-guidelines and the maintainer's rule: concise realistic end-to-end scenario, minimal fixture, maximum critical surface, complete output rather than surgical extraction. Added cli-overview.tryscript.md over a 16-file nested Rust-project fixture; it pins default depth/limit, hierarchy, sorting, alignment, fixed ten-cell bars, hidden-descendant roll-up, and no spurious ellipsis. Existing focused human golden retains the actual limit-ellipsis boundary. Verified registry latest tryscript is 0.2.0 and the repository already pins exactly 0.2.0 under its documented first-party exception.
