---
type: is
id: is-01m6yn45sy1gvyfdtpgwvvnhvw
title: "Ages engine: view, bands, partition, cumulative tallies"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn46vk6e86k68fbyx8e4yc
parent_id: is-01m6ycahyd69v7gadhehz9e7px
hold: null
hold_until: null
created_at: 2026-11-08T22:40:36.158Z
updated_at: 2026-11-08T23:46:20.056Z
started_at: 2026-11-08T23:30:28.087Z
closed_at: 2026-11-08T23:46:20.056Z
close_reason: Done on claude/eloquent-galileo-dd771d
resolution: null
duplicate_of: null
---
Spec: The ages view. ViewSpec::Ages ('ages'); bands future, under 1h, 1h to 1d, 1d to 7d, 7d to 30d, 30d to 90d, 90d to 1y (365d6h), 1y to 3y (1095d18h), over 3y, unknown, relative to request.now with half-open edges (exact edge -> newer band; age 0 -> future); regular files only; files, bytes, allocated, ignored split, cumulative per band; several roots as their union; included in --view full; metadata-only.
