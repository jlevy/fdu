---
type: is
id: is-01kzzf4f7e2mdrn6ea47b09t64
title: "PR #21 R1: guard every uv-backed Make target"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzzedh4wjer6vyq7b0yj782d
created_at: 2026-08-14T06:25:16.001Z
updated_at: 2026-08-14T06:34:52.212Z
closed_at: 2026-08-14T06:34:52.211Z
close_reason: "Fixed: all direct uv-backed targets now depend on uv-version; regression coverage derives recipes from the Make database. make check passed."
---
Review R1 from https://github.com/jlevy/fdu/pull/21#issuecomment-5290202229. Makefile:151-300 leaves python-concurrency, python-smoke, and most perf-* standalone targets without the uv-version prerequisite. Add complete dependency coverage and a regression test.
