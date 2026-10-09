---
type: is
id: is-01m4fg4mkdprja7axmjy54bzmf
title: Decide the per-file 'documentation' tag in human file listings
kind: task
status: open
priority: 3
version: 1
labels:
  - output
dependencies: []
parent_id: is-01m4f6es9he5sghxbh8df7y6sy
created_at: 2026-10-09T04:53:28.300Z
updated_at: 2026-10-09T04:53:28.300Z
---
Review A A1 on #188 (https://github.com/jlevy/fdu/pull/188#issuecomment-6074507111): render_text_metric_files (report_format.rs ~2062) still prints a per-file classification tag, e.g. '3  README.md (markdown, extension, documentation)'. The maintainer dropped the documentation COUNT from grouped rows (fdu-cuhw); a per-file tag is true of that file, but carries the same path heuristic. Decide: keep and document + golden it, or drop it too.
