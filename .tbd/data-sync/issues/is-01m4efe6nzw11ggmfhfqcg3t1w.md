---
type: is
id: is-01m4efe6nzw11ggmfhfqcg3t1w
title: "PR #186 A3: notes_problems ignores releases/download/<tag>/ links"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m4efe57c19n7q72g0edd20v4
hold: null
hold_until: null
created_at: 2026-10-08T19:21:58.717Z
updated_at: 2026-10-08T19:28:18.832Z
started_at: 2026-10-08T19:22:24.768Z
closed_at: 2026-10-08T19:28:18.832Z
close_reason: Fixed in 20fa9155; disposition https://github.com/jlevy/fdu/pull/186#issuecomment-6067458642
resolution: null
duplicate_of: null
---
Low. scripts/release/maintainer.py notes_problems (~430-432): add releases/download/<tag>/ to pinned refs so a carried-over tag is caught. Review: https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629
