---
type: is
id: is-01m77at5fvxh4xght5ae76rwdz
title: "Issue #207: audit status after 0.6.0 and finish its small remaining improvements"
kind: task
status: in_progress
priority: 2
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-11-12T07:33:32.279Z
updated_at: 2026-11-12T10:37:16.487Z
started_at: 2026-11-12T07:33:37.965Z
---
Map every acceptance checkbox of https://github.com/jlevy/fdu/issues/207 to its 0.6.0 status and tracking bead, land the small finalize-now improvements in one focused PR, post a status comment on the issue, and list the larger items that need their own plan (query sessions, partial persistence, machine progress, baselines, FSEvents, large-scan costs). An outside contributor offered machine progress in the issue's only comment; that decision is the maintainer's.

## Notes

Audit done (Fable 5.2); status posted on #207 (issuecomment-6820624512). PR #211 at 79303828: reviews A and B approve, all findings addressed, unprivileged make -k check and CI green; ready for review/merge, awaiting the maintainer's merge decision. Follow-ups under epic fdu-ejc3 plus fdu-6lml.
