---
type: is
id: is-01m0eys26vbvz8ma71v3j3e2pf
title: Absolute and relative charts
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0eyrna93ewcb5nz1jh3gncc
created_at: 2026-08-20T06:47:15.930Z
updated_at: 2026-08-20T07:29:06.075Z
closed_at: 2026-08-20T07:29:06.073Z
close_reason: "Four hand-written inline SVG figures: anchored absolute wall time at five checkpoints with the re-measured baseline drawn as an error band; every experiment's paired effect with 95% interval against the accept threshold, axis sized to the data with nothing clipped; per-entry cost per subject with synthetic trees held apart; and wall-vs-CPU for the fifteen individual accepted wins. Two bugs found and covered by tests: an axis ladder that stopped below the data maximum, and a headline that crashed when a job was absent."
---
Self-contained inline SVG, no chart library. At minimum: an anchored absolute timeline in real milliseconds across the cumulative checkpoints; a per-entry normalized view so different tree scales and both platforms sit on one axis; and a dot-and-whisker of every experiment's paired effect with its 95% interval against the accept threshold. Adversarial or synthetic subjects must be visibly separated rather than silently averaged in.
