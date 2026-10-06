---
type: is
id: is-01m4890ntmsbhjrc6accmtpay9
title: "PR #177 C2: tip"
kind: task
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T09:34:17.426Z
updated_at: 2026-10-06T15:37:10.141Z
---
Review C (https://github.com/jlevy/fdu/pull/177#issuecomment-6013447138), Low. tip: show it under a flat format suggests views that format refuses (query_report.rs:1482-1484): --analyze all --view files --sort code_lines --format long prints 'tip: show it: --view files,documents', which long refuses. Make the tip format-aware or omit it under flat formats.
