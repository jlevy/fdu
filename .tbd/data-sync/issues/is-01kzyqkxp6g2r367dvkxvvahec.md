---
type: is
id: is-01kzyqkxp6g2r367dvkxvvahec
title: Classify UTF-16 BOM input as unsupported encoding instead of generic binary
kind: bug
status: in_progress
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
hold: null
hold_until: null
created_at: 2026-08-13T23:34:16.517Z
updated_at: 2026-09-23T02:28:51.285Z
started_at: 2026-09-20T05:33:22.464Z
---
The completed content spec says UTF-16 BOMs are recognized and remain explicitly uncovered until decoding exists. Current NUL gating usually reports UTF-16 text as binary and has no unsupported-encoding coverage reason.

## Notes

Recovered in measured-value PR112: UnsupportedEncoding is a per-unit nonoperational coverage outcome, UTF-16/UTF-32 BOM recognition precedes generic binary handling, and sidecar codecs preserve it. Tests unicode_boms_are_nonoperational_unsupported_encoding_outcomes_per_unit, unsupported_encoding_boms_are_recognized_from_progressive_prefixes, and unsupported_encoding_outcomes_round_trip_through_the_sidecar cover the contract. Independent recovered-layer review complete; final composed gate/CI pending.
