---
type: is
id: is-01m4pf47nqgznzznhvgs08kkjg
title: Stabilize and merge the 0.5.0 stack and prepare release readiness
kind: task
status: in_progress
priority: 1
version: 4
delegate: codex@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-11T21:50:27.510Z
updated_at: 2026-10-12T00:26:39.168Z
started_at: 2026-10-11T21:50:41.640Z
---

## Notes

Merged PRs #191, #192, #194, #195, #196, and #197 with required independent reviews and green PR CI. The primary main checkout is clean at release-preparation merge 33a3a890. Full integrated check and cross-lint passed; release stability gates, 49 installed CLI checks, 38 pty checks, three terminal tests, and the correctness matrices passed. Required wider Library peer QA failed twice, tracked fdu-wo7c; fdu-4arl Linux attribution and human terminal Phase 6 remain open before release disposition. Non-publishing release rehearsal run 38187708393 is in progress; release-body and preflight passed. Evidence documentation is being prepared separately. No tag or publishing dispatch has occurred.
