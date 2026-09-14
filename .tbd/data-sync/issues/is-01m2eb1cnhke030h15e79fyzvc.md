---
type: is
id: is-01m2eb1cnhke030h15e79fyzvc
title: "Address review: PR #54 — H86 Linux evidence stage record"
kind: task
status: closed
priority: 1
version: 14
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - perf
  - campaign-2
dependencies: []
child_order_hints:
  - is-01m2eb1x0dh9yvhnkfkwww55w8
  - is-01m2eb1xc9wtwxrn71ysbev4kk
  - is-01m2eb1xqycp33s623fb12c3nr
  - is-01m2eb2c9yzj0w7yjp5dxk2v10
  - is-01m2eb2cnp3v1qyh4zkbqqv6br
  - is-01m2eb2d0s1v2h7pz4ghp2hz24
  - is-01m2eb2dc1q43qa3tbysmeqkpe
  - is-01m2eb2sj3fhbxk4g8n3wnrnpg
  - is-01m2eb2sy2gjy29res0hj897ym
  - is-01m2eb2ta7pz62mk2705k9h56n
  - is-01m2eb2tnqsvj39phr0xe31swp
  - is-01m2esgt594wns69rqrjzzx0ej
created_at: 2026-09-13T21:33:40.912Z
updated_at: 2026-09-14T01:46:46.312Z
closed_at: 2026-09-13T22:07:52.389Z
close_reason: "All eleven PR #54 review findings (H86-1..H86-11) addressed in 46721fd, 86d2a6a, 4dccad8 and 2d36eab; CI 19/19 at 2d36eab; disposition map posted at https://github.com/jlevy/fdu/pull/54#issuecomment-5656484254. Deferred follow-up fdu-c4jr (per-arm max/min field and durable run JSON) stays open; H86 epic fdu-xde5 notes renumbered and corrected."
resolution: null
duplicate_of: null
---
Address formal review 5192260482 on PR #54 (claude/h86-linux-evidence-stage): findings H86-1 through H86-11. The verdict (floor gates reject H86 on Linux) stands; the record needs fixing: exp-102 id collision with #52 (renumber to exp-103, per #52 review PERF-2), orphaned measured commits (tagged perf/h86-linux-candidate and perf/h86-linux-control), unrecorded fdu-arm samples, evidence-page projection errors (synthetic flag, kept arm), and several misquoted or overstated figures. Branch brought up to date with #52 by merge, not rebase, so the measured commits stay reachable. Review: https://github.com/jlevy/fdu/pull/54#pullrequestreview-5192260482
