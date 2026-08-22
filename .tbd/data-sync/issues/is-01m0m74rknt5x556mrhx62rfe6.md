---
type: is
id: is-01m0m74rknt5x556mrhx62rfe6
title: Delete scripts/run-golden.mjs once tryscript ships env-var expansion
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-22T07:49:40.084Z
updated_at: 2026-08-22T07:49:40.084Z
---
jlevy/tryscript#51 makes `env:` front matter expand variables the way `path:` already did, and adds TRYSCRIPT_EXE for the Windows suffix. Together those let a golden name its own binary:

    env:
      FDU: $TRYSCRIPT_GIT_ROOT/target/debug/fdu$TRYSCRIPT_EXE

That is the entire reason scripts/run-golden.mjs exists. Once a tryscript release carries the fix:

1. Bump the tryscript pin in package.json (respecting the 14-day cool-off; tryscript is firstParty so the waiver already covers it).
2. Add the env: block to each tests/golden/*.tryscript.md.
3. Delete scripts/run-golden.mjs and point package.json's test:golden back at `tryscript run 'tests/golden/*.tryscript.md'`.
4. In scripts/run-parity.mjs, rewrite that one env: line in the generated corpus instead of setting FDU_SURFACE/FDU_CORPUS.

The wrapper has already cost two CI failures of its own -- a Windows backslash glob, and having to reimplement colour suppression and stream selection because its output became the artifact. Removing it removes that class of bug.

Keep check-golden-invocations.mjs: the rule it enforces (never resolve fdu through PATH) still matters, and the env: form is what it should then require.
