---
type: is
id: is-01m6yj1khjc8g1rh4dw8zrs2qr
title: "PR #208 C2: check-supply-chain tolerates an expired exception silently"
kind: bug
status: open
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m6yj18ch6wszs8sv3gx41ncf
created_at: 2026-11-08T21:46:46.194Z
updated_at: 2026-11-08T21:51:41.104Z
---
Suggestion, outside the diff. scripts/check-supply-chain.mjs:42-71. https://github.com/jlevy/fdu/pull/208#issuecomment-6727541578

## Notes

Deferred from PR #208 review C (C2): a check-supply-chain change is supply-chain tooling outside that PR's scope. Proposed: fail validateExceptions (or warn) when an exception's expiresAt is in the past, so an entry like pulldown-cmark-0.13.6-rustsec-2026-0433 cannot linger after 2026-11-14; fdu-yy3g removes that entry meanwhile.
