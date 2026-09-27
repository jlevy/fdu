---
type: is
id: is-01m3h1e7cm71fn9mpae7v72s6x
title: "H155: profile post-H153 content-query materialization"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-09-27T08:59:20.851Z
updated_at: 2026-09-27T08:59:20.851Z
---
After H153, isolate the timed content-query report region from scan/content setup and profile the current engine. Direct metric reduction without materializing shared FileRows is the next bounded algorithmic candidate only if the profile names that materialization at at least 3% of wall. Preserve filtered and single-view behavior, exact report oracle, and resource gates. Do not compile the cut from the setup-skewed exp-158 sample.
