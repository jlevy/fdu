---
type: is
id: is-01m2gqn0dy0kpc3h1tvbe8f80t
title: Opened-root Report rows carry only the native path, so a report row's path does not pass back into a portable selection unchanged
kind: task
status: open
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h77qemaay1jkhbhfzh35me
created_at: 2026-09-14T19:52:35.517Z
updated_at: 2026-09-15T00:25:29.003Z
---
PR #57 review PR57-8W5K-1 (https://github.com/jlevy/fdu/pull/57#pullrequestreview-5200240760). At 7b804df, query_report.rs:595-606 and :869-870: after fdu-8w5k, every selection axis inside an opened read matches the portable path. Flat and aggregate rows carry the portable path, so a path from a page passes back into a filter unchanged. Report projection rows carry only the native path. For a name that is not valid UTF-8, a report row's path therefore does not match when passed back. This follows from the decision; it is not a defect in it. Possible API addition: give report rows a portable path field, as flat rows have, so every projection round-trips. Decide when a consumer needs report rows round-tripped.
