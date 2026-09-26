---
type: is
id: is-01m3fn7cdtnmdgjy9aff65x1c3
title: Evaluate code SLOC accuracy against established counters
kind: task
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-26T20:06:39.289Z
updated_at: 2026-09-26T20:12:40.403Z
started_at: 2026-09-26T20:06:58.943Z
closed_at: 2026-09-26T20:12:40.389Z
close_reason: Completed 30 manually classified adversarial fixtures and exact-file comparison over 107 real sources with fdu0.1.0/Tokei14. Evidence and findings prepared for research brief. Confirmed parser bugs tracked in fdu-ov8o, fdu-f1m3, fdu-lr38. Adjudicated large scan.rs discrepancy as Tokei character-literal bug; aggregate disagreement is not an accuracy score.
resolution: null
duplicate_of: null
---
Read-only differential analysis of fdu 0.1.0 code-sloc-v1 against installed Tokei, matched real source and manually classified language edge cases. Distinguish intentional semantics from parser defects; produce reproducible evidence and follow-up recommendations.
