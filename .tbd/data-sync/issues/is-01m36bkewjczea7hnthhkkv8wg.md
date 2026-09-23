---
type: is
id: is-01m36bkewjczea7hnthhkkv8wg
title: Concurrent reconcile can drop a newer failure and leave a partial path with no error
kind: bug
status: closed
priority: 1
version: 3
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:19.377Z
updated_at: 2026-09-23T08:06:58.387Z
closed_at: 2026-09-23T08:06:58.386Z
close_reason: "Fixed in 8c247937 on #114; delta-reviewed; CI green"
resolution: null
duplicate_of: null
---
Stack review R114-1 (#114). index.rs finish_reconcile stamps retained issues with the pass start epoch (retain_issue_at(issue, started_at)) while drop_disproven_issues treats issue_epoch < started_at as older. Interleaving: pass A begins on root (epoch 5), pass B begins on x (6), A reads x, fails and finishes first (issue@5, x Partial@7), then B finishes clean: the mark at 7 survives but the issue at 5 is dropped. TreeStatus reports complete=false, coverage partial(inaccessible), errors=[] until a later root pass. Fix: stamp with the retention-time epoch and keep started_at only as the disproof threshold; add a deterministic interleaving test.
