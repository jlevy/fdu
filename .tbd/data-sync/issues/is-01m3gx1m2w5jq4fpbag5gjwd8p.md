---
type: is
id: is-01m3gx1m2w5jq4fpbag5gjwd8p
title: "H153: share per-file metric resolution across unfiltered content views"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01m3gvqwswcvwe38v0pp58sny0
created_at: 2026-09-27T07:42:33.562Z
updated_at: 2026-09-27T08:59:08.172Z
closed_at: 2026-09-27T08:59:08.171Z
close_reason: One-pass shared metric resolution accepted and committed as d0902cfd; exp-159 records the 47.01% wall win and non-inferior RSS/minor faults.
resolution: null
duplicate_of: null
---
Follow H152 and Astra review. For an unfiltered report with multiple Types/Families/Languages/Documents sections, resolve ContentIndex::file and Index::classify once per file and share that resolved input across metric summaries. Keep single-view and filtered paths unchanged. Preserve current path-classification semantics; do not substitute cached detection. Proposed exp-159. Accept only if content-query wall improves at least 3% with paired 95% interval below zero on deciding-scale metabrowser, exact report oracle passes, and RSS is non-inferior. Stop if the change needs a public abstraction or persistent identity.

## Notes

2026-09-27 complete. The first retained-resolution vector cut wall ~45% but increased minor faults 29.58% and was rejected before commit. The kept one-pass form d0902cfd resolves each file once, feeds all metric accumulators, then drops the classification. Exp-159: wall -47.01% [-47.49%, -45.23%], component -59.94%, user CPU -50.75%, RSS -0.17%, minor faults -0.05%; 12/12 valid, exact oracle and unchanged fingerprint.
