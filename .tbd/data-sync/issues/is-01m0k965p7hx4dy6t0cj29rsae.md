---
type: is
id: is-01m0k965p7hx4dy6t0cj29rsae
title: "Surface parity harness: run the golden corpus against the library and Python APIs"
kind: epic
status: open
priority: 1
version: 10
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
child_order_hints:
  - is-01m0kaz0bcxmp6162vxqhgtnrt
  - is-01m0kazpvjhxt3b3436wrw0et3
  - is-01m0kazq861vm6kpx8061jwm1a
  - is-01m0kazqkb46cjw4byw8x1ae7k
  - is-01m0kdhmx1wzmh0qaaeqt8kvk7
  - is-01m0nqzynp3vxhfn2rm0c43d8v
created_at: 2026-08-21T23:06:08.965Z
updated_at: 2026-08-22T22:03:22.676Z
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

Phase 1 landed. 108 of 126 golden sessions reach parity through the Python API alone, and every remaining difference carries a named cause -- an unexplained one fails the run rather than joining the list.

Deliberately still OPEN, and not on bead state: Phase 2 has no bead yet and exists only as a spec checkbox, which is invisible to tbd ready. That is the case the update-specs-status triage calls out as the one where closing destroys information.

What remains:
  - Phase 2: the same shim over the public Rust library API, with its own deviation file. Worth more now than when written -- Phase 1 found seven definitions the CLI had copied from the library and five capabilities only the CLI could reach, and a Rust-side shim is the instrument that would have caught them without a second language in the way.
  - fdu-ds2x: tryscript requires:. Superseded in practice (run-golden.mjs preflights and states the surface) but still worth moving the check off fdu.
  - fdu-nluf: tryscript empty path: entries. Real robustness issue, off this critical path now that the corpus carries no path: entry that can be empty.

Two spec items diverged and are recorded in the spec rather than quietly dropped: the shim exits 2 rather than the specified 77, and the legitimate-deviation rule became four named classes rather than two.
