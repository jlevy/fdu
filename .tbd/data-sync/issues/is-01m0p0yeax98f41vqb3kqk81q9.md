---
type: is
id: is-01m0p0yeax98f41vqb3kqk81q9
title: "PR #42 R22: the Python binding keeps its own copy of the view-label arms"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:39:50.365Z
updated_at: 2026-08-23T00:57:54.169Z
closed_at: 2026-08-23T00:57:54.169Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
crates/fdu-py/src/lib.rs:842 hand-writes all ten ViewSpec label arms instead of calling ViewSpec::label. This is the third copy and the exact duplication PR #40 collapsed (report_format::view_label was one of the seven), and the drift class that produced fdu-ggux. The values happen to agree today; nothing makes them. Found while removing the report_format wrapper (S1).
