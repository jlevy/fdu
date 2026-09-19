---
type: is
id: is-01m2wa4f26ygmfsvf74hc11qsp
title: "H119: overlap first-pass analyze I/O with the metadata walk"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:15.141Z
updated_at: 2026-09-19T07:47:15.141Z
---
content-basic still scans, then opens every admitted file by absolute path. Submit analysis from the walk or openat from a retained parent dirfd. Do not serialize workers (H79). Metric: content-basic wall or product --analyze=lines on metabrowser-clone. Accept: wall >=3% and CI below zero; digest identical. Profile before changing.
