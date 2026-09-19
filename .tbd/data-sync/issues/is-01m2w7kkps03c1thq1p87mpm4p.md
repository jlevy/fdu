---
type: is
id: is-01m2w7kkps03c1thq1p87mpm4p
title: "H115: one bottom-up roll-up pass after sidecar restore"
kind: task
status: closed
priority: 1
version: 6
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m0py2a8eb90n6r21f4hygyvr
hold: null
hold_until: null
created_at: 2026-09-19T07:03:05.687Z
updated_at: 2026-09-19T07:25:32.121Z
started_at: 2026-09-19T07:11:57.622Z
closed_at: 2026-09-19T07:25:32.120Z
close_reason: "exp-112 accepted: content-cache-hit wall -9.69% [-26.02%, -7.13%]; restore-only bottom-up rebuild kept at 7798fdc1"
resolution: null
duplicate_of: null
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

H115 / exp-112 accepted (2026-09-19).
Wall -9.69% [-26.02%, -7.13%] on deciding-scale metabrowser-clone.
User CPU -8.23% [-11.11%, -7.73%]. Digest identical (3b8cfa71).
Engine kept at 7798fdc1. Quiet refused; unlabeled quiet; pair was uncontrolled.
Do not retry. Parent fdu-jxhk remains the EntryId composite.
