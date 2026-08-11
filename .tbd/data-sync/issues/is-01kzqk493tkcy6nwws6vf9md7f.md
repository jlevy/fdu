---
type: is
id: is-01kzqk493tkcy6nwws6vf9md7f
title: "PR #3 review R4: publish a true-base cumulative performance comparison"
kind: task
status: closed
priority: 1
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:08.601Z
updated_at: 2026-08-11T07:07:28.481Z
closed_at: 2026-08-11T07:07:28.480Z
close_reason: Published exp-012 from the literal PR base diagnostic, correctness-normalized base control, and frozen candidate in one v2 interleaved run; replaced the unsupported headline and recorded the failed CPU guardrail.
---
FDU-PR3-R4. exp-006, the experiment ledger, and PR claims use b565882 while calling it the pre-work base; the actual PR base is fdd9e523. After correctness/oracle/statistics fixes, run a quiet interleaved fdd9e523-vs-final comparison, store exact revisions structurally, and narrow or replace every unsupported headline. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
