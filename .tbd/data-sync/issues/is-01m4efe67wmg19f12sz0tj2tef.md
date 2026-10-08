---
type: is
id: is-01m4efe67wmg19f12sz0tj2tef
title: "PR #186 A2: README or notes link to fdu-demo.mp4 with no demo at COMMIT is not caught"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m4efe57c19n7q72g0edd20v4
created_at: 2026-10-08T19:21:58.267Z
updated_at: 2026-10-08T19:21:58.267Z
---
Medium. scripts/release/maintainer.py demo_check (~483-496): read README.md and the notes at COMMIT; fail when either links releases/latest/download/fdu-demo.mp4 or releases/download/vX.Y.Z/fdu-demo.mp4 and the commit has no docs/media/fdu-demo.mp4. Review: https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629
