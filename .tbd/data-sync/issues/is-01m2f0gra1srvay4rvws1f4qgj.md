---
type: is
id: is-01m2f0gra1srvay4rvws1f4qgj
title: Decide whether --skill names --watch in a command line built without it
kind: task
status: open
priority: 4
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T03:49:04.448Z
updated_at: 2026-09-14T03:49:04.448Z
---
Found while fixing fdu-224p at 0989b72 on codex/opened-root-inventory-rewrite.

0989b72 composes the `--docs` guide per build, so a command line built without `watch` no longer names `--watch` or `--interval`. The `--skill` text is not composed: `crates/fdu/src/skills/SKILL.md` at 0989b72 names `--watch` at line 118 ("cannot be combined with `--watch`") and line 168 ("a `--watch` stream carries `fdu.stream/1`"), and `compose_skill` prints it unchanged in every build.

Neither sentence tells a reader to run `--watch`, and the skill calls itself portable, so this may be intended. Decide whether the skill describes fdu or this binary. If this binary, gate those sentences the way `docs_guide!` gates the guide, and extend `the_guide_only_names_flags_that_exist` (or a sibling) to `compose_skill()`, which `make lib-only` now runs in the featureless shape. If fdu, say so beside `compose_skill` so the next reader does not file this again. Low priority.
