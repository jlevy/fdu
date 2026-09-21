---
type: is
id: is-01m32wjeren9zr84kerpgy386y
title: Decide whether time and size predicates without --kind should cover directory contents in aggregate views
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-21T21:04:54.541Z
updated_at: 2026-09-21T21:04:54.541Z
---
Product question recorded while addressing PR #103 R2 (https://github.com/jlevy/fdu/pull/103#issuecomment-5764980291) and PR #96 R3. The plan specifies that a directory matched by --min-size or --modified-since/--modified-before with no --kind covers its eligible contents, so 'fdu . --modified-since 7d --view summary' now counts every file under any directory with activity this week, and the default tree shows such directories at full size. That is documented, stated in the spec under 'Requests Whose Answers Change', and covered by a golden. The alternative the #103 review offered was to change the engine so that a directory matched only by a size or time bound (no --include, no --kind dir) does not cover: the default 'what changed this week' would then answer with the changed files and the directories holding them, as before. Recommendation: revisit before 0.2.0; if coverage is kept, keep the --kind file guidance beside every time-bounded example.
