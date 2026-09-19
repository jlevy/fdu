---
type: is
id: is-01m2x7xnqp8jhj3qjf6m2bh0qy
title: "H122: deciding-scale metadata CLI/walk profile after current engine"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T16:27:49.877Z
updated_at: 2026-09-19T19:59:54.747Z
closed_at: 2026-09-19T17:48:10.482Z
close_reason: "exp-118: confirmed, walk 96.6-97.5% of default-tree component on system-private-frameworks; leftover is __open plus getattrlistbulk; no Darwin cut; no engine change"
resolution: null
duplicate_of: null
---
Highest user-visible leverage. H108 left the default command as a cold walk (instrumented detached walk 1.292 s of 1.34 s, about 96%). Overnight optimized cache-hit restore and RSS, not that job. Profile first after the current engine (H115 + H120 in). Not H86. Not a snapshot load on fdu PATH.

Metric: deciding-scale installed fdu PATH / default-tree wall share on an immutable tree (system-private-frameworks or equivalent). Determination: the metadata walk is still at least 90% of wall, and names the leftover stage (enumerate, stat, consume).

What refutes: walk share below 90%, or the leftover is a stage already owned by a rejected hypothesis.

Why significant: this is the job users wait for. Overnight did not address it.

Protocol: docs/project/guides/performance-loop.md. Pickup: runbook Current Standing. Parent epic: fdu-8ya1.

## Notes

Confirmed exp-118; tighter leftover recorded as exp-122 (accepted determination). 77509 getattrlistbulk / 55256 dir_opens = 1.403 calls/dir including EOF. 20s sample: __open 50.36%, getattrlistbulk 18.95%, fdu::scan 3.02% (largest symbol 0.94%). No userspace cut >=3%. H125 not minted. dir_enumeration_calls kept (8dd95be8). Quiet gate 30.3%; pair uncontrolled. Do not retry as a snapshot load or consume trim.
