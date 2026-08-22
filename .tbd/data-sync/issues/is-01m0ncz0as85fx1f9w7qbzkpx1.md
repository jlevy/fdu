---
type: is
id: is-01m0ncz0as85fx1f9w7qbzkpx1
title: Extract the artifact writer, ledger view, projection, and report renderer
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:37.272Z
updated_at: 2026-08-22T18:50:37.272Z
---
Move record.py, summary.py, timeline.py, report_html.py, and check_identifiers across. Lift fdu's anchor jobs, synthetic-subject set, metric columns, and chart set into config. Open question in the spec: whether the 1431-line SVG renderer travels or only the projection does.
