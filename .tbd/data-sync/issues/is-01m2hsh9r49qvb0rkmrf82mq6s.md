---
type: is
id: is-01m2hsh9r49qvb0rkmrf82mq6s
title: "PR #55 review PR55-DUR-3: format retirement precondition is the migrate-on-upgrade alternative Q3 rejects"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2hsgt9d35edzxt2rqvfc3pv
created_at: 2026-09-15T05:44:45.571Z
updated_at: 2026-09-15T05:49:26.437Z
closed_at: 2026-09-15T05:49:26.434Z
close_reason: "e0aad48: a format's reader is retired only after a documented support window (releases since the last release that wrote it); retirement requires no re-encoding; compaction's re-encoding is optional and a pinned never-copied checkpoint keeps its format and is refused with the releases that read it; file-format requirement and open question 3 (recommendation and case against) updated"
resolution: null
duplicate_of: null
---
PR #55, delta review 5205945198. Plan @55ce4a3 :393-407 keeps every released format readable and lets compaction 'may' re-encode, then :402-405 conditions retirement on earlier releases having re-encoded every checkpoint they found. That is migrate-on-upgrade, which open question 3 (:626-634) rejects, and nothing makes re-encoding mandatory: a pinned, never-compacted checkpoint stays in its format, so no release can meet the precondition. Fix: make the precondition consistent with the chosen option, e.g. retire only after a documented number of releases have read the format, and a pinned never-compacted checkpoint either remains readable or is refused with a remedy.
