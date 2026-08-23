---
type: is
id: is-01m0pe08ypfatxjggkzfmfvyv3
title: "Revalidate PR #38 (exp-064 content tier) against current main"
kind: task
status: closed
priority: 1
version: 3
assignee: claude-code@vm
labels: []
dependencies: []
created_at: 2026-08-23T04:28:01.878Z
updated_at: 2026-08-23T05:18:30.312Z
closed_at: 2026-08-23T05:18:30.312Z
close_reason: "PR #38 taken over and revalidated. exp-064 reproduces; exp-065 records the cross-subject result; the subject record now carries provenance."
---
PR #38 carries two accepted content-tier changes (H94 rollup ancestor walk, H95 indexed
type-rule tiers) measured at -30.31% content-cache-hit and -13.40% content-basic against
main at 703ceac. That base is now 44 commits behind, and the intervening work includes the
fdu-core/fdu crate split that relocates every source file the PR touches, plus PR #43's
edits to the same generated evidence files (ledger, timeline.json, index.html).

The loop's own rule applies: a number measured before a related change is a prediction,
not a result, and the queue is re-screened after anything touching the same cost.

Work:
- merge origin/main into the PR branch (merge, never rebase -- it is not my branch)
- resolve the file relocations and the generated-evidence conflicts
- rebuild both arms and re-run the paired comparison against CURRENT main as control
- decide on the fresh numbers, not the recorded ones
- reconcile fdu-926e's framing: PR #38 corrects it from ~34% to 11.11% inclusive via a
  caller tree, and the campaign-2 plan I merged in #43 still cites the 34% figure
