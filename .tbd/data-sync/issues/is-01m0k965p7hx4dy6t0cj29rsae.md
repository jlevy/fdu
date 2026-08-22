---
type: is
id: is-01m0k965p7hx4dy6t0cj29rsae
title: "Surface parity harness: run the golden corpus against the library and Python APIs"
kind: epic
status: open
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
child_order_hints:
  - is-01m0kaz0bcxmp6162vxqhgtnrt
  - is-01m0kazpvjhxt3b3436wrw0et3
  - is-01m0kazq861vm6kpx8061jwm1a
  - is-01m0kazqkb46cjw4byw8x1ae7k
  - is-01m0kdhmx1wzmh0qaaeqt8kvk7
created_at: 2026-08-21T23:06:08.965Z
updated_at: 2026-08-22T00:22:19.296Z
---
Run the existing golden corpus against every surface, not just the CLI.

fdu ships three ways to ask the same question and Principle 7 claims they are one
capability wearing three faces. Nothing tested that: the CLI has 129 golden sessions, the
other two have a handful of hand-written assertions, and five real defects reached the
Python surface with `make check` green throughout.

Test-only shims reimplement the CLI over the library API and over the Python API; the same
golden files run against each via a `$FDU_PARITY_BIN` PATH switch, so there is no second
corpus. A capability the CLI has and an API lacks becomes unshippable, because the shim
cannot be written without it.

Prerequisite found while validating the mechanism: tryscript passes an unset `path:`
variable through as an empty PATH entry, which POSIX reads as the current directory --
verified, a CWD executable was found and run. A footgun for any tryscript user, and we
maintain tryscript, so it is a fix at the source.

## Notes

Design changed: no PATH fallthrough. Every session runs $FDU, always set to fdu-cli / fdu-py / fdu-rs, and no executable named fdu is on PATH during a parity run. A missing or misspelled shim is command-not-found on the first session rather than a silent pass against the real binary. make test-golden uses the same mechanism with FDU=fdu-cli, so there is no parity-only code path for a tacit failure to hide in.
