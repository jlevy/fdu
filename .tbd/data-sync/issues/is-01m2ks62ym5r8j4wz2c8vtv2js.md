---
type: is
id: is-01m2ks62ym5r8j4wz2c8vtv2js
title: "PR #65 review F5: goldens record zero ignored shares with the ALLOCATED pattern instead of exact 0"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2ks5fy0rg2stv9nf1vc7wzb
created_at: 2026-09-16T00:17:07.013Z
updated_at: 2026-09-16T05:51:38.540Z
closed_at: 2026-09-16T05:51:38.539Z
close_reason: "03b983c (PR #65): a share with no ignored file or directory records allocated 0 in cli-axes and cli-json (14 rows) rather than the [ALLOCATED] digit pattern a wrong non-zero would satisfy; a share of one ignored directory keeps the pattern, because that directory's own allocated size is real and platform-dependent. IgnoredTally::is_empty dropped rather than used: the text suffix asks files > 0, a different question."
resolution: null
duplicate_of: null
---
PR #65. tests/golden/cli-axes.tryscript.md, cli-cache.tryscript.md, cli-json.tryscript.md. A zero-file share records allocated as [ALLOCATED] (\\d+), so a non-zero allocated on a zero-file share would pass. 0 is portable and exact. Also IgnoredTally::is_empty (query_report.rs:556) is pub and unused in-crate: use it or drop it.
