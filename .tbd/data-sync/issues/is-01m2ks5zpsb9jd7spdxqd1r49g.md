---
type: is
id: is-01m2ks5zpsb9jd7spdxqd1r49g
title: "PR #65 review F1: a watch stream never repaints when a .gitignore edit reclassifies a selected entry"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m2ks5fy0rg2stv9nf1vc7wzb
created_at: 2026-09-16T00:17:03.698Z
updated_at: 2026-09-16T00:17:03.698Z
---
PR #65, review https://github.com/jlevy/fdu/pull/65#pullrequestreview-5217139212. crates/fdu-core/src/watch_session.rs:165-190,:249-255 and crates/fdu/src/cli.rs:779-780,:814-820. Under --watch --view files with --exclude-ignored or --only-ignored, a Reclassified effect maps to None in change_for, ignored_among skips it, and has_aggregates is false so dirty repaints nothing. Carry ignored on Change and repaint --view files on dirty; fall back to refusing the composition for 0.1.0 if it grows.
