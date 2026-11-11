---
type: is
id: is-01m74qj39sdcjg5txdmp7e2jye
title: "PR #209 G1: a capacity call on a route that did not just walk re-mounts an expired automount"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m74qj25f6rvk77ybzaz8rszg
created_at: 2026-11-11T07:18:36.088Z
updated_at: 2026-11-11T08:42:31.002Z
closed_at: 2026-11-11T08:42:31.001Z
close_reason: "Fixed in 71b026b2 (with 05029aac): capacity asked only where the non-mounting stat still finds the counted filesystem; root-only expiring-automount test and seams."
resolution: null
duplicate_of: null
---
High. query_volumes.rs walk_listed/observe_entries, capacity_statfs.rs:88; routes query_report.rs:2099, watch_session.rs:535, opened/read.rs:253-283, fdu-py lib.rs:556; principle fdu-design-principles.md:387-393. Fix: re-check each non-root boundary with the non-mounting stat before asking, in path order, skipping everything below a failed one. Review G on PR #209: https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811
