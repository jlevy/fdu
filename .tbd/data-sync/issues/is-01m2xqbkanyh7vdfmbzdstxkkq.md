---
type: is
id: is-01m2xqbkanyh7vdfmbzdstxkkq
title: "H127: first-pass vs opened-discovery I/O leftover"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: cursor-agent
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T20:57:34.804Z
updated_at: 2026-09-19T21:08:07.748Z
closed_at: 2026-09-19T21:07:22.726Z
close_reason: "H127 confirmed (exp-126): opened-discovery 8.8x first-pass component. read_dir+fstatat vs getattrlistbulk; 11524 journal clones; 1.12M live merges. No smallest cut."
resolution: null
duplicate_of: null
---
Pre-registered 2026-09-19.

H127 / exp-126. First-pass metadata walk I/O versus opened-root discovery I/O
on frozen deciding-scale metabrowser-clone.

Opened roots run no analyzers. This is not a content-on-opened-root cell and
not a completeness skip.

Determination: pair cold-scan-index with opened-discovery on the same frozen
clone. Name whether opened-discovery leftover is still journal/control (H110)
versus the first-pass walk leftover (H122), and whether any userspace stage
is at least 3% and not already rejected.

Same H125 probe both arms. Uncontrolled allowed if quiet fails.
Do not lower the 25% bar. Do not invent a skip.
