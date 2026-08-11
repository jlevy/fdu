---
type: is
id: is-01kzs66sekegfybkcjb5k3drmz
title: Unit-test the watch persistence state machine directly
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vzdeychrseqy1t2qftr9
created_at: 2026-08-11T19:53:48.242Z
updated_at: 2026-08-11T19:53:48.242Z
---
Two of the three defects found on this branch were in the watch loop's save state machine - the throttle and the pending flag - and neither was caught by the tests, because every test drives that logic end to end through the spawned binary. An end-to-end test can only observe whether a file changed on disk; it cannot enumerate the transitions. The specific cases that slipped through: a dirty batch throttled below the interval followed by an idle tree (R5), and a save attempt that wrote nothing or failed while still clearing the pending flag (R7). Both are decisions, not I/O. Extract the decision from save_if_pending into a pure function over (pending, elapsed, save outcome) returning (next pending, whether to reset the throttle), and table-test every combination. The I/O stays where it is; only the branch logic moves, so the change is small and the coverage is exact. Filed after the second round of review on PR #5 found a regression the first round's fix had introduced, which is the signal that this logic needs tests of its own rather than more careful reading.
