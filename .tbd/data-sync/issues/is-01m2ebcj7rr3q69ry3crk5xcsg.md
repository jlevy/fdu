---
type: is
id: is-01m2ebcj7rr3q69ry3crk5xcsg
title: "PR #49 review FLOOR-12: parfloor's unreadable-directory and d_type gaps veto a subject"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:47.063Z
updated_at: 2026-09-13T22:05:54.165Z
closed_at: 2026-09-13T22:05:54.164Z
close_reason: "Fixed (harness side; parfloor.c unchanged and uncompiled on this macOS host): a disagreeing reference row (parfloor-enum) is dropped with its reason instead of vetoing, references never seed the oracle, an unreadable subject root is refused up front, and parfloor stat's unreadable-subdirectory gap is documented as a scoreboard limitation."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Low. explorations/benchmarks/spikes/parfloor.c:116-121, 133-141, 221 (invoked by floor.py) at 1fa2309. A directory parfloor fails to open is skipped without dirs++, while fdu and arena_spike count it, so any subject with one permission-denied directory is a guaranteed oracle veto; enum has no DT_UNKNOWN fallback; a root-open failure prints dirs = 2^64-1. Fix chosen (harness side; parfloor.c is Linux-only and cannot be compiled on the macOS host doing this work): a disagreeing parfloor-enum reference row is dropped rather than vetoing the subject, reference rows never seed the oracle, an unreadable subject root is refused up front, and the unreadable-subdirectory limitation of parfloor stat is documented.
