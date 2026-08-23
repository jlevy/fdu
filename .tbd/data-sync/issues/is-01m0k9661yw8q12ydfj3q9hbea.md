---
type: is
id: is-01m0k9661yw8q12ydfj3q9hbea
title: "tryscript: an unset path variable puts the working directory on PATH"
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
created_at: 2026-08-21T23:06:09.341Z
updated_at: 2026-08-23T00:05:43.691Z
---
An unset `path:` variable expands to an empty string and is passed through as an empty
PATH entry, which POSIX resolves as the current directory.

Reproduced: a golden file declaring `path: [$UNSET_VAR]` found and ran an executable in the
working directory. Any tryscript user with a fixture directory containing a file named like
a real tool has that tool silently shadowed.

Drop empty entries, and add a test that a bare `$VAR` does not put the working directory on
PATH. Blocks the parity switch, but worth fixing on its own.

## Notes

Recorded upstream as jlevy/tryscript#55 rather than implemented here; it is a tryscript behaviour and fdu's corpus no longer carries a path: entry that can be empty.

Still a real robustness issue for tryscript generally: an unset variable expands to an empty PATH element, which POSIX reads as the current directory, so a test naming a directory it cannot resolve searches the sandbox instead of failing cleanly.
