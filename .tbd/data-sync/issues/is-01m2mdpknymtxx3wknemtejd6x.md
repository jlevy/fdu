---
type: is
id: is-01m2mdpknymtxx3wknemtejd6x
title: "PR #65 delta review PR65D-PERF-1: the published speed gate describes a head without #63's engine commits"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-16T06:15:39.965Z
updated_at: 2026-09-16T06:15:39.965Z
---
PR #65 delta review 5219107144. The gate table compares main against d95d729, whose merge-base with 9105768 is 1fd71a9, so none of #63's six engine commits were in the gated binary, and #63 published no gate. 581557c adds a file_name() compare and two BTreeMap::range probes per Op::Upsert on every scanner batch regardless of read_controls. Measure one interleaved 15-pair run at the merged head against origin/main on the control-free subject, and restate the PR body's speed-gate section as measured.
