---
type: is
id: is-01m0nt4swnqr1yr0w38cxhnyyn
title: "PR #40 review R2: library diagnostics name CLI flags to Python callers"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:40:58.772Z
updated_at: 2026-08-22T23:14:22.731Z
closed_at: 2026-08-22T23:14:22.731Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu/src/query/query_report.rs:676 (new here) and :328 (pre-existing). display_notes and validate_analysis hardcode --analyze/--view. Thread a surface label as ViewSpec::resolve and AnalysisSet::parse_labeled already do.
