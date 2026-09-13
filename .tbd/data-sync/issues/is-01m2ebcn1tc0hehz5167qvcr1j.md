---
type: is
id: is-01m2ebcn1tc0hehz5167qvcr1j
title: "PR #49 review FLOOR-13: the offset justification double-counts the root"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:49.946Z
updated_at: 2026-09-13T22:09:01.330Z
closed_at: 2026-09-13T22:09:01.329Z
close_reason: "Fixed: the index tally note now says the root is among the 7,843 directories beside parfloor's 7,842, with a test that its arithmetic adds up; the PR body's entry counts are corrected with FLOOR-6."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Low. floor.py:203-204 and the PR body at 1fa2309. The index tier's tally note reads '84,536 = 7,843 dirs + 68,134 files + 8,559 symlinks + the root', but the first three already sum to 84,536 because the root is one of the 7,843. The -1 offset is right; its stated proof double-counts. Fix: correct the note and the PR text.
