---
type: is
id: is-01m4t5r6f12yjxnpyq8nd7chy5
title: Run final candidate regression and stability gates after disk cleanup
kind: task
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-13T08:23:33.588Z
updated_at: 2026-10-13T09:24:54.869Z
started_at: 2026-10-13T08:24:05.795Z
closed_at: 2026-10-13T09:24:54.868Z
close_reason: "All 15 final-candidate automated stability stages passed on e6b8850b: full make check, five-target cross-lint, semver check, local release rehearsal, exact rehearsed wheel install, 49/49 installed CLI checks, 65 explained peer readings, 38/38 PTY checks, three terminal tests, all correctness matrices and negative controls. Initial PTY preflight fixture was too fast; preserved evidence and reran only that stage with Applications. Timed executable/source/cache were internal. Loaded-host timings are exploratory and do not clear Linux performance. Temporary worktree, build outputs, environments, fixtures and internal runtime removed; state, logs, report and release packages retained. Historical Library disposition and human/Windows QA remain open."
resolution: null
duplicate_of: null
---
Run final e6b8850b release regression/stability gates with external build/test scratch. Any performance measurements use internal executable, source and caches per user request; preserve evidence and remove generated files after checks. Do not waive Linux performance, Library discrepancy, or human terminal gates.
