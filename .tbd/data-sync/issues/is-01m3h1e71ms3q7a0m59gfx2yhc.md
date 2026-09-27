---
type: is
id: is-01m3h1e71ms3q7a0m59gfx2yhc
title: "H154: replicate H153 shared metric resolution on Linux"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - campaign-2
  - linux
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-09-27T08:59:20.500Z
updated_at: 2026-09-27T08:59:20.500Z
---
Replicate the landed d0902cfd H153 one-pass shared metric resolution on a reconstructible deciding Linux subject, preferably linux-v6.12. Proposed exp-160. Compare 1ba06b19-equivalent independent metric resolution with d0902cfd using content-query, 12 interleaved pairs, exact report oracle, quiet regime if available. Accept the Linux magnitude only if wall improves at least 3% with the paired interval below zero and RSS/faults are non-inferior. This is replication, not a new engine change.
