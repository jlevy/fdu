---
type: is
id: is-01m0k965p7hx4dy6t0cj29rsae
title: "Surface parity harness: run the golden corpus against the library and Python APIs"
kind: epic
status: closed
priority: 1
version: 14
spec_path: docs/project/specs/done/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
child_order_hints:
  - is-01m0kaz0bcxmp6162vxqhgtnrt
  - is-01m0kazpvjhxt3b3436wrw0et3
  - is-01m0kazq861vm6kpx8061jwm1a
  - is-01m0kazqkb46cjw4byw8x1ae7k
  - is-01m0kdhmx1wzmh0qaaeqt8kvk7
  - is-01m0nqzynp3vxhfn2rm0c43d8v
  - is-01m0nv9134ddskjzyam5v3hjx0
  - is-01m0nvz764v76702g0m1q40vhx
  - is-01m0nvz7j00dm6w1snefmvw713
created_at: 2026-08-21T23:06:08.965Z
updated_at: 2026-08-23T00:05:56.539Z
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

Closed with the spec at zero unchecked items and every child closed -- both checked, not inferred from bead state.

108 of 126 golden sessions reach parity through the Python API alone; the 20 that differ each carry a named cause and an unexplained one fails the build. Phase 2 was replaced by the CLI-on-the-public-API split (fdu-s74c), which enforces the same property with a crate boundary rather than a second command line.

The two remaining items were tryscript behaviours, not fdu work, and are recorded upstream as jlevy/tryscript#54 and #55.
