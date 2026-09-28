---
type: is
id: is-01m3j15py91vjyse2zzvhgxaqp
title: "Ignore-aware transient summary: fold the ignored share without retaining an index"
kind: task
status: in_progress
priority: 1
version: 8
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
updated_at: 2026-09-28T16:20:51.317Z
started_at: 2026-09-28T14:30:07.463Z
---
Default fdu --view summary reads .gitignore, and the summary reducer keeps no control table, so the planner falls closed to a full retained index plus a snapshot write. On the 1M balanced Linux tree (no .gitignore files at all) that costs 1.52 s against 0.92 s for --no-gitignore (screen), and 319 MiB against 10 MiB. The ignored share is a per-entry predicate over the matcher stack the walker already builds; classify in the workers and fold an ignored partition in the streaming reducer. Must match the indexed answer exactly (golden and parity corpora, including negation and nested .gitignore). Option C.1 in docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md.

## Notes

2026-09-28: merged to main via #149 (0dec1d85) after review fixes (eeb257c9: exact-name probe, single read), full make check + cross-lint on aa58a6b1, CI green, and the user's go-ahead. The Linux cell (probe aggregate-summary, linux-v6.12 deciding, 12 quiet pairs) still confirms or reverts it; case-variant .gitignore semantics tracked in fdu-0w1b.
