---
type: is
id: is-01m48ym4mfnsewyhtws08fnjq7
title: "PR #174 B1: output guide still shows '8 languages below min share' (fdu-output-design.md:241)"
kind: bug
status: in_progress
priority: 3
version: 2
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m48ykwg1fz7tsw30h6xfhtmd
hold: null
hold_until: null
created_at: 2026-10-06T15:51:55.277Z
updated_at: 2026-10-06T15:52:00.075Z
started_at: 2026-10-06T15:52:00.074Z
---
Low. docs/project/architecture/fdu-output-design.md:241 shows the stale example '8 languages below min share'; output now prints '8 rows below min share' (tests/golden/cli-content.tryscript.md:697). Fix: change the example, optionally point to the multi-view form at :64. PR #174, review B: https://github.com/jlevy/fdu/pull/174#issuecomment-6020029670
