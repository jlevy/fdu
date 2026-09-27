---
type: is
id: is-01m3gx1m2w5jq4fpbag5gjwd8p
title: "H153: share per-file metric resolution across unfiltered content views"
kind: task
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01m3gvqwswcvwe38v0pp58sny0
created_at: 2026-09-27T07:42:33.562Z
updated_at: 2026-09-27T07:42:38.122Z
---
Follow H152 and Astra review. For an unfiltered report with multiple Types/Families/Languages/Documents sections, resolve ContentIndex::file and Index::classify once per file and share that resolved input across metric summaries. Keep single-view and filtered paths unchanged. Preserve current path-classification semantics; do not substitute cached detection. Proposed exp-159. Accept only if content-query wall improves at least 3% with paired 95% interval below zero on deciding-scale metabrowser, exact report oracle passes, and RSS is non-inferior. Stop if the change needs a public abstraction or persistent identity.

## Notes

2026-09-27: started from clean commit 1ba06b19 after H152 exact-oracle work. Current profiling-build component is 31.125 s / 100 reports on live metabrowser (127,104 files). Whole-process 8 s sample is setup-skewed (read/open 80.19%), so it is attribution only and not a query-path percentage. Code inspection confirms four repeated ContentIndex::file + Index::classify resolutions per shared FileRow. Implement only the bounded unfiltered multi-metric path, then paired release A/B.
