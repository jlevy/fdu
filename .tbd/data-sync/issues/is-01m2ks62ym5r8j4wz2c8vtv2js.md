---
type: is
id: is-01m2ks62ym5r8j4wz2c8vtv2js
title: "PR #65 review F5: goldens record zero ignored shares with the ALLOCATED pattern instead of exact 0"
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m2ks5fy0rg2stv9nf1vc7wzb
created_at: 2026-09-16T00:17:07.013Z
updated_at: 2026-09-16T00:17:07.013Z
---
PR #65. tests/golden/cli-axes.tryscript.md, cli-cache.tryscript.md, cli-json.tryscript.md. A zero-file share records allocated as [ALLOCATED] (\\d+), so a non-zero allocated on a zero-file share would pass. 0 is portable and exact. Also IgnoredTally::is_empty (query_report.rs:556) is pub and unused in-crate: use it or drop it.
