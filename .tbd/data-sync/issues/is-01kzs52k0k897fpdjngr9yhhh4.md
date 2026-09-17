---
type: is
id: is-01kzs52k0k897fpdjngr9yhhh4
title: Python Session mirroring the Rust surface
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhcmhj09n41s1zae35yhm
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:34:02.002Z
updated_at: 2026-09-17T02:10:51.474Z
closed_at: 2026-09-17T02:10:51.473Z
close_reason: "Superseded by Python OpenedIndex (fdu.opened, PR #48)"
resolution: canceled
duplicate_of: null
---
metabrowser is the driving consumer and a subprocess boundary would defeat the point of progressive results. Mirror start/report/is_complete/cancel/prioritize, with provenance on rows. Deterministic shutdown tests, as the existing concurrency tests do.
