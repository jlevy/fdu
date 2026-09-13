---
type: is
id: is-01m2ebdnfwx1w3yds9yt4y1w72
title: "PR #48 review CLASS-6: which git ignore inputs are honored is undocumented"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:23.163Z
updated_at: 2026-09-13T21:40:23.163Z
---
Low. control.rs:32-33. .git/info/exclude, core.excludesFile, and nested repositories are neither implemented nor mentioned, and .gitignore files under ignored directories are read. Fix: one paragraph in control.rs naming exactly which git inputs are honored. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
