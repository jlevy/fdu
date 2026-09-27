---
type: is
id: is-01m3j15py91vjyse2zzvhgxaqp
title: "Ignore-aware transient summary: fold the ignored share without retaining an index"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T18:13:56.296Z
updated_at: 2026-09-27T18:13:56.296Z
---
Default fdu --view summary reads .gitignore, and the summary reducer keeps no control table, so the planner falls closed to a full retained index plus a snapshot write. On the 1M balanced Linux tree (no .gitignore files at all) that costs 1.52 s against 0.92 s for --no-gitignore (screen), and 319 MiB against 10 MiB. The ignored share is a per-entry predicate over the matcher stack the walker already builds; classify in the workers and fold an ignored partition in the streaming reducer. Must match the indexed answer exactly (golden and parity corpora, including negation and nested .gitignore). Option C.1 in docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md.
