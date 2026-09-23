---
type: is
id: is-01m363y726frrgtqtp2z8xanpm
title: Withdraw failed directory listing evidence consistently in cold and warm reports
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T03:11:23.161Z
updated_at: 2026-09-23T03:29:03.590Z
---
Final PR117 path-independence at 0d34e327 found 12 Linux cold/warm differences under unreadable src/nested: cached reconciliation retained children_complete=true and emitted known ages while cold emitted incomplete/unknown. Fix canonical listing evidence, preserve newer scoped verification and healthy siblings, and prove excluded failed subtrees do not taint eligible ancestor metrics. Keep registry empty; add real-permission regression and focused concurrency coverage.

## Notes

Implemented canonical listing-evidence withdrawal in 7e6e8acf. Failed retained directory listings invalidate that subtree; omitted metadata-failed entries invalidate their nearest retained parent listing only. Successful ancestor and sibling listings remain usable for query-time eligible-subtree composition. Older passes respect newer scope/epoch ownership. Parallel reconciliation now records successful formerly-incomplete listings so permissions recovery can restore completeness. Private tests cover real denied listing with serial/parallel workers, recovery, published Partial state, metadata-failure boundary, cold/warm equality, and newer-child arbitration. Fresh core suite: 749 passed, 1 ignored, no failures. Independent answer-agent production review clear; parent review and public List regression, final full gate and platform path-independence remain pending. No registry waiver added.

Public surface regression 1ad8b8fd verifies actual Off cold vs Auto/ReadOnly warm cache paths, incomplete ancestor/blocked age unknown, healthy sibling age known, and excluded failed-child pruning yielding complete eligible ancestors while root status stays partial. Both public tests pass with real PermissionDenied. Core clippy all-build-features/all-targets clean. Independent Astra answer-agent reviewed canonical and public commits with no findings. Parent review, cross-platform lint, final full gate and actual platform path-independence still pending.

Parent review identified an additional publication edge: an unscoped failure can withdraw listings while aggregate root state already remains Partial. Fixed in 8decb3bf with exact DirectoryIncomplete transitions, including Python kind/binding, transition path/dirty impact and normal bounded journal costing. Deterministic regression proves only these effects advance the clock when IndexState and cumulative DiscoveryProgress remain equal. Required opened contract coverage now includes a new failed-listing-and-recovery golden; generation and normal replay pass. Python model tests: 59 passed; final workspace all-build-features/all-targets clippy clean. make cross-lint passed macOS x86 and Windows before this additive transition followup; no new platform-gated production code. Parent final review, integration, full gate and actual platform path-independence remain required.
