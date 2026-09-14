---
type: is
id: is-01m2f3t54tbn74vr2vv6yjb1rm
title: A complete refresh on a Failed opened root may drop the issue that explains the failure
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:46:38.234Z
updated_at: 2026-09-14T15:33:34.794Z
closed_at: 2026-09-14T15:33:34.793Z
close_reason: "81bea9f: settled by test -- apply_opened does not refuse commits on a Failed root, so a complete refresh did drop the explaining issue (observed: 0 retained). finish_reconcile now skips drop_disproven_issues while the phase is Failed; refresh stays allowed as on a Stopped root."
resolution: null
duplicate_of: null
---
PR #48 delta review DELTA48-ENG-3 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194005473), PLAUSIBLE. opened.rs:677-688 and index.rs:1767-1782 at d48b8f8. begin_refresh admits a refresh on a Failed root, and after 509b536 a complete pass drops pathed issues it disproves, which may include the issue that explains why the phase is terminal. Whether this can happen depends on whether apply_opened refuses every commit while the root is Failed. Settle it with a test: fail a root, refresh it, and check that the explaining issue survives or that the refresh is refused.
