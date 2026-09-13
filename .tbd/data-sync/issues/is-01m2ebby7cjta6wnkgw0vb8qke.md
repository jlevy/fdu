---
type: is
id: is-01m2ebby7cjta6wnkgw0vb8qke
title: "PR #49 review FLOOR-6: entries means two different things across the campaign"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:26.571Z
updated_at: 2026-09-13T22:09:00.702Z
closed_at: 2026-09-13T22:09:00.700Z
close_reason: "Fixed: entries uses the perf-subjects definition (root + dirs + files + other, from parfloor stat), checked against tree.fingerprint on a real tree; the dir+file denominator stays and is labeled ns/(dir+file); the report gives /usr as 84,536 entries and no longer classifies /opt, whose count under that definition was not recorded."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Medium. floor.py:497, 582 and report-2026-08-28-linux-floor-scoreboard.md at 1fa2309. floor.py's entries = dirs + files, excluding the root and symlinks (/usr = 75,976), while tree.py and subjects.py count the root, symlinks and other kinds (/usr = 84,536), and subjects.py applies the 50,000-entry deciding bar to that count -- so the report classifies /opt as screening (47,819 < 50,000) under a different definition than the nominating tool. ns/entry also divides by a denominator ~11% smaller than the floor's statx count. Fix: entries uses the subjects.py definition; the narrower denominator stays and its column is labelled ns/(dir+file); the /opt classification is reconciled in the report and PR text.
