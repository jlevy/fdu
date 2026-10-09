---
type: is
id: is-01m4gy069faa60h3xjhsfzkdz0
title: Accept a file as a root
kind: feature
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T18:14:57.069Z
updated_at: 2026-10-09T18:14:57.069Z
---
fdu refuses a PATH that is not a directory, so `fdu *` (typed the way `du -sh *`, `dust *`, and `pdu *` are) stops at the first file in nearly every directory. PR #192 (review A5, https://github.com/jlevy/fdu/pull/192#issuecomment-6085921156) kept the refusal for 0.5.0 and made the command line say what to type instead (`fdu */`). Accepting a file as a root needs these semantics decided first: (1) a one-file tree: what the tree section shows for a root that is a file (a root row with no children, its share of the total, its age from its own mtime, which a directory root never counts); (2) flat lists: whether the file is a row named by its label, with an empty relative path or its own name, and what `root` + `path` resolve to in machine output; (3) summary: one file, zero directories, and whether the root directory count rule changes; (4) ignored state: a file root has no .gitignore of its own and no parent rules are read, so whether it can ever be gitignored; (5) content analysis: the file is analyzed as any file, but the sidecar and snapshot are keyed by root, so whether a file root caches at all; (6) one root: whether `fdu FILE` answers too, which keeps the sum rule, and how --watch, --cache-status, and --cache-clear treat it; (7) overlap: a file inside another root is a repeated path.
