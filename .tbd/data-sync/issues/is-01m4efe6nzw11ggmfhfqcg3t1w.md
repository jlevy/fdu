---
type: is
id: is-01m4efe6nzw11ggmfhfqcg3t1w
title: "PR #186 A3: notes_problems ignores releases/download/<tag>/ links"
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m4efe57c19n7q72g0edd20v4
hold: null
hold_until: null
created_at: 2026-10-08T19:21:58.717Z
updated_at: 2026-10-08T19:22:24.770Z
started_at: 2026-10-08T19:22:24.768Z
---
Low. scripts/release/maintainer.py notes_problems (~430-432): add releases/download/<tag>/ to pinned refs so a carried-over tag is caught. Review: https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629
