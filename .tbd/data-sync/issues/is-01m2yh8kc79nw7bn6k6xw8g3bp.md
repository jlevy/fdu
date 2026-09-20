---
type: is
id: is-01m2yh8kc79nw7bn6k6xw8g3bp
title: Address remaining 0.1 correctness blockers after performance review
kind: task
status: in_progress
priority: 1
version: 17
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
  - is-01m2yt3kzcakn6nka88x1z48ja
hold: null
hold_until: null
created_at: 2026-09-20T04:30:19.526Z
updated_at: 2026-09-20T07:05:32.933Z
started_at: 2026-09-20T04:31:31.377Z
---
Audit remaining release correctness against the current #92 stack, reconcile stale or already-fixed beads, implement confirmed defects in coherent slices with subagents, review changes, validate and publish PRs. Preserve unrelated directory-rollup work and existing core-model ownership. Final candidate verification remains distinct from publishing.

## Notes

Reviewed sub-slices are now draft PR #98 (Windows validity, fe06b11b; native CI exposed avoidable allocation regressions under repair) and #99 (ignore matcher cfe38b8e plus explicit permission/native-watch preconditions a86dbfb3). Both are based on exact PR92 head 937f9445. Report/7/Python and provenance/watch integration continue in separate owned worktrees. Root independently compared iterative deep JSON decoding against standard JSON across 2012 cases. Known memory limitations remain tracked under b2qz (Markdown) and 1zb6 (long code lines). No merge or release authorization.

Integration now includes report7/stream2 serializers and typed Python models, per-analyzer outcomes with encoding coverage, stale/error/provenance fixes, controls-off snapshot projection across routes, Windows ChangeTime validity, and Git pattern corrections. Root CLI golden validation passed165cases before the final projection merge. Original PR91 and PR92 exact-head full make check both passed. PR99 all19CI checks green. PR98 all3 full platform matrices passed at96f16922; final Windows performance harness found an independent-oracle digest mismatch under investigation. Root review of state follow-up found concurrent root passes can undercount newer omitted errors and falsely recover Complete, plus fatal multipath abort can clear unvisited issues; state agent assigned fixes and regressions. Integrated full gate and final Astra route review pending. b2qz/1zb6 remain documented analyzer memory limits; no release/merge performed.
