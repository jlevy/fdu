---
type: is
id: is-01m2ebbhx2c56v3s8d4y1wz9sa
title: "PR #49 review FLOOR-3: the quiet regime is asserted, not verified per trial"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:13.953Z
updated_at: 2026-09-13T21:50:53.832Z
closed_at: 2026-09-13T21:50:53.831Z
close_reason: "Fixed: entry uses measure._host_regime (refuses an unreadable load average), every trial is held to measure._host_pressure_snapshot/_host_pressure_reasons before and after, breaching trials are invalid, and any invalid measured trial downgrades the document to uncontrolled with the reasons rendered (downgrade chosen over refusal so the run survives as screening-grade)."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Medium. floor.py:406-428, 450-452, 490 at 1fa2309. _require_quiet runs once per subject before any trial, checks only load_1m/cpu, passes silently when load is unavailable, and host_pressure_after is recorded but never enforced -- while the loop guide's quiet contract holds before and after every sample. Fix: reuse measure._host_pressure_snapshot, measure._host_pressure_reasons and measure._host_regime (as compare_tools.py does), mark breaching trials invalid, downgrade the document's regime to uncontrolled when any measured trial breached, and refuse quiet when load is unavailable.
