---
type: is
id: is-01m2xt60nz02xa4wnkbxspfg75
title: "H131: restore DFS joins parent path instead of path_of"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T21:46:57.597Z
updated_at: 2026-09-19T21:54:06.324Z
closed_at: 2026-09-19T21:54:06.323Z
close_reason: Accepted exp-130. Restore DFS parent-path join kept at 7840ce9b. Wall -4.07% [-4.54%, -3.28%] on frozen metabrowser-clone. Quiet this tick 27.23%.
resolution: null
duplicate_of: null
---
H131 / exp-130. Cache-only restore walks every file and reconstructs its relative path with path_of (ancestor walk + PathBuf collect). exp-129 sampled that at 11.85% of content_open. The DFS already holds the parent; joining the child name keeps path identity and completeness.

Not H116 (HashMap stays). Not H129 (classify already gone). Not a public path_of change. Completeness stays H125 restore-count. Digest must stay 3be19a3e.

Accept: content-cache-hit wall on frozen metabrowser-clone down at least 3% with the interval below zero. Quiet once; if it fails, skip and label uncontrolled. Revert on reject.
