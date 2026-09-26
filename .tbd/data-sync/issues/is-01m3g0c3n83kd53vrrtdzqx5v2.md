---
type: is
id: is-01m3g0c3n83kd53vrrtdzqx5v2
title: Define managed-store exclusion for whole-home replay
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-26T23:21:28.487Z
updated_at: 2026-09-26T23:21:28.487Z
---
A macOS whole-home scan contains the default snapshot path under ~/Library/Caches/fdu, and the checkpoint store is also user-local. Persisting either can create self-generated FSEvents and make fdu account for its own changing state. Model one engine-owned managed-store exclusion policy shared by cold scans, full revalidation, scoped replay, checkpoints, CLI, and Python; report excluded store bytes separately and test roots that contain the stores.
