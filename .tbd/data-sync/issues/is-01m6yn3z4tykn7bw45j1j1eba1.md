---
type: is
id: is-01m6yn3z4tykn7bw45j1j1eba1
title: "Change-time windows on every surface: request model, CLI, Python, watch, opened reads"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn411vxs4rbchbvbyc1zdz
  - type: blocks
    target: is-01m6yn41za441ay8y4vnmkd502
parent_id: is-01m6ycafymrea5sgq8r7n621re
created_at: 2026-11-08T22:40:29.338Z
updated_at: 2026-11-08T23:16:23.527Z
closed_at: 2026-11-08T23:16:23.527Z
close_reason: Done in 2757fdcb on claude/eloquent-galileo-dd771d
resolution: null
duplicate_of: null
---
RequestSpec/ReadSpec changed_since and changed_before through Request::build (same grammar and AxisNames as modified_*); CLI --changed-since, --changed-before, --sort changed with help text; Python Selection(changed_since=, changed_before=) and SortKey.CHANGED through _api and the native binding; Candidate construction in watch_session and opened/read carries change time. Parity: same answer from CLI and Python.
