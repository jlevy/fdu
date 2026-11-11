---
type: is
id: is-01m76a5kpx8knnn8m15k2n43by
title: "Memory: --view files holds about four row-sized buffers at its peak; activity slot 24 to 32 B"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T22:03:04.285Z
updated_at: 2026-11-11T22:03:04.285Z
---
Review K6 and K8 on #209. K6: FileRow grew 16 B but the --view files peak grew ~68 B/row, so several row buffers are alive at once (per-root every_entry vectors, their concatenation, the stable sort scratch); pre-existing amplification, find the copy with a heap profile. K8: Option<DirectoryActivity> holding Option<Times> has no niche, so query_subtrees activity slots grew 24 to 32 B; store Times with the Times::NONE sentinel.
