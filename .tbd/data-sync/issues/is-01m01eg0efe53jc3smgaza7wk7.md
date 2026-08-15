---
type: is
id: is-01m01eg0efe53jc3smgaza7wk7
title: "H89: Qualify the selected policy with the macOS cold-cache protocol"
kind: task
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
  - macos
dependencies: []
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:52:34.382Z
updated_at: 2026-08-15T01:17:11.009Z
---
After fdu-rjqx establishes a repeatable macOS cold-cache protocol and fdu-9x4o selects a candidate or records no winner, qualify that outcome outside the primary warm-steady release claim. Keep warm-steady as the main interactive regime, label sync-plus-purge only as a diagnostic, and use the verified dedicated ordinary APFS-volume preparation from fdu-rjqx for stronger cold evidence. Do not use a RAM disk for device-latency conclusions.

Acceptance: every sample records preparation success, device/filesystem facts, tree fingerprint, policy history, wall/resources, and invalidation reason; the report states whether cold evidence supports, limits, or contradicts the warm-selected policy without generalizing beyond the measured host/volume. This P2 qualification does not block fdu-8evu or fdu-j062 by default; if it reveals a supported-product defect, open or promote an explicit release-blocking bead rather than silently changing this epic’s gate.
