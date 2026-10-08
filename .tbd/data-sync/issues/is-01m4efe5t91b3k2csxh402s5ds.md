---
type: is
id: is-01m4efe5t91b3k2csxh402s5ds
title: "PR #186 A1: real-git release tests inherit GIT_* and user git config, can commit into an outer repo"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m4efe57c19n7q72g0edd20v4
created_at: 2026-10-08T19:21:57.832Z
updated_at: 2026-10-08T19:21:57.832Z
---
Medium. tests/release/test_maintainer.py ShowBytesTests (~1367-1399) and SigningRecipeTests (~906-926): clear GIT_*, GIT_CONFIG_GLOBAL=devnull, GIT_CONFIG_NOSYSTEM=1, commit --no-verify. Review: https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629
