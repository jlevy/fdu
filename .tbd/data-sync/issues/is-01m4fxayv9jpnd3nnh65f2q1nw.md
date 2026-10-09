---
type: is
id: is-01m4fxayv9jpnd3nnh65f2q1nw
title: Measure the age column's cost on the default report
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:06.888Z
updated_at: 2026-10-09T09:06:04.179Z
---
Two regimes: (1) paired make perf-compare of the default one-shot report on a real tree, cold and warm, control = ~/fdu-perf/control-148ef78e/perf_probe; (2) a retained-index report timing (Index.report / opened root) at a large entry count, before and after. If (2) regresses measurably, switch to a per-directory activity maximum maintained beside newest_mtime_ns (snapshot fingerprint bump). Record regime and result.
