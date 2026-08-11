---
type: is
id: is-01kzs53egqjvdx9t3ckd9xe6nc
title: Resolve the composable-CLI spec's remaining surface questions
kind: task
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-11T19:34:30.165Z
updated_at: 2026-08-11T19:37:38.354Z
---
Spec open questions 1, 2, and 4, none of which block the shipped surface but all of which shape what gets added next. (1) Short flag for --view: -v collides with the verbose convention, so none is proposed; decide whether to leave it long-only. (2) Multiple roots per invocation, as fd and find allow: one index per root composes easily in the library, but the CLI ergonomics and the cache story are undesigned. (4) Whether a general --group-by ever surfaces once the reducer registry lands, generalizing the types view, or whether named views stay the entire vocabulary and new groupings arrive only as new views. Question 3 is already tracked as fdu-oqoy and fdu-jej9. Each answer is cheap to record and expensive to reverse once a flag ships.
