---
type: is
id: is-01m48bsg7p862abytve8fz42ks
title: "PR #179 A5: test the short-token direction of the document-share fix"
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T10:22:48.043Z
updated_at: 2026-10-06T15:37:29.457Z
---
Review A on #179 (https://github.com/jlevy/fdu/pull/179#issuecomment-6014211671), Low suggestion: add a fixture where the pooled document-word total exceeds the rows' sum (e.g. 100 one-character tokens + ten 30-character tokens: pooled 110 vs rows 83; fdu 0.3.0 showed 45.5% + 30.0% = 75.5%) to document_word_shares_sum_to_their_denominator_when_pooling_is_not_additive or a golden, so both directions are covered.
