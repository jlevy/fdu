---
type: is
id: is-01m2f3b56x0e7sfe7dkamk1pzp
title: "perf-floor: meets_threshold still decides a subject the oracle vetoed or a document downgraded to uncontrolled"
kind: bug
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:38:26.780Z
updated_at: 2026-09-14T04:38:26.780Z
---
PR #49 delta review PR49-DELTA-2 (https://github.com/jlevy/fdu/pull/49#pullrequestreview-5193948340). floor.py:866-906, 940-963, 1041, 1143 at 6e018a0: FLOOR-4's fix nulls meets_threshold for spread only. When the tally oracle vetoes a subject (the numbers do not compare) or the document is downgraded to uncontrolled, the scoreboard still prints a decided pass or fail mark. The veto path is common: any unreadable directory (the FLOOR-12 limitation) triggers it. Fix: null meets_threshold whenever the subject is vetoed or the document is uncontrolled, and test both paths.
