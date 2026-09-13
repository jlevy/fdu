---
type: is
id: is-01m2ebbstqqvy5pjd26x37esd9
title: "PR #49 review FLOOR-5: the floor's probe build diverges from make perf-probe-release"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:22.070Z
updated_at: 2026-09-13T21:58:35.090Z
closed_at: 2026-09-13T21:58:35.090Z
close_reason: "Fixed: perf-floor depends on perf-probe-release and passes --probe \"$(PERF_RELEASE)\"; floor.py builds only the spikes and refuses a missing probe naming make perf-probe-release. The #52 Makefile conflict is left for merge time. The review's 'consider --no-oracle after #52' is not applicable on this branch: the flag (fdu-4xtm) lives on the #52 stack."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Medium. floor.py:264-272 and the Makefile perf-floor target at 1fa2309. build_instruments duplicates the perf-probe-release cargo line under a comment claiming it builds the same probe; PR #52 adds --features gitignore to perf-probe-release, after which the comment is false, alternating perf-compare and perf-floor recompiles fdu-core with the feature flipped, and the two probes do different work on trees with control files. Fix: perf-floor depends on perf-probe-release and passes the built binary's path; floor.py no longer builds the probe. PR #52 edits the same Makefile lines 391-397 -- that textual conflict is left for merge time. The review's 'consider --no-oracle for the index tier after #52' cannot apply here: the flag (fdu-4xtm) lives on the #52 stack, not on main.
