---
type: is
id: is-01m2ebc188d1xnzfrsrmz433zt
title: "PR #49 review FLOOR-7: the quiet check trips on the harness's own build"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:29.671Z
updated_at: 2026-09-13T21:50:54.096Z
closed_at: 2026-09-13T21:50:54.095Z
close_reason: "Fixed: the regime entry polls within a stated bound (QUIET_WAIT_SECONDS=180, --quiet-wait) for a settling host before measure's entry check decides; chosen over 'check before building' because make now builds the probe before floor.py starts (FLOOR-5)."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Low. floor.py:671-679 at 1fa2309. run() builds the instruments and then checks quiet; a cold release build saturates every core and the 1-minute load needs ~83 s on 4 cores to fall under 0.25/core, so a first run refuses on an idle host. Fix chosen: poll with a stated, liftable bound until the bar is met -- 'check before building' no longer suffices once FLOOR-5 moves the probe build into make, which finishes before floor.py starts.
