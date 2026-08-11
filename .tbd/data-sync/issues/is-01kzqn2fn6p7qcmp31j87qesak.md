---
type: is
id: is-01kzqn2fn6p7qcmp31j87qesak
title: "P1: golden coverage for the five-axis surface"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:35:06.917Z
updated_at: 2026-08-11T16:29:11.654Z
closed_at: 2026-08-11T16:29:11.654Z
close_reason: "Added tests/golden/cli-axes.tryscript.md (26 blocks: every axis alone and combined, all four formats, 11 error paths) and migrated/re-recorded the three existing golden files; 52 blocks green. Unstable fields use named patterns, not elisions. The goldens found four defects unit tests missed: usage errors exited 1 instead of 2; the JSON tree emitted balanced-but-invalid [{a}{b},] (fixture had no multi-child directory - regression test added); JSONL collapse left '[ {'; and the renderer swap had silently dropped human colour and the per-path errors list from partial results. All fixed."
---
Extend the tryscript suite to cover the new surface, following golden-testing-guidelines and the existing frontmatter conventions (sandbox: true, fixtures/project, PATH to target/debug, pinned env FORCE_COLOR=0 NO_COLOR=1 LANG/LC_ALL=C TZ=UTC, XDG_CACHE_HOME=.cache for cache scenarios, named patterns for unstable fields). Stable-vs-unstable classification is the load-bearing part: paths, sizes, counts, kinds, schema and generator strings are stable and must match exactly; SCAN_PATH, ALLOCATED, MTIME_NS, and the two new RFC3339 timestamps are unstable and get named patterns - never elide a whole line with '...' where a pattern would keep the field visible. New/updated files: cli-surface (axis composition - view lists, --kind, include/exclude, min-size, sort/reverse/limit, depth vs scan-depth), cli-json (fdu.report/1 for each view and each machine format), cli-human (text rendering per view, files-view pipe-cleanliness), plus error-path blocks for unknown view tokens, duplicate views, bad WHEN and SIZE values, each asserting the exact message and exit code. Run npx tryscript@latest docs before authoring to confirm current pattern/elision syntax.
