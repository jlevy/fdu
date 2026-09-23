---
type: is
id: is-01m35t5b640thkxx0mar0zyvjg
title: "PR #96 A96-2: qualify cached directory completeness by scan scope"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:20:31.041Z
updated_at: 2026-09-23T00:20:31.041Z
---
Plan line 273 and inherited machine-output docs say cache-only rows are complete. Executed scan-depth-bounded cached row remains complete=false and age=null, correctly. Say cache-only delivery adds staleness while completeness still reflects stored scan-depth boundaries. Nonblocking documentation correction.
