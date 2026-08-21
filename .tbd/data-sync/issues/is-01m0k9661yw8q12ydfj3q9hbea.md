---
type: is
id: is-01m0k9661yw8q12ydfj3q9hbea
title: "tryscript: an unset path variable puts the working directory on PATH"
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
created_at: 2026-08-21T23:06:09.341Z
updated_at: 2026-08-21T23:37:35.593Z
---
An unset `path:` variable expands to an empty string and is passed through as an empty
PATH entry, which POSIX resolves as the current directory.

Reproduced: a golden file declaring `path: [$UNSET_VAR]` found and ran an executable in the
working directory. Any tryscript user with a fixture directory containing a file named like
a real tool has that tool silently shadowed.

Drop empty entries, and add a test that a bare `$VAR` does not put the working directory on
PATH. Blocks the parity switch, but worth fixing on its own.
