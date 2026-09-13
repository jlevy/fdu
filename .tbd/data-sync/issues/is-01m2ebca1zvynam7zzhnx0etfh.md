---
type: is
id: is-01m2ebca1zvynam7zzhnx0etfh
title: "PR #49 review FLOOR-10: the oracle's key set is frozen to the first trial"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:38.686Z
updated_at: 2026-09-13T22:05:53.778Z
closed_at: 2026-09-13T22:05:53.777Z
close_reason: "Fixed: the oracle is established after the rounds from the first trial reporting each key (never from a reference row), with per-key sources recorded, and every trial including warmups is held to it."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Low. floor.py:478-483 at 1fa2309. The oracle's keys come from the first instrument's first trial, so an order beginning with parfloor-enum (dirs only) never compares files or bytes between the instruments that emit them. Fix: grow the oracle with keys it has not seen, recording which instrument supplied each.
