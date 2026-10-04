# fast-glob Stand-in for tryscript

tryscript, the golden-test runner, finds its test files with one `fast-glob` call.
fast-glob depends on micromatch, which depends on braces, and every published braces
version is affected by
[GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm)
(CVE-2026-93687, a stack-exhaustion denial of service on deeply nested brace patterns)
with no patched release.
`npm audit --audit-level=moderate` in `make check` and CI therefore failed on every
branch from 2026-09-18.

This package replaces fast-glob in the development tree instead of excusing the
advisory. The root `package.json` installs it as a devDependency and points tryscript’s
`fast-glob` at it through `overrides` (`"$fast-glob"`), which removes fast-glob,
micromatch, and braces from the lockfile.
It implements fast-glob’s call surface on
[tinyglobby](https://github.com/SuperchupuDev/tinyglobby), whose matcher (picomatch) has
no dependency on braces.

Only the options tryscript passes are translated (`ignore`, `absolute`, `dot`) plus two
with identical meaning and defaults (`cwd`, `onlyFiles`). Any other option throws, so a
tryscript release that relies on more of fast-glob fails loudly instead of matching a
different file set. `expandDirectories` is forced off, because fast-glob never expands a
directory pattern.

Before the switch, fast-glob 3.3.3 and this stand-in returned identical file sets for
the golden suite’s pattern and seven edge cases (relative and absolute patterns, `cwd`,
dotfiles in and out, directory patterns, brace alternatives); the golden suite passes
through it.

Remove this package and its override when either tryscript stops depending on fast-glob
or braces publishes a fixed release.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
