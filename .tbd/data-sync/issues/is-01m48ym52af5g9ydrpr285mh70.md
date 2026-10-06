---
type: is
id: is-01m48ym52af5g9ydrpr285mh70
title: "PR #174 B2: percentages note repeats a denominator per section (report_epilogue.rs:236-246)"
kind: bug
status: closed
priority: 3
version: 4
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m48ykwg1fz7tsw30h6xfhtmd
hold: null
hold_until: null
created_at: 2026-10-06T15:51:55.721Z
updated_at: 2026-10-06T16:32:43.856Z
started_at: 2026-10-06T15:52:00.829Z
closed_at: 2026-10-06T16:32:43.855Z
close_reason: "fixed in 53d5bfab + 256c00a7 (local fix-174-B): denominators grouped by label in first-seen order; unit test fails before, passes after; golden cli-content.tryscript.md:818 updated"
resolution: null
duplicate_of: null
---
Low. crates/fdu-core/src/report_format/report_epilogue.rs:236-246 prints 'code lines (CODE), code lines (LANGUAGES)' (golden cli-content.tryscript.md:778). Suggested fix: group sections by label in first-seen order: 'code lines (CODE, LANGUAGES), document words (DOCUMENTS)'; extend text_labels_a_percentage_that_is_not_a_byte_share; regenerate golden. PR #174, review B: https://github.com/jlevy/fdu/pull/174#issuecomment-6020029670

## Notes

Fixed in 53d5bfab (local fix-174-B on origin/main 55d66863): grouped by label in first-seen order; unit test case added (fails before, passes after); golden cli-content.tryscript.md:818 edited by hand.
