---
type: is
id: is-01m0y17vys1572ytke2y4fmp5b
title: "Map the rewrite to files, functions, tests, and PR #47 reuse"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels: []
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
hold: null
hold_until: null
created_at: 2026-08-26T03:18:54.680Z
updated_at: 2026-08-26T04:24:56.970Z
started_at: 2026-08-26T03:18:59.672Z
closed_at: 2026-08-26T04:24:56.968Z
close_reason: "The requested implementation map, complete PR #47 reuse audit, architecture/test decisions, MetaBrowser integration plan, and wired bead graph are committed and pushed to draft PR #48; local and hosted validation pass."
resolution: null
duplicate_of: null
---
Expand the approved design into an implementation-ready execution map. Pin every checkpoint to current-main files/functions and tests, inventory all PR #47 implementation commits as cherry-pick, selective extraction, test/fixture reuse, or rejection, identify remaining design gaps, and create a fully wired implementation bead graph.

## Notes

Completed on PR #48 through commits 2f15f9f and d7e96ba. Added the architecture document; expanded the plan to 1,961 lines with checkpoint criteria, current-main file/function ownership, MetaBrowser changes, composed integration, and a dependency-wired 22-bead execution graph; classified all 76 implementation-like PR #47 commits. Verified 9b31220 as the sole whole-commit candidate in an isolated worktree with fdu-core no-default and all-feature tests. Audited PR #47 test assets and selected a controlled transparent-box session design after comparing black-box matrices, a runtime trace bus, a full simulator, and broad dependency injection. The selected per-owner cfg(test) control drives real orchestration and records complete production values through declarative sessions. Local make check and all 19 GitHub CI jobs passed at d7e96ba.
