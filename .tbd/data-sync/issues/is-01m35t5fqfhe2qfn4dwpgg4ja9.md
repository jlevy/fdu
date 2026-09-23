---
type: is
id: is-01m35t5fqfhe2qfn4dwpgg4ja9
title: "Integrate PR #109 validator with the Linux performance evidence stack"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:20:35.691Z
updated_at: 2026-09-23T00:20:35.691Z
---
Standalone #109 36048330 is review-clear. Combining it with #94/#97/#105 removes CLAIM_ONLY_EXPERIMENTS while seven added records depend on it: exp-141,146,148,149,150,152,154. Migrate deliberately to verdict.kept, preserve timeline tests, regenerate ledger/projection/page and run evidence gates. Resolve timeline.py, test_timeline.py and guide conflicts in an isolated merge; no rebases of cited evidence commits. Sol scratch rehearsal in the alpha audit provides a reviewable patch.
