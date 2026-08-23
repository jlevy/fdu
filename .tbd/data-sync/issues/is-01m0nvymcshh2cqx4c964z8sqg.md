---
type: is
id: is-01m0nvymcshh2cqx4c964z8sqg
title: State the surface-parity principle in the design principles
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01m0nvyn4g0xyf31aby8cr9rwm
parent_id: is-01m0nvym0yjmvv43cme79f3cd0
created_at: 2026-08-22T23:12:33.688Z
updated_at: 2026-08-22T23:52:59.142Z
closed_at: 2026-08-22T23:52:59.141Z
close_reason: The surface-parity principle is stated in fdu-design-principles.md with the evidence that both rules were broken while believed to hold; fdu-surface-architecture.md says what each surface is, how the boundary and the harness work, and what each deviation class means; AGENTS.md points at both plus the two traps (--update expands patterns, the artifact is recorded by CI).
---
docs/project/architecture/fdu-design-principles.md is the highest-level design document and the one AGENTS.md tells agents to read before changing engine behaviour. Principle 7 already says 'same concepts at every level; the CLI invents nothing'. It does not say how that is known to be true, or what the second rule is.

Add, at principle level:
  - The CLI invents nothing, enforced by a crate boundary rather than by review: the command line depends on the engine the way any consumer does, so a private item it needs becomes a visible act of making something public.
  - Every surface gives the same answer, enforced by replaying one golden corpus against each: differences are recorded, classified, and an unexplained one fails the build.

Both should carry the evidence that they matter, because both were broken and the breakage was invisible: the binding drifted behind the CLI five times with make check green, and the library depended on its own front end to format a number.
