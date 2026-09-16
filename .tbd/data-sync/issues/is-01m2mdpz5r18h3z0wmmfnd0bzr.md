---
type: is
id: is-01m2mdpz5r18h3z0wmmfnd0bzr
title: "PR #65 delta review PR65D-WATCH-1: a default watch leaves each row's ignored bit stale after a rule edit"
kind: bug
status: open
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T06:15:51.734Z
updated_at: 2026-09-16T06:15:59.544Z
---
PR #65 delta review 5219107144. crates/fdu-core/src/watch_session.rs:207-212,:322; crates/fdu/src/cli.rs:773-774,:831-833. Under the default IgnoredEntries::Include with --view files only, a rule edit leaves the consumer's per-row ignored bit stale: batch_facts reads no Reclassified entry unless the selection filters by it, change_for returns None, and has_aggregates is false so dirty repaints nothing. Membership is still correct, which is F1's contract. The comment's 'nothing about its row changed' is inaccurate and is corrected in this PR; making the bit live under Include is the follow-up.
