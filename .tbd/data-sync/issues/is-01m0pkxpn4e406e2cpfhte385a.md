---
type: is
id: is-01m0pkxpn4e406e2cpfhte385a
title: "Address review: PR #38 — the qualifier did not reach the generated views"
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-23T06:11:29.060Z
updated_at: 2026-08-23T06:13:41.334Z
closed_at: 2026-08-23T06:13:41.334Z
close_reason: "Fixed in this PR: exp-064's verdict reason qualified and the three generated views regenerated, H96-H99 folded back into the registry table, the --tree-reconstructible help corrected to the shape contract, and five tests over the four new rendering branches (mutation-verified). fdu-926e retitled."
---
Review of PR #38 found four defects of one shape: a correction the PR argues for landed in hand-written prose but not in the field that is rendered or read.

1. The H96-H99 registry rows in performance-loop.md were appended after the table's blank-line terminator, so they sat outside the table with no delimiter row, and flowmark then prose-wrapped H96 across eight lines. The four hypotheses this PR renumbered to fix an id collision did not render as table rows at all.
2. exp-064's frontmatter verdict.reason -- the string the ledger, timeline.json and index.html all render -- still quoted -13.40% on content-basic unqualified. Every hand-written location was qualified; the three generated views, which the PR itself argues are what people quote from, were not.
3. record.py's --tree-reconstructible help said following provenance yields a tree with the run's engine digest, contradicting the schema, the model docstring and the PR's central argument, all of which say same shape, not same digest. No regeneration reproduces the digest, so a recorder following the help could never pass the flag, and exp-064's recorded true would be indefensible.
4. The four new summary.py rendering branches (reconstructible, not-reconstructible, unrecorded, sparse-ratio) shipped untested, and only the unrecorded branch is exercised by the committed corpus.

Also corrected fdu-926e's title, which still asserted the ~34% figure its own re-scope note refutes.
