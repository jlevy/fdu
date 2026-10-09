---
type: is
id: is-01m4fehyd3bg51vvpnvdmh42je
title: "Singular units in human rows: '1 line', '1 word'"
kind: bug
status: open
priority: 3
version: 1
labels:
  - output
dependencies: []
parent_id: is-01m4f6es9he5sghxbh8df7y6sy
created_at: 2026-10-09T04:25:47.170Z
updated_at: 2026-10-09T04:25:47.170Z
---
Review A A7 on #187, predates it: render_text_metrics prints '1 lines' and '1 words' (report_format.rs ~1592-1607) while files already pluralize ('1 file'). Use plural() for lines and words; goldens with single counts change.
