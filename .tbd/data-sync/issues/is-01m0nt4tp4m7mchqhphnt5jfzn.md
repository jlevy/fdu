---
type: is
id: is-01m0nt4tp4m7mchqhphnt5jfzn
title: "PR #40 review R4: parity artifact header documents four gaps this PR closed"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:40:59.587Z
updated_at: 2026-08-22T23:14:22.736Z
closed_at: 2026-08-22T23:14:22.736Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
scripts/run-parity.mjs:117. HEADER is a hand-written constant re-emitted on every --update, so CI byte-comparison can never catch it. Also misdescribes the skip list.
