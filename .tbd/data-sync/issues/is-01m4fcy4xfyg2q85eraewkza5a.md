---
type: is
id: is-01m4fcy4xfyg2q85eraewkza5a
title: Drop the documentation-path count from human grouped rows
kind: task
status: closed
priority: 1
version: 2
labels:
  - output
  - release
dependencies: []
parent_id: is-01m4f6es9he5sghxbh8df7y6sy
created_at: 2026-10-09T03:57:29.888Z
updated_at: 2026-10-09T05:10:01.011Z
closed_at: 2026-10-09T05:10:01.010Z
close_reason: "Merged in #188: human grouped rows drop the documentation count; machine output keeps it."
resolution: null
duplicate_of: null
---
Maintainer decision 2026-10-09 on #187: 'N documentation' (files under doc/docs/documentation components or named README/CHANGELOG/CONTRIBUTING, a path heuristic in classify/file_type_detection.rs) is nearly every file in DOCUMENTS and its label reads as a measure of the documents. Remove it from human grouped rows; machine output keeps classification.documentation and documentation_files. Generated and vendored stay. Requires re-recording the 0.4.0 demo (fdu-5dt6), which shows the line.
