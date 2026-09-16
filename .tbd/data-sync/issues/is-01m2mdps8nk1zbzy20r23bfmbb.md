---
type: is
id: is-01m2mdps8nk1zbzy20r23bfmbb
title: "PR #65 delta review PR65D-DOC-1: four statements say every stream record carries the classification"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-09-16T06:15:45.684Z
updated_at: 2026-09-16T07:15:44.608Z
closed_at: 2026-09-16T07:15:44.608Z
close_reason: "955ba6f: the cli-watch golden prose, CHANGELOG.md, SKILL.md, README.md and Change::ignored's doc now say an upsert and a rule-edit removal carry the bit, and that an ordinary removal, an invalidation and every record of a run that read no rules omit it."
resolution: null
duplicate_of: null
---
PR #65 delta review 5219107144. tests/golden/cli-watch.tryscript.md:31-36 prose, CHANGELOG.md:206, crates/fdu/src/skills/SKILL.md:177, crates/fdu-core/src/watch_session.rs:44-47. Only upserts and a reclassification removal carry ignored; a plain removal and an invalidation do not, and 'removal records carry no metadata' is now false for a reclassification removal. Change::ignored's doc says None means unobserved when it is also None where there is no entry to classify.
