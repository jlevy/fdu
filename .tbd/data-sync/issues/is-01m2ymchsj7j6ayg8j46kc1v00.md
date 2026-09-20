---
type: is
id: is-01m2ymchsj7j6ayg8j46kc1v00
title: Code analysis retains an unbounded logical line despite allocation-bounded wording
kind: bug
status: open
priority: 2
version: 1
labels:
  - content
  - memory
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T05:24:54.704Z
updated_at: 2026-09-20T05:24:54.704Z
---
Root correctness review at 7711150f: content/content_code_metrics.rs CodeAccumulator holds line: Vec<u8>, appends every byte until CR/LF, and clear() retains its peak capacity for the rest of the file. A supported minified or generated source file consisting of one very long line therefore consumes memory proportional to that line, potentially the entire file, per analysis worker. The module claims allocation-bounded classification. The unknown-type prefix fix does not bound this analyzer buffer once a supported code language is selected. Preserve exact results: implement incremental lexical state with bounded lookahead rather than truncating or skipping input, and prove chunk-boundary equivalence for every supported syntax. Until that design is implemented, document the longest-line memory limitation alongside exact Markdown buffering. This is distinct from b2qz whole-Markdown parser/source retention.
