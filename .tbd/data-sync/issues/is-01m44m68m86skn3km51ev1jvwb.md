---
type: is
id: is-01m44m68m86skn3km51ev1jvwb
title: Bump tryscript to 0.3.0 and remove the fast-glob stand-in and override
kind: task
status: closed
priority: 2
version: 4
spec_path: packages/cli-animate/docs/project/specs/active/plan-2026-10-04-cli-animate.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m44jm7xwyb4m5vqpf3ethg4w
hold: null
hold_until: null
created_at: 2026-10-04T23:32:37.127Z
updated_at: 2026-10-05T01:44:12.469Z
started_at: 2026-10-04T23:32:37.714Z
closed_at: 2026-10-05T00:59:49.276Z
close_reason: Done in 7af8ab3f (jlevy/fdu#171)
resolution: null
duplicate_of: null
---
tryscript 0.3.0 (first-party) drops fast-glob per jlevy/tryscript#56; remove scripts/shims/fast-glob, its devDependency and override; npm audit stays clean.
