---
type: is
id: is-01m4kjcrqhyg5h35x1gkf9d8bm
title: Land H197, parent-relative directory opens, through a reviewed pull request
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-10T18:49:49.553Z
updated_at: 2026-10-10T18:49:49.553Z
---
exp-219: on claude/h197-relative-opens (d7cbfc71, on 0d6011da), opening each directory relative to its parent's descriptor takes 3.35% [-6.18%, -0.93%] off the default tree and 5.06% off the summary on node-modules-dense (accepted), unresolved on linux-v6.12, -2.34% on linux-balanced-1m. Before review: a walk-level test that the fd budget holds (and falls back by path) under a small RLIMIT_NOFILE, a test of a subtree renamed mid-walk, Windows/macOS builds unaffected (glibc Linux only), the serial walk and reconciliation still by path. Decide whether it ships in 0.5.0 or after.
