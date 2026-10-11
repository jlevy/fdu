---
type: is
id: is-01m4kjcrqhyg5h35x1gkf9d8bm
title: Land H197, parent-relative directory opens, through a reviewed pull request
kind: task
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-10T18:49:49.553Z
updated_at: 2026-10-11T05:29:29.534Z
started_at: 2026-10-11T05:29:24.815Z
---
exp-219: on claude/h197-relative-opens (d7cbfc71, on 0d6011da), opening each directory relative to its parent's descriptor takes 3.35% [-6.18%, -0.93%] off the default tree and 5.06% off the summary on node-modules-dense (accepted), unresolved on linux-v6.12, -2.34% on linux-balanced-1m. Before review: a walk-level test that the fd budget holds (and falls back by path) under a small RLIMIT_NOFILE, a test of a subtree renamed mid-walk, Windows/macOS builds unaffected (glibc Linux only), the serial walk and reconciliation still by path. Decide whether it ships in 0.5.0 or after.

## Notes

Review A5 on PR #194: exp-219 accepts H197 on the accept rule's arithmetic only; the rule's fourth criterion, that the complexity is worth it, is owed to this bead's review: 403 lines, three new unsafe blocks, descriptors held for queued subdirectories (at most a quarter of the soft RLIMIT_NOFILE, at most 1,024), and the rename case. On a tree renamed mid-walk a relative open lists the child of the directory whose listing named it, where the absolute path may name another directory or none (d7cbfc71's read_at comment), so the registration's 'never decides an answer' holds only for a tree not renamed mid-walk. Write the judgment down in the landing PR and in exp-219's follow-up.
