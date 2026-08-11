---
type: is
id: is-01kzs52k0k897fpdjngr9yhhh4
title: Python Session mirroring the Rust surface
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:34:02.002Z
updated_at: 2026-08-11T19:34:02.002Z
---
metabrowser is the driving consumer and a subprocess boundary would defeat the point of progressive results. Mirror start/report/is_complete/cancel/prioritize, with provenance on rows. Deterministic shutdown tests, as the existing concurrency tests do.
