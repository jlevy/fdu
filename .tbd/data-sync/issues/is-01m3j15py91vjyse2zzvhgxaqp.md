---
type: is
id: is-01m3j15py91vjyse2zzvhgxaqp
title: "Ignore-aware transient summary: fold the ignored share without retaining an index"
kind: task
status: in_progress
priority: 1
version: 10
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
delegate: claude-code
labels:
  - performance
dependencies:
  - type: blocks
    target: is-01m3kkrj5f6n9b38g6d1w6mrew
parent_id: is-01m3mcwynm1rkdjencnq5621mq
hold: null
hold_until: null
created_at: 2026-09-27T18:13:56.296Z
updated_at: 2026-09-28T20:54:41.961Z
started_at: 2026-09-28T14:30:07.463Z
---
Default fdu --view summary reads .gitignore, and the summary reducer keeps no control table, so the planner falls closed to a full retained index plus a snapshot write. On the 1M balanced Linux tree (no .gitignore files at all) that costs 1.52 s against 0.92 s for --no-gitignore (screen), and 319 MiB against 10 MiB. The ignored share is a per-entry predicate over the matcher stack the walker already builds; classify in the workers and fold an ignored partition in the streaming reducer. Must match the indexed answer exactly (golden and parity corpora, including negation and nested .gitignore). Option C.1 in docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md.

## Notes

2026-09-28: #149 merged on the macOS evidence (exp-170/171) before its Linux cell, at the user's direction; the H161 row in performance-loop.md stays the authority. Since the macOS runs, eeb257c9 added an exact-name confirmation on probe hits (a partial listing up to the .gitignore entry) and a single read of a probed .gitignore; count probe hits with FDU_COUNTERS=1 on linux-v6.12. If Linux misses wall while keeping the RSS win, bring the trade-off to the maintainer rather than reverting silently.

2026-09-28 (Linux session): running the H161 Linux cell here as pre-registered in the H161 row: aggregate-summary bare (controls on), control a5c0ab46 probe vs candidate main 0d73ed54 probe (includes eeb257c9), placebos both arms --no-controls and default-tree, 12 quiet pairs, linux-v6.12 deciding and linux-balanced-1m screening; wall -3% CI<0 and peak RSS down >= 50%. Id: exp-187. FDU_COUNTERS=1 probe-hit count on linux-v6.12 recorded with it.
