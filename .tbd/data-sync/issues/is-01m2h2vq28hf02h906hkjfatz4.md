---
type: is
id: is-01m2h2vq28hf02h906hkjfatz4
title: "PR #55 review PR55-CUR-1: replay cursor never advances past a denied subtree"
kind: bug
status: in_progress
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h2v87crh401e90pfe52w8g
created_at: 2026-09-14T23:08:29.624Z
updated_at: 2026-09-14T23:09:13.384Z
---
PR #55 at dc27c14: checkpoints plan :124-129, :162-167. A stably denied subtree blocks cursor publication, and a complete-only age-gate sweep publishes nothing. Record denied subtrees as typed gaps and publish a partial checkpoint with a marker.
