---
type: is
id: is-01m48ym52af5g9ydrpr285mh70
title: "PR #174 B2: percentages note repeats a denominator per section (report_epilogue.rs:236-246)"
kind: bug
status: in_progress
priority: 3
version: 2
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m48ykwg1fz7tsw30h6xfhtmd
hold: null
hold_until: null
created_at: 2026-10-06T15:51:55.721Z
updated_at: 2026-10-06T15:52:00.831Z
started_at: 2026-10-06T15:52:00.829Z
---
Low. crates/fdu-core/src/report_format/report_epilogue.rs:236-246 prints 'code lines (CODE), code lines (LANGUAGES)' (golden cli-content.tryscript.md:778). Suggested fix: group sections by label in first-seen order: 'code lines (CODE, LANGUAGES), document words (DOCUMENTS)'; extend text_labels_a_percentage_that_is_not_a_byte_share; regenerate golden. PR #174, review B: https://github.com/jlevy/fdu/pull/174#issuecomment-6020029670
