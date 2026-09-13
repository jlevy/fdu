---
type: is
id: is-01m2eb2d0s1v2h7pz4ghp2hz24
title: "PR #54 review H86-6: RSS deltas quoted as ratio-of-medians instead of paired figures"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:34:14.040Z
updated_at: 2026-09-13T22:07:36.108Z
closed_at: 2026-09-13T22:07:36.107Z
close_reason: "Fixed in 2d36eab: paired -49.16% and -35.05% (and -14.28% for opened-discovery) in the artifact reason and body, the streaming-parity plan, the ledger prose regenerated from the reason, the fdu-xde5 and fdu-prph notes, and the PR body."
resolution: null
duplicate_of: null
---
Low. Artifact :410, :420-422, streaming-parity plan insertion, ledger prose (from verdict.reason), and the PR body quote peak RSS -49.4% and -35.9%, which are ratios of medians. The artifact's paired figures are -49.16% (cold-scan-index) and -35.05% (default-tree); performance-loop.md requires the paired figure. Fix: quote -49.2% and -35.0% everywhere.
