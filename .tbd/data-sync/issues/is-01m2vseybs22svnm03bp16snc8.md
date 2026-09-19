---
type: is
id: is-01m2vseybs22svnm03bp16snc8
title: Quiet-machine post-0.1.0 performance baseline
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m2vs96dg6kw3ka6k4ghyvf0f
created_at: 2026-09-19T02:55:52.695Z
updated_at: 2026-09-19T04:42:57.832Z
closed_at: 2026-09-19T04:42:57.831Z
close_reason: "exp-105 baseline recorded (uncontrolled; quiet cell failed). exp-106 rejected H107: wall +1.64% [-4.00%, +4.37%] on metabrowser."
resolution: null
duplicate_of: null
---
exp-105 planned. Quiet Darwin/arm64 bare-metal, os_cache warm-steady. Measure the current main/0.1.0 engine (this branch = origin/main at start) on nominated deciding subjects: rustup-toolchains (default-tree, cold-scan-index) and a source-checkout (content-cache-hit if the tree is dense enough). Record regime. This is the control every later experiment on this PR compares against. Revise the dated QA report from installed-PATH timings; update README ballpark only if the new evidence makes 200K files/sec or 4M cached lines/sec dishonest.
