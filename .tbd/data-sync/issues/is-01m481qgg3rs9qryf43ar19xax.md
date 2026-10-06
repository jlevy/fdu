---
type: is
id: is-01m481qgg3rs9qryf43ar19xax
title: Make documents TOTAL word and page counts match the sum of their rows
kind: task
status: open
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T07:26:57.026Z
updated_at: 2026-10-06T15:37:42.183Z
---
Since fdu-ij5n, a documents section's share denominator is the sum of its rows' document words, but the TOTAL row's document_words/pages stay pooled (LogicalWordStats::logical_words picks one of three counting rules - tokens, chars/6, chars/3 - from statistics pooled across all files, crates/fdu-core/src/content/content_model.rs:550), so the rows do not add up to the total (fixture: rows 125+50, pooled total 142). Options: keep pooled totals (same meaning in every grouping) or make logical words additive (apply the clamp per file), which changes every logical-word number and the analyzer's measure. Maintainer decision.

## Notes

Maintainer decision 2026-10-06: ship 0.4.0 with pooled TOTAL document words (documented exception in fdu-design-principles 'A Roll-Up Partitions What It Reports On'); revisit after 0.4.0.
