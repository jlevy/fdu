---
type: is
id: is-01m0m74rknt5x556mrhx62rfe6
title: "Make tryscript's path: authoritative so the corpus cannot fall through to an installed fdu"
kind: task
status: open
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-22T07:49:40.084Z
updated_at: 2026-08-22T08:02:04.375Z
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

## Notes

CORRECTION to this bead's original premise.

I claimed jlevy/tryscript#51 (env: expands vars, plus TRYSCRIPT_EXE) would let each golden name its own binary and delete scripts/run-golden.mjs. Windows CI disproved that: tryscript runs sessions through cmd.exe, which does not expand $FDU -- it wants %FDU%. A variable in the session command line is not portable, so the corpus is back to invoking a bare `fdu` and naming its directory through `path: - $FDU_BIN`.

#51 is still a correct fix on its own merits (path: and env: disagreeing about $VAR is a real inconsistency) but it does not remove the runner.

What would actually help, and is the real limitation:

  path: PREPENDS to the inherited PATH rather than replacing it for command resolution.

So if $FDU_BIN fails to resolve, lookup continues and finds ~/.cargo/bin/fdu. Today that is caught by a preflight in run-golden.mjs that stats the binary before any session runs -- verified: hiding target/debug/fdu with the installed build still on PATH stops the run with a diagnostic. But that is fdu guarding against a tryscript behaviour, which is the shape of thing this bead exists to remove.

Wanted in tryscript, either:
  - a path mode that is authoritative for command resolution (declared dirs only), or
  - `requires: [fdu]`, asserting the command resolves from a declared path and reporting what it resolved to (fdu-ds2x).

What run-golden.mjs would still do afterwards, and this part is irreducible: pick which surface to run (rust vs the python parity shim) and set $FDU_BIN. Parity needs two surfaces over one corpus, so something has to choose. That is ~51 lines of code today, and the preflight is the part tryscript could take over.
