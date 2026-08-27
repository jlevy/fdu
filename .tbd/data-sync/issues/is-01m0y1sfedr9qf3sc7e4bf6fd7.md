---
type: is
id: is-01m0y1sfedr9qf3sc7e4bf6fd7
title: Measure a disposable fdu adapter against the unchanged MetaBrowser contract
kind: task
status: open
priority: 1
version: 12
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sfw7kwjprd6sfky281fj
  - type: blocks
    target: is-01m0y1shykye8sc7h7e9rkk6kh
  - type: blocks
    target: is-01m10nr8phjnfwhjak56e360gw
  - type: blocks
    target: is-01m10nrc1rnh7e8zzwx0z8r76c
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nq3f65ssfz0jj2nkxavrn
  - is-01m10nq3svzaygbyh3mvmkt0g7
  - is-01m10nq43t0s22axb5bsbhnnfy
  - is-01m10nq4dv23nfwsghy63h8628
  - is-01m10nq4rd347gj3makrvtrd5m
created_at: 2026-08-26T03:28:31.692Z
updated_at: 2026-08-27T03:57:23.336Z
---
On MetaBrowser PR #74, build a deliberately disposable adapter against the unchanged provider protocol. Instrument row materialization, sorting, scans, totals, latency, memory, and route-visible ordering on the representative corpus; publish evidence, retain the harness, and delete naive replica and aggregation code.

## Notes

Dedicated MetaBrowser spike branch codex/fdu-opened-root-e2e-spike was created from unchanged PR #74 head 3183888808b366b5ba1c381dec1cbb18b49d969e. Children bootstrap an exact-revision fdu wheel, build the disposable adapter, instrument both providers and routes, run the installed-wheel end-to-end lifecycle, then publish evidence and delete naive implementation code while retaining the harness.
