---
type: is
id: is-01m48p9he4pwjvebw6nr7mzpcb
title: "Retarget #176 to main after #169 and #175 merge, then re-review"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T13:26:19.332Z
updated_at: 2026-10-06T13:26:19.332Z
---
PR #176 (performance index implementation) is informally stacked: it carries #169 (loop history) and #175 (index spec) merged in. Once both land on main, merge main into #176's branch (never rebase), retarget its base to main, run make check plus make perf-test and perf-report-check, and request a fresh review of the diff against main before marking ready. Review E2/G6 on #176.
