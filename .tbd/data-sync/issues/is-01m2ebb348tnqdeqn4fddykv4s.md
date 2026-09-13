---
type: is
id: is-01m2ebb348tnqdeqn4fddykv4s
title: "Address review: PR #48 — opened-root engine technical review"
kind: task
status: open
priority: 1
version: 29
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m2ebbzxr2pf419p03tfzqv5b
  - is-01m2ebc09kpwbynxzd5dqmj5mh
  - is-01m2ebc0nr76aax2mkq8h342q8
  - is-01m2ebc11er5wb1mb6xafrgd04
  - is-01m2ebc1ehdvn5vymvv23hkswq
  - is-01m2ebc1w7jdhmtt8q46j20tn8
  - is-01m2ebct59tw5f78dav1v4btcs
  - is-01m2ebcth0cebeeyb7e2b9y2bk
  - is-01m2ebctx6db7kbvryv8wxhdhb
  - is-01m2ebcv902e7hww3mk3wyenzq
  - is-01m2ebcvmctmh2ee0n0xy101jy
  - is-01m2ebcw0076801z6crbk1y6mg
  - is-01m2ebcweg6teba05mnc45dzpz
  - is-01m2ebcwt2kq5hb23gzev5cveg
  - is-01m2ebcx5mn29ngkmdh0vsb9h7
  - is-01m2ebdjw3vkxk0dppemsjcskw
  - is-01m2ebdk8je4aenkqz41fsq710
  - is-01m2ebdkt84sqbd23m3w5e707p
  - is-01m2ebdm962nswte313912pwxz
  - is-01m2ebdmy2cawv6vxnbv1fmqd3
  - is-01m2ebdnfwx1w3yds9yt4y1w72
  - is-01m2ebdp32y4vq1m3w2q02qbz5
  - is-01m2ebdpn17ywtkyt2yk18wsxp
  - is-01m2ebdq4c8z47yct91aand7rh
  - is-01m2ebdqgp1sz7kzknpewg7r7x
  - is-01m2eeh0t3339n8778x47ceswe
  - is-01m2eeh15s9phhkxjjs90vn25g
created_at: 2026-09-13T21:38:58.823Z
updated_at: 2026-09-13T23:40:04.193Z
---
Formal review 5192314101 at head c853f7c: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 — 2 Blocker, 2 High, 15 Medium, 14 Low across LIFE, READ, CLASS, PY. Every finding gets one child bead (or an existing bead with appended context) and an explicit fixed/rebutted/deferred disposition in the PR reply. Merge order: #48 must not reach main without #51's control-observation gate (CLASS-1); LIFE-6 and READ-4 land together.

## Notes

2026-09-13 status: disposition map posted at https://github.com/jlevy/fdu/pull/48#issuecomment-5657097329 for head f917cb7 (CI green, 19 of 19). 22 findings fixed and their beads closed. Deferred and still open: CLASS-1 on fdu-1onj, READ-5 on fdu-8w5k, READ-8 on fdu-91ru, CLASS-8 on fdu-0778 (context appended to each), and child beads LIFE-7 fdu-dkr0, LIFE-8 fdu-lfiy, LIFE-9 fdu-jxuq, LIFE-10 fdu-8jp0, CLASS-5 fdu-s0xg, CLASS-7 fdu-m5zj. This parent stays open while those children are. Found along the way and filed separately under fdu-snej: fdu-k18s (directories added after discovery are never marked complete).
