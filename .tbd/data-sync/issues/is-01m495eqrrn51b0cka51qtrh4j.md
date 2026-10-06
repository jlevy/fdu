---
type: is
id: is-01m495eqrrn51b0cka51qtrh4j
title: Empty DOCUMENTS section prints a header with nothing under it
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T17:51:18.295Z
updated_at: 2026-10-06T17:51:18.295Z
---
On main 55d66863, a report whose DOCUMENTS section has no rows still prints the section header. Either omit the section or print an explicit empty-state line, consistent with docs/project/architecture/fdu-output-design.md. Found by the README senior review (2026-10-06).
