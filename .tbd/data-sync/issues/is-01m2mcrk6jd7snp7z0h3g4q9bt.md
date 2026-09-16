---
type: is
id: is-01m2mcrk6jd7snp7z0h3g4q9bt
title: "PR #67 delta D67-5: cache status promises a clear reclaims a staging file it will decline"
kind: bug
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:59:16.433Z
updated_at: 2026-09-16T05:59:19.672Z
---
Delta review 5218953084 of PR #67, P3 (optional).

`crates/fdu-core/src/report_format.rs:1618-1630@a5e2124`.

`render_cache_status_text` counts every `Leftover` in "fdu --cache-clear=all reclaims
them", including a staging temporary too young to be reclaimed. The delta's own golden
shows it: status promises 3, the clear takes 2 and says why.

Fix: make the promise match what a clear would take, or word it so a young staging file is
visibly excluded.
