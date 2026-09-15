---
type: is
id: is-01m2h78jn2w89pwc3pg5cb6g87
title: "PR #57 review PR57-8W5K-2: report projection allocates one String per walked entry for portable identity"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2h77qemaay1jkhbhfzh35me
created_at: 2026-09-15T00:25:25.400Z
updated_at: 2026-09-15T00:28:49.956Z
closed_at: 2026-09-15T00:28:49.954Z
close_reason: "Rebutted, no change: the review reports the cost as requested and marks it not a defect. The walk is bounded by max_work and runs only for a filtered report projection; presizing portable_path would be an unmeasured change on the page-read path, which AGENTS.md's performance loop keeps behind a recorded experiment."
resolution: null
duplicate_of: null
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5200240760, P3, informational (asked for; not a defect). At 7b804df, crates/fdu-core/src/query/query_report.rs:869-870 and opened/read.rs:967-979: portable_path starts from String::new(), so a filtered report projection walk adds one String per walked entry, possibly reallocating; String::with_capacity was suggested.
