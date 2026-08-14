---
type: is
id: is-01kzyevbq7jcb3nyvwq15edmh2
title: Land or close the two long-open PRs whose reviews are already resolved
kind: chore
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-13T21:01:03.079Z
updated_at: 2026-08-13T21:01:03.079Z
---
PR #4 (fix(cache): address PR #3 review, open since 2026-08-11, 7 commits not in main) and PR #11 (refactor: clarify Rust module filenames, open since 2026-08-13; scripts/check-rust-module-names.mjs is not on main) both have every review thread resolved with fix replies, and their review beads (fdu-dirt, fdu-cjo1) are closed. Their content is nonetheless unlanded. Decide per PR: merge, or close with the reason recorded so the closed review beads stop implying delivered work.
