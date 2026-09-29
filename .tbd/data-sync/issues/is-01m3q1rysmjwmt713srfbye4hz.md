---
type: is
id: is-01m3q1rysmjwmt713srfbye4hz
title: "macOS half of fdu-puk7: does the bulk reader's lstat fallback trigger autofs mounts?"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3q1rvcnwdw17kysjfag0f6g
created_at: 2026-09-29T17:00:39.092Z
updated_at: 2026-09-29T17:00:39.092Z
---
The bulk reader declines trigger directories to lstat; check whether that mounts on macOS and whether AT_NO_AUTOMOUNT-equivalent handling is needed.
