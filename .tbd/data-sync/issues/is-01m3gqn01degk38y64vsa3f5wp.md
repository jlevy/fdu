---
type: is
id: is-01m3gqn01degk38y64vsa3f5wp
title: Clarify README introduction and evaluate watch output invalidation
kind: task
status: closed
priority: 2
version: 10
delegate: claude-code
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-27T06:08:16.938Z
updated_at: 2026-09-28T16:20:47.008Z
started_at: 2026-09-27T06:10:20.708Z
closed_at: 2026-09-27T07:07:55.456Z
close_reason: "README introduction and Speed section rewritten and condensed; wall-clock/throughput tables clarified and rounded; Linux rerun setup documented without stale ranking claims. Native-watch duplicate repaint reproduced and tracked as fdu-wb5n; Linux run tracked as fdu-nffc. Commits 2b2a0f44 and 0b75a79b pushed to PR #132. Full make check, 335 benchmark tests, docs formatting, shell syntax checks, and all required CI passed. Task-owned smoke environments relocated to external scratch and imports verified; configurable paths tracked as fdu-fihm."
resolution: null
duplicate_of: null
---
Rewrite the README opening concisely from first principles, lead with supported macOS performance comparisons, move 0.x stability guidance below the introduction, evaluate watch output invalidation without changing engine behavior, and audit/improve the Linux performance summary against committed evidence. Opening rewrite committed as 2b2a0f44; local make check passed; CI running. Linux coverage review added at user request.

## Notes

Final precommit review: approve pending full gate. README Speed reduced from roughly 80 to 30 lines; detailed methodology/memory/semantics linked to full report. Linux setup is documented, not executed on this Mac; current rankings deferred to existing fdu-nffc. All presentation values checked by rendering preserved raw JSON, avoiding double rounding. No raw evidence, measurement arithmetic, dependencies, engine behavior, or public API changed. Renderer regression suite: 335 passing tests, including subsecond and low-rate values. Watch diagnosis remains fdu-wb5n. Full local gate running for final changes; initial README commit CI fully green.
