---
type: is
id: is-01m0nzwr6pcg3fnj1445zpc81z
title: "Address review: PR #42 — CLI on the public API, and the fdu/fdu-core rename"
kind: task
status: closed
priority: 1
version: 23
labels: []
dependencies: []
child_order_hints:
  - is-01m0nzxsrte7kj7d051qe0b63p
  - is-01m0nzxt55fy6m44xt4k10c634
  - is-01m0nzxthn96qvqjy6wevaf0my
  - is-01m0nzxtygcnx2tjrk2b8gzwcc
  - is-01m0nzxvb7zsac07x6bbdyzgwn
  - is-01m0nzxvqjdr1ddxdz70bmpwz8
  - is-01m0nzxw4dmd6gqfzqf3f6rsxv
  - is-01m0nzxwh7wcdp7r6dhh21a6fv
  - is-01m0nzxwxybtpqv61ksa5p9dev
  - is-01m0nzxxarrcvcpfpkwz63ca7y
  - is-01m0nzxxqj44qqhsv0qajgzt8v
  - is-01m0nzxy4a329jh4qvb21wdhag
  - is-01m0nzxygtwhjg67dbxn6tmnbm
  - is-01m0nzxyyhyhfjp67vtw1h0nqy
  - is-01m0nzxzan58earxsdzzw21hpv
  - is-01m0nzxzpqd47bf22af4rqn0ee
  - is-01m0nzy02sjfh6ttq1fv0v6dja
  - is-01m0nzy0fdwze239vnncxkz1a4
  - is-01m0nzy0w0ayf31872hbq2atjt
  - is-01m0nzy19r3f8gg1rwz29bc5zh
  - is-01m0nzy1p6w4322fjr367wxt91
created_at: 2026-08-23T00:21:26.358Z
updated_at: 2026-08-23T00:43:25.312Z
closed_at: 2026-08-23T00:43:25.312Z
close_reason: "All 19 findings and both suggestions addressed: 21 fixed, none deferred, and two findings corrected where the review's premise or its suggested fix did not survive verification (R6's cross-crate include! would not reach the published .crate; R13's 126 was the compared-session count and was right). Disposition map posted on PR #42. make check green end to end, and make release-rehearse -- which is not in the gate, and is where the packaging blocker hid -- now completes."
---
Structured review of PR #42 published as a PR comment. 2 Blocker, 3 High, 6 Medium, 8 Low findings plus 2 non-blocking suggestions. Each finding is a child bead; every one gets an explicit disposition (fixed, rebutted, or deferred).
