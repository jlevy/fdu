---
type: is
id: is-01m6yc9x14t8x0v0j2v2dc34jv
title: "Recent growth without a baseline: change time, age profiles, and volume capacity"
kind: epic
status: open
priority: 2
version: 31
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
child_order_hints:
  - is-01m6ycae17xp8ypt4jhhgx3am2
  - is-01m6ycaezvjs33xsh7fp1whpka
  - is-01m6ycafymrea5sgq8r7n621re
  - is-01m6ycagweqhppqznjzwagp365
  - is-01m6ycahyd69v7gadhehz9e7px
  - is-01m6ycajwxh4xpjv9f5qc0rx9m
  - is-01m2xbag4cjyqqvq776rnfxcyt
  - is-01m6yn4ejvcq7bxw43fd0em0cq
  - is-01m6z186j8bw9e8najpz11kf5d
  - is-01m70h1y3e66qv3grdtmfcr72b
  - is-01m70qy0sfzmpbqxd9tk4vmx1f
  - is-01m749dgh5vncr5vfym71twr50
  - is-01m74c8yycvgvjfyne7k65v8gv
  - is-01m74f0q44a22m0f5tt1y2chgy
  - is-01m74c9ghthtt4gggx5r9bbqct
  - is-01m74sny84wap88zdgvh79hxd1
  - is-01m75hvp13zk79cex02q85ffc1
  - is-01m75hvrx3h51gv51ze4afzjtp
  - is-01m75hvswdg31kybhy2kax4fp7
  - is-01m75v586ca3vv6vremva5h5er
  - is-01m75v690exesn86x6pv44ysx8
  - is-01m76a5gzs2zjjtvr524jrhs8h
  - is-01m76a5hn111z2jz5wvhx6jrxv
  - is-01m76a5j8sxem7yj05439sqh4b
  - is-01m76a5jzdcj68z4bmzfnbbw2q
  - is-01m76a5kpx8knnn8m15k2n43by
  - is-01m76atkmshagehk9v4crg7n46
  - is-01m76b043whs978xtx4vx3vne7
  - is-01m76kcnw131yavjmyhhcs4wwm
  - is-01m76mhgvsr7fg5vn9d0vhs21p
created_at: 2026-11-08T20:06:26.596Z
updated_at: 2026-11-12T01:04:20.344Z
---
Answer 'where did disk space go over a window?' from one metadata walk with no saved state, as the spec describes: phase 0 documents the recipe that works today; phase 1 adds change time (max of mtime and ctime) as a selection window and sort key plus a selection-coverage note; phase 2 adds the ages view (selected bytes partitioned by change-time band, every window from one walk); phase 3 adds the volumes view (capacity of each filesystem the report touches beside the bytes it counted; promotes fdu-eas0). Net change with deletions stays with checkpoints (fdu-8ybz).
