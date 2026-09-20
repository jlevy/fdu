---
type: is
id: is-01kzyqkxp6g2r367dvkxvvahec
title: Classify UTF-16 BOM input as unsupported encoding instead of generic binary
kind: bug
status: in_progress
priority: 3
version: 2
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
hold: null
hold_until: null
created_at: 2026-08-13T23:34:16.517Z
updated_at: 2026-09-20T05:33:22.466Z
started_at: 2026-09-20T05:33:22.464Z
---
The completed content spec says UTF-16 BOMs are recognized and remain explicitly uncovered until decoding exists. Current NUL gating usually reports UTF-16 text as binary and has no unsupported-encoding coverage reason.
