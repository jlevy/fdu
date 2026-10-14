---
type: is
id: is-01m4wrc27djab3g7teargkmsp9
title: Clarify OpenedIndex native ownership and Python result volume
kind: task
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T08:27:27.840Z
updated_at: 2026-10-14T08:56:00.913Z
started_at: 2026-10-14T08:28:16.509Z
closed_at: 2026-10-14T08:56:00.912Z
close_reason: PR205 now leads with OpenedIndex native ownership and coarse requests, documents rationale/checklist and source-grounded copy matrix, and tracks seven future improvements including legacy watch snapshots, batch conversion and optional buffers. Astra reviews clear; final CI passed at2d9b28f0. No code behavior changed or new performance measurements claimed.
resolution: null
duplicate_of: null
---
Follow-up to PR205: lead review and suggested user reply with OpenedIndex encapsulating inventory, observation and query costs in Rust. Explain selected projections/change polling, output-dependent Python allocation, opt-in complete results, and high-churn/retained-page caveats without implying zero-copy or guaranteed low volume.
