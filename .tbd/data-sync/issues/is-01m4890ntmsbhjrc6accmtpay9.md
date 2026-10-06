---
type: is
id: is-01m4890ntmsbhjrc6accmtpay9
title: "PR #177 C2: tip"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-06T09:34:17.426Z
updated_at: 2026-10-06T09:34:17.426Z
---
Review C (https://github.com/jlevy/fdu/pull/177#issuecomment-6013447138), Low. tip: show it under a flat format suggests views that format refuses (query_report.rs:1482-1484): --analyze all --view files --sort code_lines --format long prints 'tip: show it: --view files,documents', which long refuses. Make the tip format-aware or omit it under flat formats.
