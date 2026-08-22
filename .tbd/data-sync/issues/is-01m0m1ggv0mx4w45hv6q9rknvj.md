---
type: is
id: is-01m0m1ggv0mx4w45hv6q9rknvj
title: Golden PATH falls through to the installed fdu when target/debug does not resolve
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-22T06:11:13.887Z
updated_at: 2026-08-22T06:53:19.370Z
closed_at: 2026-08-22T06:53:19.369Z
close_reason: "Closed by naming the executable through $FDU instead of searching PATH. The corpus no longer carries path: entries, so there is no lookup left to fall through; an unset variable is 'command not found' on the first session. Demonstrated by hiding target/debug/fdu with the installed build still on PATH: the run stops with the path it wanted instead of going green. check-golden-invocations.mjs keeps the bare name from returning, and is proven to fire on all three forms."
---
tryscript's `path:` front matter PREPENDS to the inherited system PATH rather than replacing it. Every golden file relies on `path: - $TRYSCRIPT_GIT_ROOT/target/debug` to select the build under test.

If that entry ever fails to resolve -- variable unset, target/ removed, a typo, or a cross-compiled layout -- PATH lookup silently continues into the inherited PATH and finds whatever `fdu` is installed there. On this machine that is ~/.cargo/bin/fdu. The suite then passes while testing a completely different binary, with no diagnostic.

Reproduced with tryscript 0.2.0: a file whose only path entry is an unset variable resolves `command -v fdu` to /Users/levy/.cargo/bin/fdu.

An unset entry also expands to an empty PATH element, which POSIX reads as the current directory (see fdu-nluf).

This is the tacit-failure mode that makes parity testing unsafe: the whole point is that fdu-rs and fdu-py are different binaries, so 'which binary actually ran' has to be asserted, never assumed. Fix by adding a preflight that resolves the binary the same way PATH would and asserts it is the intended path and implementation, before any golden runs.
