---
type: is
id: is-01m481qgg3rs9qryf43ar19xax
title: Decide whether logical (document) words should be additive across rows
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T07:26:57.026Z
updated_at: 2026-10-06T07:26:57.026Z
---
Since fdu-ij5n, a documents section's share denominator is the sum of its rows' document words, but the TOTAL row's document_words/pages stay pooled (LogicalWordStats::logical_words picks one of three counting rules - tokens, chars/6, chars/3 - from statistics pooled across all files, crates/fdu-core/src/content/content_model.rs:550), so the rows do not add up to the total (fixture: rows 125+50, pooled total 142). Options: keep pooled totals (same meaning in every grouping) or make logical words additive (apply the clamp per file), which changes every logical-word number and the analyzer's measure. Maintainer decision.
