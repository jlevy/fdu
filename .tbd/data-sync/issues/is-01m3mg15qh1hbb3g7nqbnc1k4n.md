---
type: is
id: is-01m3mg15qh1hbb3g7nqbnc1k4n
title: "QA: run_installed_cli_qa.py JSON analysis check never runs (reads analysis.total, absent in fdu.report/10)"
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T17:12:05.104Z
updated_at: 2026-09-28T22:51:03.815Z
---
check_json_analysis in scripts/run_installed_cli_qa.py (~643-656) reads analysis.total, which fdu.report/10 does not have, so the check always passes vacuously. Fix it to read the current shape and fail when the field is missing.

## Notes

2026-09-28: implemented on claude/fdu-alternatives-research-qx0xn0 (PR #155), merged at 2d1eea9d/e0923063. 1ffed6a8: reads reports[0].metrics.total.metrics.physical_lines and fails when missing. Pending: independent review and CI, then close.
