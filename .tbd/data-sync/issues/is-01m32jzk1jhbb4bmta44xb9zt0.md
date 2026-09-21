---
type: is
id: is-01m32jzk1jhbb4bmta44xb9zt0
title: "PR #97 review R2: test recycled-buffer reuse"
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels: []
dependencies: []
parent_id: is-01m32jzamqf58124kga00kdbd0
created_at: 2026-09-21T18:17:19.154Z
updated_at: 2026-09-21T18:17:19.154Z
---
High. Comment 5765288334. crates/fdu-core/src/scan.rs next_vec clear() at ~2619-2628 and recycle at ~1306-1310. Deleting recycled.clear() keeps the existing suite green; previously sent ops are folded again. Add one scan::tests test: threads Some(4), batch_size 3, read_controls false, 16 dirs × 40 files with distinct sizes, fold through scan_summary_fold, assert files/bytes/dirs and ops == report.entries. Do not weaken the recycle path.
