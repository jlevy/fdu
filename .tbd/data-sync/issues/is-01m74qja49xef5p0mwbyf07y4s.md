---
type: is
id: is-01m74qja49xef5p0mwbyf07y4s
title: "PR #209 G7: under --ignored=only selected and scanned differ with no note"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m74qj25f6rvk77ybzaz8rszg
created_at: 2026-11-11T07:18:43.081Z
updated_at: 2026-11-11T08:42:37.146Z
closed_at: 2026-11-11T08:42:37.146Z
close_reason: "Fixed in 64e0b713: scanned counts the scan's own population; golden and parity updated."
resolution: null
duplicate_of: null
---
Low. docs/machine-output.md:19-23, query_report.rs:2391-2400. Preferred fix: count scanned over the population's own entries. Review G on PR #209: https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811
