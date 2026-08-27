---
type: is
id: is-01m0xs2ffhy8av1qm0dn9kyc31
title: Implement the opened-root inventory engine rewrite
kind: epic
status: in_progress
priority: 1
version: 41
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/48
    at: 2026-08-26T01:54:36.597Z
delegate: codex@spud10.local
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
child_order_hints:
  - is-01m0xyqrr2t9q75j8v9q7v6kwj
  - is-01m0xy0ag289f93fkrhqknc5w1
  - is-01m0y17vys1572ytke2y4fmp5b
  - is-01m0y1sawtthrp0bq2agcv07f8
  - is-01m0y1sb9w9kbd0rsdq8sq3xyc
  - is-01m0y1sbmnpxmcyt3rvm7qt1rg
  - is-01m0y1sc1h17y99grptjb9pzha
  - is-01m0y1sce6j2y5ac1nzgtsdwsx
  - is-01m0y1scsff119ypyb93tbbnxh
  - is-01m0y1sd4tmt95d9tdsynn7v6g
  - is-01m0y1sdf88k6zme7tb9hbkjrd
  - is-01m0y1sdsamq2hrrexxcvx98g3
  - is-01m0y1se38tcc11akkz34mjrme
  - is-01m0y1sed8zrkf6hdnp5wrq5ty
  - is-01m0y1seqtkvcawjhawny979ry
  - is-01m0y1sf2nph021wtx28p8ahxh
  - is-01m0y1sfedr9qf3sc7e4bf6fd7
  - is-01m0y1sfw7kwjprd6sfky281fj
  - is-01m0y1sg8emg0sgyv1pj8sa6x7
  - is-01m0y1sgqd1sd33stssgw25f2q
  - is-01m0y1shykye8sc7h7e9rkk6kh
  - is-01m0y1sjbfs5h264xhme2vqymg
  - is-01m0y1sjnptgqhgvqcx1cjkkhw
  - is-01m0y1sk24z37hnvpxee6apg8e
  - is-01m0y1skdmrymm5zphnstcdn86
  - is-01m0y7ckn0gq2gy1fve4t50xvm
  - is-01m0y9g99mn8nzj7ydyt6yj478
  - is-01m0ydh0b0dw4xhf37y4gxm1jb
  - is-01m0yhq8268z0qrza1fnwrddfm
  - is-01m10afnckpkf0m2fnhgw3sx1d
hold: null
hold_until: null
created_at: 2026-08-26T00:56:09.456Z
updated_at: 2026-08-27T03:57:23.636Z
started_at: 2026-08-26T01:54:35.977Z
---
Implement the clean opened-root and streaming design from the linked plan in merge-sized slices from current main. Preserve the one-shot engine and CLI defaults; first establish exact commits and ownership, then the minimal opened-root lifecycle, then adopt it through MetaBrowser's provider conformance boundary.

## Notes

Status after planning reconciliation: native Phase 2 and its five session goldens are complete and green at fdu PR #48 head 86f64eb (GitHub Actions run 33035356756, all 19 checks). Remaining Python, unchanged-contract MetaBrowser spike, joint contract, reference provider, native indexes, thin adapter, cross-provider semantics, installed-wheel product proof, and final acceptance are now decomposed into commit-sized child beads under their checkpoint parents. Next executable slice is fdu-seku under fdu-bnsk; MetaBrowser spike branch codex/fdu-opened-root-e2e-spike is pinned to PR #74 head 3183888 and waits for the Python checkpoint.
