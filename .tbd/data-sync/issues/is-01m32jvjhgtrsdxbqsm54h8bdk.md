---
type: is
id: is-01m32jvjhgtrsdxbqsm54h8bdk
title: "PR #94 review R6: exp-138/140 not re-paired on the merged engine"
kind: task
status: open
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6bvgxbks1d5t5q5p0k44
created_at: 2026-09-21T18:15:07.567Z
updated_at: 2026-09-21T18:35:06.829Z
---
Senior review of PR #94 (https://github.com/jlevy/fdu/pull/94#issuecomment-5765270106), R6 (Low).

exp-138 (H139 content-cache-hit) and exp-140 (H141 content-query) were measured with verdict.commit a5c98d59. f8a2ed94 (#92 R1-R3: snapshot.rs per-name component==name check, content_cache.rs completeness denominator, query_report.rs) and c441edf6 (query_report.rs single-view Cow borrow) landed afterwards. Only exp-142/143 ran on the 937f9445 probe; no Linux content-cache-hit or content-query pair exists on the merged engine.

Disposition on #94: the doc claim was corrected in place (Linux Standing + spec now state the commit and 'not re-paired after R1-R3/c441edf6; expected below noise'). The re-measurement itself is deferred: it needs a quiet Linux host (load/core <= 0.25) which was not available.

Follow-up when a quiet Linux host is free: one quiet 12-pair content-cache-hit on the main probe against the same #91 control e667b739, subject linux-v6.12. Mint the next free id (exp-144+). If the effect is below the 3% wall rule, fold the result into the Linux Standing and drop the not-re-paired caveat.
