---
type: is
id: is-01m2y7c1v3etw9wkzz87vgm6ke
title: Render directory size and actual age across text, machine formats and Python
kind: feature
status: in_progress
priority: 1
version: 5
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7c6yzccahx1ntvy2wf1s1
  - type: blocks
    target: is-01m2y7cbprn6w292gjenv0twd7
  - type: blocks
    target: is-01m2yhsw73csne6aef665mmp4n
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
hold: null
hold_until: null
created_at: 2026-09-20T01:37:26.880Z
updated_at: 2026-09-20T04:39:45.634Z
started_at: 2026-09-20T01:39:25.996Z
---
Add text size/age/path and JSON/JSONL/YAML directory records with both byte metrics, counts, newest mtime, signed age_seconds and root ignored state. Expose through Python report dictionaries and public typing. Verify all formats decode consistently, additive full expansion and truncation visibility.
