---
type: is
id: is-01m4swms8xg4h4r2x9trvgptkq
title: Audit fdu help, embedded docs, and agent skill before 0.5.0
kind: task
status: in_progress
priority: 1
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-13T05:44:24.590Z
updated_at: 2026-10-13T06:15:40.914Z
started_at: 2026-10-13T05:44:36.408Z
---

## Notes

PR201 head9119a149: Linux parity artifact11792540797 copied unchanged and old policy assertions corrected. Independent ReviewB confirmed A1/A3 resolved, A2 full local make check still open. CI revealed oversized full-skill regex on all three platforms; minimal matcher boundary fix in progress. User confirmed fdu help/docs/skill scope and requested external build cleanup plus local uv installation upgrade. Four audited fdu-only disposable paths permanently removed (~2.27GiB), two merged clean owned release worktrees removed (~134MiB); external free rose6.6 to8.8GiB before new wheel build. Other-project cleanup was rejected by auto-review and not performed. Current wheel build uses external Cargo/cache scratch. Release gates remain unwaived.
