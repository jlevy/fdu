---
type: is
id: is-01m32jzm7qrrcje9e9tw4x6by7
title: "PR #97 review R6: finish unused send_full allocation"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels: []
dependencies: []
parent_id: is-01m32jzamqf58124kga00kdbd0
created_at: 2026-09-21T18:17:20.375Z
updated_at: 2026-09-21T18:17:20.375Z
---
Low. Comment 5765288334. finish → send_full → next_vec leaves an unused Vec. Fix: in finish, send without next_vec replacement (self.batch = Vec::new()). Note the public-path capacity change as inherited in the H147 record.
