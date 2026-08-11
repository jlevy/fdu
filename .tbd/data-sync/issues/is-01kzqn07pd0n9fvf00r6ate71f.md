---
type: is
id: is-01kzqn07pd0n9fvf00r6ate71f
title: "P1: Selection type and glob-matcher dependency decision"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn0ryk5bywq86c1f4k50fe
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:33:53.228Z
updated_at: 2026-08-11T05:34:10.898Z
---
query module: Selection { include/exclude globs (repeatable flags, never comma-split since brace globs contain commas), min_size, kinds (file|dir|symlink), modified window as half-open [since, before) with since inclusive and before exclusive, render depth (all = unbounded, 0 = root totals only per du), limit (-n all), sort (size|count|mtime|name) plus reverse, size metric (allocated|apparent) }. Evaluated at view time against the retained index; never part of the cache key, so one cache serves every query (tag-don't-prune). Blocking sub-decision recorded in deny.toml under the 14-day cool-off: globset (compiles all patterns into one matcher - the approach the rollup research documents as making gitignore cost O(1) per entry) versus a small first-party glob. Decide before implementing include/exclude; measure against the research's H-registry gitignore item if the choice is close.
