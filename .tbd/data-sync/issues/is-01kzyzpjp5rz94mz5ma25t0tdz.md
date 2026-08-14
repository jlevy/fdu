---
type: is
id: is-01kzyzpjp5rz94mz5ma25t0tdz
title: Add one-line text performance summary
kind: feature
status: closed
priority: 1
version: 5
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-14T01:55:32.164Z
updated_at: 2026-08-14T02:21:25.888Z
closed_at: 2026-08-14T02:16:26.623Z
close_reason: "Implemented in 9664230: exact one-line text performance footer with walked/read/fresh/cached metrics, docs, unit and golden coverage; make check and all PR checks pass."
---
Render one compact performance line at the end of human text reports. Include bytes/files walked, content bytes read, analysis throughput, read throughput, and cache-versus-fresh analysis work. Dim gray only when color is active; plain text without ANSI otherwise; omit from machine formats. Update design/docs and golden/e2e coverage.

## Notes

Implemented exact scan file/apparent-byte counters, actual analyzer read-byte and wall-time counters, content-sidecar hit bytes, and a one-line gray text footer with fresh/cache split, cache tier, rates, and total time. Machine formats, lifecycle/skill output, and watch omit it. Unit/integration/doc/no-default-features checks and all 99 golden CLI cases pass; design and user/agent docs are updated. PR #15 merged during finalization, so the clean follow-up is commit 4bce242 in PR #18.
