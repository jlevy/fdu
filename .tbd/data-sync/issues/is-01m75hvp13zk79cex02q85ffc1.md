---
type: is
id: is-01m75hvp13zk79cex02q85ffc1
title: "Address PR #209 review H: volumes on routes that did not just walk (mount table, recorded root device)"
kind: task
status: closed
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
child_order_hints:
  - is-01m75hvpzkfp3w5ygsmabk6htq
  - is-01m75hvqx1x1pzwrgx67qfq796
  - is-01m75hvrx3h51gv51ze4afzjtp
  - is-01m75hvswdg31kybhy2kax4fp7
  - is-01m75hvtsaxd65a6vfqq1t5cpy
  - is-01m75hvvn1xeqc0jzjhmhgvcyq
  - is-01m75hvwf6k5zsxrsdnpgghhnv
  - is-01m75hvxa3mxb0q3gkw56x4k87
hold: null
hold_until: null
created_at: 2026-11-11T14:58:13.155Z
updated_at: 2026-11-11T16:21:42.544Z
started_at: 2026-11-11T14:58:34.889Z
closed_at: 2026-11-11T16:21:42.544Z
close_reason: Review H addressed on PR 209 at fa8ce99e; dispositions https://github.com/jlevy/fdu/pull/209#issuecomment-6801149845
resolution: null
duplicate_of: null
---
Review H (follow-up, round 3) on PR #209 at 6b96e475: https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952. Findings H1 (High), H2, H3 (Medium), H4-H7 (Low), H8 (suggestion). Coordinator decisions: H1 mount-table gate; H2 record the root device in the index and snapshot; H3 and H4/H8 remainder deferred to beads.
