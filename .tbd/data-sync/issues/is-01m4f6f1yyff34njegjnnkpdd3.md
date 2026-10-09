---
type: is
id: is-01m4f6f1yyff34njegjnnkpdd3
title: "Stack grouped rows: files on the label line, each further measure on its own line below"
kind: task
status: open
priority: 1
version: 1
labels:
  - output
  - release
dependencies: []
parent_id: is-01m4f6es9he5sghxbh8df7y6sy
created_at: 2026-10-09T02:04:23.901Z
updated_at: 2026-10-09T02:04:23.901Z
---
Maintainer decision 2026-10-08, for 0.4.0: in render_text_metrics (crates/fdu-core/src/report_format.rs), every grouped row (DOCUMENTS, languages, types, families) keeps size, share, label and file count on its first line; when it carries more, each further measure -- lines with their breakdown, words with pages, generated, vendored, documentation, and each non-analyzed coverage reason -- goes on its own continuation line, indented under the file count. Rows with only a file count stay one line. Goal: the 0.4.0 demo fits 140 columns (today's widest DOCUMENTS row is 154). Human output only; machine formats unchanged. Blocks the 0.4.0 demo re-record (fdu-5dt6).
