---
type: is
id: is-01m4swms8xg4h4r2x9trvgptkq
title: Audit fdu help, embedded docs, and agent skill before 0.5.0
kind: task
status: in_progress
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-13T05:44:24.590Z
updated_at: 2026-10-13T05:55:41.870Z
started_at: 2026-10-13T05:44:36.408Z
---

## Notes

Independent strong-tier audit at main34253a9e found7 concrete gaps: moving agent fallback, API/reference routes, mixed watch schemas/invalidation, unsupported1y filter, overstated mtime watermark, optional-watch qualification, README typo/comparison. Fixes committed050c9575 on codex/self-documenting-release, draftPR #201. Follow-up caught stale usage fallback sentence (fixed) and Linux parity artifact (CI regeneration pending). Static checks passed: pinned docs-format/check, rustfmt, diffcheck, golden portability/invocations/observability, composed-skill fullgolden equality. Full localmakecheck and finalrelease stability blocked by external scratch33MiBfree. No tag/publication; user now authorizes release after gates.
