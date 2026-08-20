---
type: is
id: is-01m0gf6pv7jy26jkfvypqpyqy4
title: Twenty experiment artifacts are missing the standard doc footer
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-20T20:53:34.694Z
updated_at: 2026-08-20T20:53:34.694Z
---
44 of the 64 files in docs/project/experiments/ carry the common-doc-guidelines footer and 20 do not, so the convention is currently half-applied and a reader cannot tell whether its absence means anything.

Noticed while reviewing PR #27, which had added the footer to some of them as a side effect of other edits. Left out of PR #36 deliberately: it is 20 mechanical file edits with no relationship to that PR's subject, and burying them there would make its diff unreviewable.

AGENTS.md says to apply the guidelines to every human-authored document and retain the footer. An experiment artifact is human-authored in its Markdown body even though its frontmatter is generated, so the footer belongs on all 64.
