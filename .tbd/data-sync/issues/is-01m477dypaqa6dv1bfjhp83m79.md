---
type: is
id: is-01m477dypaqa6dv1bfjhp83m79
title: "Move inline report annotations (e.g. 'Percentage column: code lines') into the epilogue as notes"
kind: task
status: in_progress
priority: 1
version: 4
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m477dzjmz4414feb9mxpnnr4
  - type: blocks
    target: is-01m47846395xp250085ae705pe
parent_id: is-01m477dy519nmp2vvdjtg0n3z1
hold: null
hold_until: null
created_at: 2026-10-05T23:47:20.905Z
updated_at: 2026-10-05T23:59:29.384Z
started_at: 2026-10-05T23:47:22.427Z
---
Content views print 'Percentage column: code lines' / 'document words' / 'raw words' directly under the section heading on stdout (share_metric_note in crates/fdu-core/src/report_format.rs). Design rule: no unclassified notes in result output; every note or tip is a note:/tip: line after the output. Move these into the report epilogue (consolidated across views into one line), audit the renderer for any other inline annotations, and record the rule in docs/project/architecture/fdu-output-design.md and the report_format/report_epilogue module docs. Update goldens and Python parity.
