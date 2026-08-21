---
type: is
id: is-01m0hg8hb2dn9d9wjntnpd4xzk
title: Retain word metrics for every text family, not just prose and markup
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:17.601Z
updated_at: 2026-08-21T07:15:53.544Z
closed_at: 2026-08-21T07:15:53.544Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
`LogicalWordStats`, `raw_words` and `paragraphs` are computed for every admitted text
file and then zeroed for anything outside `prose`/`markup` (content_analysis.rs, the
`TextAdmission::Accepted` arm).

Under `words`, retain `raw_words` and `logical_word_stats` wherever they were measured,
including the `code` family: the work is already done, and normalized word volume is the
cheap proxy for context-window sizing that agent consumers ask for.

Keep `paragraphs` gated to `prose`/`markup` -- a paragraph is a prose concept and a
blank-line-separated block of code is not one.

Aggregates stay meaningful because `families` and `types` keep the rows separate.
