---
type: is
id: is-01m2th8zkwx3fjq3wrm2cbztaz
title: "PR #86 Documentation check fails: README.md not in flowmark normal form"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-18T15:13:34.331Z
updated_at: 2026-09-18T15:21:14.990Z
closed_at: 2026-09-18T15:21:14.989Z
close_reason: "Fixed in a473ec32 on cursor/first-user-readme-f28b; Documentation check passes and PR #86 is CLEAN."
resolution: null
duplicate_of: null
---
Commit 749b7977 on cursor/first-user-readme-f28b edited README.md (restored paired performance comparison numbers) without running 'make docs-format'. CI 'Documentation' job fails at 'make docs-format-check' with 'Would reformat: README.md'. Reproduced locally in the pr-86 worktree. Fix is a pure reflow, no content change.
