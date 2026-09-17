---
type: is
id: is-01m2rss53ech1v9fvh9hdd4cgg
title: Review stacked core-models PRs (#78-#83) for merge readiness
kind: task
status: closed
priority: 1
version: 5
labels: []
dependencies: []
child_order_hints:
  - is-01m2rt8s04hfaybe9y5y5mqr69
  - is-01m2rt8sa4ha4zxd3s7tf3p8xh
created_at: 2026-09-17T23:03:43.982Z
updated_at: 2026-09-17T23:26:36.437Z
closed_at: 2026-09-17T23:26:36.437Z
close_reason: "Stack review complete at 7390b62b. #83 onto #82 is ready to merge: CI fully green (including Windows Test and Windows path-independence full matrix). Whole stack onto main is ready as a 0.1.0 unit with registered leftover PI classes (projection-route, unverified-subtree, windows-change-time). No merge blockers. Follow-up: fdu-c89g (stale engine Known Gaps, P3 docs). Local make check rust-test/lib-only hit a pre-existing flaky parallel_equivalence assertion; CI ubuntu Test passed. Screening perf vs origin/main on /usr/lib (virtualized, uncontrolled) showed no confirmed >3% wall regression."
---
Full-stack review of explicit core models: architecture at structural level, code review of PRs 78/79/81/82/83, make check, and paired performance comparison vs main. Source: https://github.com/jlevy/fdu/pull/83
