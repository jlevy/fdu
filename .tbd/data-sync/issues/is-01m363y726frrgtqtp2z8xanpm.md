---
type: is
id: is-01m363y726frrgtqtp2z8xanpm
title: Withdraw failed directory listing evidence consistently in cold and warm reports
kind: bug
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T03:11:23.161Z
updated_at: 2026-09-23T03:20:27.110Z
---
Final PR117 path-independence at 0d34e327 found 12 Linux cold/warm differences under unreadable src/nested: cached reconciliation retained children_complete=true and emitted known ages while cold emitted incomplete/unknown. Fix canonical listing evidence, preserve newer scoped verification and healthy siblings, and prove excluded failed subtrees do not taint eligible ancestor metrics. Keep registry empty; add real-permission regression and focused concurrency coverage.

## Notes

Implemented canonical listing-evidence withdrawal in 7e6e8acf. Failed retained directory listings invalidate that subtree; omitted metadata-failed entries invalidate their nearest retained parent listing only. Successful ancestor and sibling listings remain usable for query-time eligible-subtree composition. Older passes respect newer scope/epoch ownership. Parallel reconciliation now records successful formerly-incomplete listings so permissions recovery can restore completeness. Private tests cover real denied listing with serial/parallel workers, recovery, published Partial state, metadata-failure boundary, cold/warm equality, and newer-child arbitration. Fresh core suite: 749 passed, 1 ignored, no failures. Independent answer-agent production review clear; parent review and public List regression, final full gate and platform path-independence remain pending. No registry waiver added.
