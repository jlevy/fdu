---
type: is
id: is-01m2mcrsay065yan47363rsqrk
title: "PR #67 delta D67-6: an empty YAML cache listing renders caches as null where JSON renders an empty array"
kind: bug
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:59:22.717Z
updated_at: 2026-09-16T05:59:25.805Z
---
Delta review 5218953084 of PR #67, P3 (optional), pre-existing from `816fcf7`.

`crates/fdu-core/src/report_format.rs:1473@a5e2124`.

`format!("schema: {}\ncaches:", ...)` with no statuses emits `caches:` followed by nothing,
which YAML reads as null; JSON emits `[]`. Worth fixing inside `fdu.cache/1` while the
version is still 1, rather than after a release pins the shape.
