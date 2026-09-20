---
type: is
id: is-01m2y7cf9fsawdtq8p5grnr3nq
title: Verify directory queries, surface/cache parity and stacked PR CI
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-20T01:37:40.650Z
updated_at: 2026-09-20T01:37:40.650Z
---
Add portable product goldens and Python/one-shot cold,warm,cache-only coverage for directory queries, one scan with changing names, mixed views, selection and bounds. Review expected output rather than blindly updating goldens. Run make docs-format and make check; investigate failures. Commit/push codex/directory-rollup-query, create PR against perf/campaign-next-2026-09-19 (#92), watch CI green, close finished beads and tbd sync.
