---
type: is
id: is-01m0ethvrzj0kgm3wzyvevnq4p
title: Extensions view silently omits files with no extension
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m0erhq35tpxzjecxn3p9jzx2
created_at: 2026-08-20T05:33:25.662Z
updated_at: 2026-08-20T05:33:25.662Z
---
Found during the end-to-end round on macOS.

`--view extensions` over tests/golden/fixtures/project reports .tar.gz (128 B), .md (71 B), and .rs (36 B) = 235 B, while the tree and summary views report 263 B over the same 6 files. The missing 28 B is Makefile, which has no derived extension and therefore gets no row — even with `--limit all`. The existing golden 'Extensions Preserve the Original Raw Grouping' in tests/golden/cli-axes.tryscript.md pins this behavior, so it is deliberate today rather than a regression.

The concern is that the omission is silent: rows do not sum to the reported total and nothing in the output says why. Every other roll-up view accounts for all bytes. Options: give extension-less files an explicit row (a '(none)' bucket, matching how `types` uses 'unknown:.bin'), or state the exclusion in --help and the README so the shortfall is expected rather than surprising.

Not fixed in the header PR: it changes a pinned view contract and deserves its own decision.
