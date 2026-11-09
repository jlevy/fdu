---
type: is
id: is-01m6yn4cxyczxzkggvm3jfk0hp
title: Volumes tests and goldens
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn4dqpp7r5hqgmfs5ap59m
parent_id: is-01m2xbag4cjyqqvq776rnfxcyt
created_at: 2026-11-08T22:40:43.454Z
updated_at: 2026-11-09T00:17:20.530Z
closed_at: 2026-11-09T00:17:20.530Z
close_reason: Done on claude/eloquent-galileo-dd771d
resolution: null
duplicate_of: null
---
One filesystem gives one row; a tmpfs mounted below the root gives two where the test can mount (skip otherwise), and --one-filesystem gives one; no observation gives unknown; capacity never in a snapshot; --scan-depth 0 quick form; goldens with named patterns for host values; parity; cross-lint.
