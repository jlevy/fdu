---
type: is
id: is-01m0k9661yw8q12ydfj3q9hbea
title: "tryscript: an unset path variable puts the working directory on PATH"
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
created_at: 2026-08-21T23:06:09.341Z
updated_at: 2026-08-22T07:50:10.091Z
---
An unset `path:` variable expands to an empty string and is passed through as an empty
PATH entry, which POSIX resolves as the current directory.

Reproduced: a golden file declaring `path: [$UNSET_VAR]` found and ran an executable in the
working directory. Any tryscript user with a fixture directory containing a file named like
a real tool has that tool silently shadowed.

Drop empty entries, and add a test that a bare `$VAR` does not put the working directory on
PATH. Blocks the parity switch, but worth fixing on its own.

## Notes

Still valid as a tryscript robustness issue: an empty path: entry expands to an empty PATH element, which POSIX reads as the current directory.

No longer on fdu's critical path. After jlevy/tryscript#51 the corpus carries no path: entries at all -- each golden names its binary through env: instead. See fdu-9h2w for the prepend behaviour that actually broke things here, and fdu-z7sp for the cleanup.
