---
type: is
id: is-01m0prhpj01wa15ypxm0er2q6s
title: Walk telemetry as typed values in Python
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:18.623Z
updated_at: 2026-08-23T07:32:18.623Z
---
Expose the walk telemetry the CLI footer already computes (files and bytes walked, cache tier, fresh vs cached analysis) as typed values delivered beside report/session/watch results, never inside the versioned envelope. Embedded clients run measured loops of their own and need the same evidence.
