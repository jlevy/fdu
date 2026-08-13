---
type: is
id: is-01kzxwah348yq9sg1em0cqv2k4
title: "Audit PR #8 performance narrative and dumac gap"
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - review
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
child_order_hints:
  - is-01kzxwdc5w2hpw5yefe75xnxdv
  - is-01kzxxkmmqedebn5wqdr2gxjft
created_at: 2026-08-13T15:37:17.156Z
updated_at: 2026-08-13T15:59:44.278Z
---
Audit every change in PR #8 against commits, profiles, experiment artifacts, and final comparator evidence. Explain precisely what the macOS getattrlistbulk and batching work improved, what work still differs from dumac, whether a proven platform limitation exists, and which experiments falsified proposed ways to close the remaining wall-time gap. Rewrite the top-level PR description so each accepted change has mechanism, measured effect, semantic boundary, and current status; track any genuinely open follow-up.

## Notes

Full PR inventory reconciled against commits, experiment artifacts, source profiles, current main, and the definitive comparison. Draft PR body now distinguishes current-main deltas from cumulative campaign numbers, separates four batching layers, explains the statistically tied dumac result and per-directory kernel floor, discloses reverted experiments and known limits, and tracks H67 as fdu-ea8e. Review also found and fixed fdu-jnm8.
