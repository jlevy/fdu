---
type: is
id: is-01m4mpbtgcd3rc0rt0cbrefcfz
title: "PR #194 A6: Python evidence cannot show which wheel was which; one timing claim has no evidence and does not parse"
kind: bug
status: in_progress
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-pr194-finish
labels: []
dependencies: []
parent_id: is-01m4mpb135bvc7rjcgran42387
hold: null
hold_until: null
created_at: 2026-10-11T05:18:27.339Z
updated_at: 2026-10-11T21:59:10.958Z
started_at: 2026-10-11T05:18:36.604Z
---
Low. evidence/exp-217/python-modified-at.json:5,63; exp-217-...md:328-330. Fix: record each arm's commit or wheel identity and why both say 0.4.0; commit or reword the command-line and native timings. Review: https://github.com/jlevy/fdu/pull/194#issuecomment-6105735578

## Notes

The unsupported 10.0 s CLI and 11.4 s native list figures are withdrawn in exp-217; the committed end-to-end call and model-conversion figures are quoted instead. Both original wheels report 0.4.0, and no commit or wheel hash was captured. Deferred identity work waits on recovery of the original wheel/build attestations or a separately authorized, attested rerun; do not retrofit a claimed measured hash from the narrative. This limitation does not block merging the qualified record.
