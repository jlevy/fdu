---
type: is
id: is-01m2mcrqayt1akwe3dzr2fk860
title: "PR #63 delta review PR63D-DOC-2: control_refused does not count a refused re-read the inert check dismisses"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-16T05:59:20.669Z
updated_at: 2026-09-16T06:19:17.035Z
closed_at: 2026-09-16T06:19:17.034Z
close_reason: "3630f5c: Counts::control_refused documents that it is counted where the table decides, so a re-read of an already refused file is not counted and a warm revalidate of a refused tree reports its reads against zero refusals."
resolution: null
duplicate_of: null
---
Delta review 5218970886 at 9105768: counters.rs:57-58, control.rs:307-310, index.rs:3777. control_refused is bumped inside upsert_identified; after the inert-batch skip in 581557c a refused re-read that controls_unchanged_by dismisses never reaches it, so a warm revalidate of an over-budget tree reports N control reads and 0 refusals. Say so where the counter is documented.
