---
type: is
id: is-01kzkw4qwvn5q1vk9x01pj11xh
title: Keep Python independent of watch and use native Windows cache discovery
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzkw4ddv9g9jry50tp4xzgtw
created_at: 2026-08-09T18:21:43.195Z
updated_at: 2026-08-09T18:31:47.260Z
closed_at: 2026-08-09T18:31:47.260Z
close_reason: Implemented with regression coverage; the complete local handoff gate passes.
---
fdu-py enables the optional watch feature even though it exposes no watch API, violating the deletable-feature contract and increasing wheel dependencies. Remove it. Default cache discovery must honor XDG_CACHE_HOME as an override and use LOCALAPPDATA or USERPROFILE/AppData/Local on Windows instead of depending on HOME/.cache.
