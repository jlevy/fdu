---
type: is
id: is-01m4efe8ecw3qwtvv04tdy02mg
title: "PR #186 A7: refuse a Git LFS pointer or non-MP4 as the demo"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m4efe57c19n7q72g0edd20v4
created_at: 2026-10-08T19:22:00.523Z
updated_at: 2026-10-08T19:22:00.523Z
---
Suggestion. stage_demo/demo_check: refuse content starting with the LFS pointer header or lacking ftyp at bytes 4..8. Review: https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629
