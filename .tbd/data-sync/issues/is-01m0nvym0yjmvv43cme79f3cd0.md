---
type: is
id: is-01m0nvym0yjmvv43cme79f3cd0
title: Document the surface-parity architecture at the level it belongs
kind: epic
status: closed
priority: 1
version: 5
labels: []
dependencies: []
child_order_hints:
  - is-01m0nvymcshh2cqx4c964z8sqg
  - is-01m0nvyms0qxt84edvdvjk1z79
  - is-01m0nvyn4g0xyf31aby8cr9rwm
created_at: 2026-08-22T23:12:33.308Z
updated_at: 2026-08-22T23:52:59.487Z
---
Three surfaces answer one question -- the Rust engine, the Python binding, and the command line -- and two rules hold them together: the CLI invents nothing beyond the engine, and every surface gives the same answer. Both are now enforced mechanically, and neither is written down where a reader would look first.

The enforcement exists (a crate boundary for the first, a parity harness for the second). What is missing is the design-level statement of WHY, so the next person extending fdu knows the rules are load-bearing rather than incidental, and so an agent reading AGENTS.md finds them before writing code that breaks them.

## Notes

All three children closed and the documents exist. Checked the spec side too: no unchecked items remain in the architecture work.
