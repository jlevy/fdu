---
type: is
id: is-01kzqk4a2pj7pss8ekwczfzrnw
title: "PR #3 review R8: bound cache policy claims by measured evidence"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:09.589Z
updated_at: 2026-08-11T06:28:14.305Z
closed_at: 2026-08-11T06:28:14.304Z
close_reason: Separated read/write cache policy, bounded auto claims to measured evidence, added the required platform/scale matrix, and chose cold scan for unknown cells.
---
FDU-PR3-R8. README.md and composable CLI plan. The exact stat-tier cache is slower on measured APFS and cross-platform benefits are hypotheses. Separate cache write policy from read-path selection; specify an evidence-gated platform/filesystem/scale/state matrix and first-output/completion measurements; retain explicit controls without calling auto universally fastest. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
