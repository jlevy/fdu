---
type: is
id: is-01m2ebbe9ggqs7c0b3qqk7616v
title: "PR #49 review FLOOR-2: a fixed instrument rotation, not an interleave"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:10.255Z
updated_at: 2026-09-13T21:45:39.209Z
closed_at: 2026-09-13T21:45:39.208Z
close_reason: "Fixed: rounds follow a carryover-balanced schedule (every instrument follows every other equally often, boundaries included, never itself), recorded per subject. The suggested ordinal % n rotation was not used because a cyclic shift preserves adjacency (simulated: 26-27 of 33 rounds behind one predecessor)."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Medium. floor.py:459-471 at 1fa2309. Every round runs parfloor-stat, parfloor-enum, arena-spike, aggregate, index in that order, so each instrument always follows the same predecessor (parfloor-stat always follows index, the largest memory churner) -- while the PR attributes arena_spike's bimodality to the preceding process. The review's suggested ordinal % n rotation does not balance predecessors (a cyclic shift preserves adjacency: simulated at 5 instruments x 33 rounds, each instrument still follows one predecessor 26-27 times and never meets two of the other four). Fix: a carryover-balanced schedule where every instrument follows every other equally often, round boundaries included, recorded in the JSON.
