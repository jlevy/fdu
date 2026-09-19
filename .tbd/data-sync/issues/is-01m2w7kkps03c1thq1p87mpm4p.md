---
type: is
id: is-01m2w7kkps03c1thq1p87mpm4p
title: "H115: one bottom-up roll-up pass after sidecar restore"
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0py2a8eb90n6r21f4hygyvr
created_at: 2026-09-19T07:03:05.687Z
updated_at: 2026-09-19T07:03:56.979Z
---
Named remaining H83 apply/install cut after H114 reject. Sidecar restore still rebuilds roll-ups per file times depth (1.12M merges on metabrowser). One bottom-up pass after all restore inserts. Do not retry H114 type-id String alloc. Screen content-cache-hit wall >=3% with CI below zero; digest identical.

## Notes

Queued after H114 / exp-111 reject.

Named mechanism: sidecar restore still rebuilds roll-ups per file times depth
(exp-108: 1,121,963 merges on metabrowser-clone). One bottom-up pass after all
restore inserts does the same work in O(files + dirs).

Metric: content-cache-hit wall on metabrowser-clone. Accept: median >=3% and
95% CI entirely below zero; content digest identical; intermediate directory
roll-ups unchanged.

Do not retry H114 type-id String alloc, parse-speed, instruction trims, or the
H113 file-count completeness shortcut. Do not start this as an instruction-only
trim.
