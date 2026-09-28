---
type: is
id: is-01m3mcwvbyhdpbhtakmadmqq0w
title: "0.2.0 rehearsal: release.yml on a ref at 6ec77163, artifacts verified"
kind: task
status: closed
priority: 0
version: 4
delegate: claude-code
labels: []
dependencies:
  - type: blocks
    target: is-01m3mcwvsynn6f2sezxwzndq1n
parent_id: is-01m3mcwtkbvr9kj5j2qwpyyd3j
hold: null
hold_until: null
created_at: 2026-09-28T16:17:17.693Z
updated_at: 2026-09-28T16:26:43.603Z
started_at: 2026-09-28T16:18:29.597Z
closed_at: 2026-09-28T16:26:43.601Z
close_reason: Rehearsal run 36450043255 on release/v0.2.0 (pinned at 6ec77163) succeeded; headSha 6ec77163a8a1; all 8 artifacts verified against SHA256SUMS.
resolution: null
duplicate_of: null
---
Dispatch the rehearsal on a branch pointing at 6ec77163 (main has moved on to #149), record the run and head SHA, download artifacts, verify SHA256SUMS: both crates, the sdist, five wheels.
