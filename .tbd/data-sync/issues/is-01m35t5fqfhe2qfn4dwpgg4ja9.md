---
type: is
id: is-01m35t5fqfhe2qfn4dwpgg4ja9
title: "Integrate PR #109 validator with the Linux performance evidence stack"
kind: task
status: closed
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:20:35.691Z
updated_at: 2026-09-23T16:24:24.457Z
closed_at: 2026-09-23T16:24:24.455Z
close_reason: "Applied on 2026-09-23: #109 merged the performance stack (34d32876) with the verdict.kept migration (exp-141/146/148/149/150/152 neither, exp-154 control), ledger and report regenerated; merged to main in 7e06e5a4; perf-evidence-check passes."
resolution: null
duplicate_of: null
---
Standalone #109 36048330 is review-clear. Combining it with #94/#97/#105 removes CLAIM_ONLY_EXPERIMENTS while seven added records depend on it: exp-141,146,148,149,150,152,154. Migrate deliberately to verdict.kept, preserve timeline tests, regenerate ledger/projection/page and run evidence gates. Resolve timeline.py, test_timeline.py and guide conflicts in an isolated merge; no rebases of cited evidence commits. Sol scratch rehearsal in the alpha audit provides a reviewable patch.

## Notes

Sol completed isolated eight-PR integration at detached c5b759ee; source branches unchanged. Preserved neither for exp-141/146/148/149/150/152, control for exp-154, with exp-103 neither from #109. Timeline generator validated 155 artifacts, 333 performance tests passed, generated HTML/format/diff checks passed. Only projected kept-arm change versus #105 is exp-154 null to control. Prepared integration-combined.patch and integration-109-remerge-diff.txt in task artifacts. Still open until applied to real branches, reviewed, and gated.

2026-09-22 read-only current-stack rehearsal: PR105 245395c0 + standalone PR109 36048330 conflicts in performance-loop.md, realtree/tests/test_timeline.py, and realtree/timeline.py. PR117 can take PR109 cleanly, but then PR105 causes these three plus two engine conflicts. Recommended real order is correctness, PR94, PR97, PR105, then PR109 so kept-arm migration happens once after final performance records. Exact inputs in /tmp/fdu-alpha-stack-PnTm5M/reconciliation-audit.md. Existing scratch validation remains reference evidence; no real merge or publication yet.
