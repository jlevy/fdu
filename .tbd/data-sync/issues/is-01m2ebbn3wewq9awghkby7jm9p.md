---
type: is
id: is-01m2ebbn3wewq9awghkby7jm9p
title: "PR #49 review FLOOR-4: a spread-flagged tier can still be marked as meeting its threshold"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:17.243Z
updated_at: 2026-09-13T21:52:55.014Z
closed_at: 2026-09-13T21:52:55.013Z
close_reason: "Fixed: a spread-flagged tier's meets_threshold is None (chosen over evaluating against p95, which would conflate tail with modality) and renders ?≤threshold; the banner says such a tier is neither closed nor open."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Medium. floor.py:590, 627-633 at 1fa2309. score() sets meets_threshold = ratio <= threshold regardless of the spread flag, and render() prints a tick beside a flagged spread on the same row, so the machine-readable tier-closed signal can be true when the median landed in the lower mode (reproduced: index median 50 ms vs 40 ms floor, flagged, meets_threshold=True). Fix: when the spread flag is set, meets_threshold is None and renders '?'.
