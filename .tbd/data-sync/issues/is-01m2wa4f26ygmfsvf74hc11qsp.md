---
type: is
id: is-01m2wa4f26ygmfsvf74hc11qsp
title: "H119: overlap first-pass analyze I/O with the metadata walk"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:15.141Z
updated_at: 2026-09-19T08:27:40.202Z
closed_at: 2026-09-19T08:27:40.201Z
close_reason: |
  H119 screened: walk overlap cannot clear 3%. content-basic sample (76,605 stacks): read 59.06%, __open 17.44%, fdu::scan 0.13%. openat leftover needs unsafe; not tonight. No engine change. Next: H117.
resolution: null
duplicate_of: null
---
content-basic still scans, then opens every admitted file by absolute path. Submit analysis from the walk or openat from a retained parent dirfd. Do not serialize workers (H79). Metric: content-basic wall or product --analyze=lines on metabrowser-clone. Accept: wall >=3% and CI below zero; digest identical. Profile before changing.

## Notes

H119 / exp-116 pre-register (2026-09-19), before profile and change.

Claim: first-pass analyze opens every admitted file by absolute path after the
metadata walk finishes. Overlapping I/O with enumeration (submit from the walk,
or openat from a retained parent dirfd) should cut content-basic wall.
Do not serialize workers (H79).

Job: content-basic wall on deciding-scale metabrowser-clone (component excludes
the setup scan; do not judge on it).
Accept: median at least 3% faster and 95% paired interval entirely below zero;
content digest identical; worker parallelism retained.
Control: HEAD c0b40564 (H115 in; H116 and H118 reverted). FDU_COUNTERS unset
for the claim-grade pair.

Profile before changing. Quiet first; if the start gate fails, uncontrolled
(allowed). No RAM disk. Revert engine on reject.
