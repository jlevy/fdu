---
type: is
id: is-01m2ebc4enza063pqwwraqy8dv
title: "PR #49 review FLOOR-8: max/min >= 2 over-claims modality"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:32.948Z
updated_at: 2026-09-13T21:52:55.288Z
closed_at: 2026-09-13T21:52:55.287Z
close_reason: "Fixed: renamed spread_suspect, compares the unrounded max/min ratio, and the comment and banner claim a second mode or an outlier rather than more than one population."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Low. floor.py:524, 530, 640-646 at 1fa2309. multimodal_suspect = round(max/min, 2) >= 2.0, but the banner says the median summarizes more than one population: one 81 ms outlier among 29 x 40 ms samples flags, a true 15/15 bimodal at 40/70 ms does not, and 1.995 rounds up and flags. Fix: rename to spread_suspect, compare the unrounded ratio, and reword the banner to claim only what max/min shows.
