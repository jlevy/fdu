---
type: is
id: is-01kzqn2fn6p7qcmp31j87qesak
title: "P1: golden coverage for the five-axis surface"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:35:06.917Z
updated_at: 2026-08-11T05:35:06.917Z
---
Extend the tryscript suite to cover the new surface, following golden-testing-guidelines and the existing frontmatter conventions (sandbox: true, fixtures/project, PATH to target/debug, pinned env FORCE_COLOR=0 NO_COLOR=1 LANG/LC_ALL=C TZ=UTC, XDG_CACHE_HOME=.cache for cache scenarios, named patterns for unstable fields). Stable-vs-unstable classification is the load-bearing part: paths, sizes, counts, kinds, schema and generator strings are stable and must match exactly; SCAN_PATH, ALLOCATED, MTIME_NS, and the two new RFC3339 timestamps are unstable and get named patterns - never elide a whole line with '...' where a pattern would keep the field visible. New/updated files: cli-surface (axis composition - view lists, --kind, include/exclude, min-size, sort/reverse/limit, depth vs scan-depth), cli-json (fdu.report/1 for each view and each machine format), cli-human (text rendering per view, files-view pipe-cleanliness), plus error-path blocks for unknown view tokens, duplicate views, bad WHEN and SIZE values, each asserting the exact message and exit code. Run npx tryscript@latest docs before authoring to confirm current pattern/elision syntax.
