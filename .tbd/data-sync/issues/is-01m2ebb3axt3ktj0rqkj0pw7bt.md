---
type: is
id: is-01m2ebb3axt3ktj0rqkj0pw7bt
title: "Address review: PR #49 — the Linux floor scoreboard"
kind: task
status: closed
priority: 1
version: 21
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
dependencies: []
parent_id: is-01m0p4ya63vm7c49hw49pt4xaw
child_order_hints:
  - is-01m2ebbacqb1qqy526s0bh06k1
  - is-01m2ebbe9ggqs7c0b3qqk7616v
  - is-01m2ebbhx2c56v3s8d4y1wz9sa
  - is-01m2ebbn3wewq9awghkby7jm9p
  - is-01m2ebbstqqvy5pjd26x37esd9
  - is-01m2ebby7cjta6wnkgw0vb8qke
  - is-01m2ebc188d1xnzfrsrmz433zt
  - is-01m2ebc4enza063pqwwraqy8dv
  - is-01m2ebc7j9b7w94d6y9jp0wwgp
  - is-01m2ebca1zvynam7zzhnx0etfh
  - is-01m2ebcdxhc9j53kv6y1pg7sqn
  - is-01m2ebcj7rr3q69ry3crk5xcsg
  - is-01m2ebcn1tc0hehz5167qvcr1j
  - is-01m2esgr7z3zqmx7fskdj3kfzx
  - is-01m2esgrngd4vjkprfqa8xq2yg
  - is-01m2esgs0skcccfsvsej970xcp
  - is-01m2esgscrc17cgwzsp4tec58c
  - is-01m2esgsrjvv641ntm9mc8zfxk
created_at: 2026-09-13T21:38:59.036Z
updated_at: 2026-09-14T01:46:45.905Z
closed_at: 2026-09-13T22:17:33.174Z
close_reason: "All 13 findings fixed on claude/perf-floor-linux-2026-08-28 through 6e018a0 (CI 19/19 green); disposition map posted at https://github.com/jlevy/fdu/pull/49#issuecomment-5656540687. Also fixed outside the review: make perf-floor defaulted to the uncontrolled regime. PR #52's Makefile conflict is left for merge time."
resolution: null
duplicate_of: null
---
Formal review 5192251516 on PR #49 (https://github.com/jlevy/fdu/pull/49#pullrequestreview-5192251516), head 1fa2309: 13 findings FLOOR-1..FLOOR-13 (one High, five Medium, seven Low) plus 17 proof tests, all in the floor harness (explorations/benchmarks/realtree/floor.py), its tests, the Makefile perf-floor target, and the scoreboard report. One child bead per finding; each gets exactly one disposition (fixed, rebutted, deferred) in the PR comment that closes the loop.
