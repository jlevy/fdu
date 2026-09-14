---
type: is
id: is-01m2gtp61gh3c4j4frh1q9sw88
title: "Remove the gitignore build feature: ignore handling is always compiled in, and read_controls alone decides whether .gitignore is read"
kind: task
status: closed
priority: 1
version: 3
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T20:45:39.759Z
updated_at: 2026-09-14T21:53:12.768Z
closed_at: 2026-09-14T21:53:12.767Z
close_reason: "PR #60 (https://github.com/jlevy/fdu/pull/60), stacked on #57: 077318c removes the gitignore build feature and all 78 cfg sites, with the lib-only matrix, CI, and perf recipes updated; bb26446 reverts 30c3895's opened roll-up workaround; fd62205 and df43bdd update docs and add the build-feature terminology to AGENTS.md. make check and make cross-lint pass locally."
resolution: null
duplicate_of: null
---
DECISION (user, 2026-09-14): the gitignore feature makes no sense as a compile-time option. It is a built-in capability and always compiled in. Whether a scan reads .gitignore control state is decided only at runtime, by ScanConfig::read_controls (default off for one-shot reports and library open after #57; the opened root always on). watch is a different kind of thing, with a real dependency tree (notify plus its platform backends) and a stated deletability principle, and stays a feature.

Why: the gitignore feature pulls in no dependency (the matcher is in-tree, about 1,300 lines). Its stated purpose, keeping filesystem reads removable, is now served by the runtime switch. It costs 78 cfg sites in 11 files, a 4-way lib-only test matrix, and a build shape where 'observes controls' means something different. That shape produced the featureless opened roll-up failure fixed in 30c3895 and the open question on fdu-x3yt.

Scope:
- Remove the feature from fdu-core, fdu and fdu-py Cargo.toml.
- Delete every cfg(feature = "gitignore") gate.
- Revert 30c3895's featureless workaround, since opened roots now always observe.
- Update the lib-only matrix in the Makefile and CI.
- Update perf-probe-release and any harness or doc passing --features gitignore.
- Update docs: Cargo comments, surface and engine architecture, the plan, and the README.
- Leave historical evidence records unchanged.
- Close fdu-x3yt as moot.
