---
type: is
id: is-01m2yh8kc79nw7bn6k6xw8g3bp
title: Address remaining 0.1 correctness blockers after performance review
kind: task
status: in_progress
priority: 1
version: 15
delegate: codex@spud10
labels: []
dependencies: []
child_order_hints:
  - is-01m2yhjb6r67n23jera0t03ebv
  - is-01m2yhjbkazg1zds9x9janfd5m
  - is-01m2yhjbz0hfj930q39b01rmap
  - is-01m2yhjcawjwehkfdxrejrhe64
  - is-01m2yhtwjg6j0vjyjf1v9sehd5
  - is-01m2ymchsj7j6ayg8j46kc1v00
  - is-01m2ymmc4c03yvnc10myb2jmk6
  - is-01m2yrppkm6hp1gbsytzmcmnme
  - is-01m2ysreqwf3xczbbevbv4ve06
  - is-01m2ysrer2fjkw6qa1z57epf46
hold: null
hold_until: null
created_at: 2026-09-20T04:30:19.526Z
updated_at: 2026-09-20T06:58:47.680Z
started_at: 2026-09-20T04:31:31.377Z
---
Audit remaining release correctness against the current #92 stack, reconcile stale or already-fixed beads, implement confirmed defects in coherent slices with subagents, review changes, validate and publish PRs. Preserve unrelated directory-rollup work and existing core-model ownership. Final candidate verification remains distinct from publishing.

## Notes

Reviewed sub-slices are now draft PR #98 (Windows validity, fe06b11b; native CI exposed avoidable allocation regressions under repair) and #99 (ignore matcher cfe38b8e plus explicit permission/native-watch preconditions a86dbfb3). Both are based on exact PR92 head 937f9445. Report/7/Python and provenance/watch integration continue in separate owned worktrees. Root independently compared iterative deep JSON decoding against standard JSON across 2012 cases. Known memory limitations remain tracked under b2qz (Markdown) and 1zb6 (long code lines). No merge or release authorization.
