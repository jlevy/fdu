---
type: is
id: is-01m2nnne6b8p7b6jkkr4380prn
title: "PR #68 review PR68-1: Release headline exceeds evidence scope"
kind: bug
status: closed
priority: 0
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2nnn53byv4wgyjv0t0szynf
hold: null
hold_until: null
created_at: 2026-09-16T17:54:04.617Z
updated_at: 2026-09-16T17:58:27.292Z
started_at: 2026-09-16T17:54:30.432Z
closed_at: 2026-09-16T17:58:27.291Z
close_reason: "Fixed in PR #68 commit 45e688c: narrowed the README/report to exploratory synthetic-corpus calibration with no release qualification or portable ordering, and committed the complete redacted harness result with exact commands, binary identities, raw pairs, resources, validity data and bootstrap intervals."
resolution: null
duplicate_of: null
---
Formal review 5226316777, PR68-1. README.md:9-13 and 162-190 plus report lines 9-12 and 117-132 claim typical performance and fastest-tool ordering from a generated balanced corpus under an uncontrolled host. performance-loop.md:139-147 limits uncontrolled evidence to exploration/discovery; lines 192-210 forbid generated corpora from establishing tool ordering. Fix by qualifying the result as exploratory and synthetic with no portable ordering, or remeasure under an eligible real-tree release regime.
