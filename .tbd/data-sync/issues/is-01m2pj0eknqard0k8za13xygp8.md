---
type: is
id: is-01m2pj0eknqard0k8za13xygp8
title: --analyze code|words buffers whole unknown-type and Markdown files
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels:
  - content
  - memory
dependencies: []
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-09-17T02:09:25.618Z
updated_at: 2026-09-17T02:09:25.618Z
---
content_analysis.rs holds the entire file for unknown-type files (.log, .dat, no extension) and for
Markdown. A 100 MiB .out file peaked at 131 MB RSS against 24 MB for `lines`; several workers on
multi-GB logs can exhaust memory. Opt-in and documented as analyzed through EOF. Drop the buffer once
the 16 KiB prefix settles the type; bound Markdown source retention.
