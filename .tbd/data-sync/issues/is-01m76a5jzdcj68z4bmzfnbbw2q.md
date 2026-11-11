---
type: is
id: is-01m76a5jzdcj68z4bmzfnbbw2q
title: "volumes: a stable machine label for each capacity-unknown reason"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T22:03:03.533Z
updated_at: 2026-11-11T22:03:03.533Z
---
Review J9 on #209 (optional suggestion). VolumeRow::unknown_reason is English text reworded several times on #209; give it an enum with a stable machine label beside the human text, e.g. unknown: {kind: not_a_mount_point, reason: ...}. Schema change: needs a report schema bump after 0.6.0.
