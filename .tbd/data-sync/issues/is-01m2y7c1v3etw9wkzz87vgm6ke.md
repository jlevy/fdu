---
type: is
id: is-01m2y7c1v3etw9wkzz87vgm6ke
title: Render directory size and actual age across text, machine formats and Python
kind: feature
status: open
priority: 1
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01m2y7c6yzccahx1ntvy2wf1s1
  - type: blocks
    target: is-01m2y7cbprn6w292gjenv0twd7
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-20T01:37:26.880Z
updated_at: 2026-09-20T01:38:12.298Z
---
Add text size/age/path and JSON/JSONL/YAML directory records with both byte metrics, counts, newest mtime, signed age_seconds and root ignored state. Expose through Python report dictionaries and public typing. Verify all formats decode consistently, additive full expansion and truncation visibility.
