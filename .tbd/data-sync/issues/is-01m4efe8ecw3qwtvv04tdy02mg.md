---
type: is
id: is-01m4efe8ecw3qwtvv04tdy02mg
title: "PR #186 A7: refuse a Git LFS pointer or non-MP4 as the demo"
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
created_at: 2026-10-08T19:22:00.523Z
updated_at: 2026-10-08T19:22:26.395Z
started_at: 2026-10-08T19:22:26.394Z
---
Suggestion. stage_demo/demo_check: refuse content starting with the LFS pointer header or lacking ftyp at bytes 4..8. Review: https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629
