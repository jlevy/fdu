---
type: is
id: is-01m2xqywv4ssfs89346n79jgz1
title: "H128: default-tree leftover on file-heavy metabrowser"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: cursor-agent
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T21:08:07.139Z
updated_at: 2026-09-19T21:12:29.720Z
closed_at: 2026-09-19T21:12:29.720Z
close_reason: "H128 confirmed (exp-127): file-heavy default-tree walk still the job (92.9% of component). 1.952 getattrlistbulk/dir. Snapshot not loaded. No new cut."
resolution: null
duplicate_of: null
---
Pre-registered 2026-09-19.

H128 / exp-127. default-tree leftover on frozen file-heavy metabrowser-clone
after H122 (frameworks) and H127 (opened-discovery).

Determination: walk share of deciding-scale default-tree component is or is
not still at least 90%; named leftover is or is not a new userspace cut.

Same H125 probe. Uncontrolled allowed if quiet fails. Do not invent a skip.
