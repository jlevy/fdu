---
type: is
id: is-01m3mg16k7j7mtx4sn4wzqfqgt
title: "Decide: --stale-ok answers are barely marked stale in plain text (-q removes the only marker)"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T17:12:05.989Z
updated_at: 2026-09-28T17:12:05.989Z
---
With --stale-ok, plain text says only 'cache only' in the perf footer, and -q removes it; docs/usage.md says the answer is 'labelled stale'. Not a regression. Decide whether plain text should carry a visible stale marker that -q keeps, or fix the docs.
