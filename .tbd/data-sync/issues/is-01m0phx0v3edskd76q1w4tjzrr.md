---
type: is
id: is-01m0phx0v3edskd76q1w4tjzrr
title: Add a dense mode to gen_tree.py and default content jobs to it
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-experiment-evidence-scope.md
labels: []
dependencies: []
created_at: 2026-08-23T05:36:09.571Z
updated_at: 2026-08-23T05:36:09.571Z
---
gen_tree.py writes holes via os.truncate above 256 bytes -- right for metadata-tier work, actively misleading for content-tier work, since reading a hole costs nothing and inflates any per-file bookkeeping win. Add a mode that writes real bytes; keep the 15,977-file sparse subject available for continuity with exp-064.
