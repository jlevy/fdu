---
type: is
id: is-01m2pj058tjrqdmdexgt0r88q6
title: Carry the requested analyzer set to reports and project retained content to it
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - content
  - deferrable
dependencies: []
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-09-17T02:09:16.057Z
updated_at: 2026-09-17T02:57:30.447Z
---
Add the requested set to the report request (Rust Query/ReportRequest, Python, CLI); project metric
fields, document-word derivation, and analysis.analyze/analyzers to the request; Unsupported records
never answer a set they cannot. Guard with a property test: for every stored set S and requested
R ⊆ S, report(R, warmed by S) equals report(R, cold), for CLI JSON and Python as_dict.
