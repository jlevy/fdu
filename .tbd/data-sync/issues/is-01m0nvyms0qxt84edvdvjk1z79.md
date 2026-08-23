---
type: is
id: is-01m0nvyms0qxt84edvdvjk1z79
title: Write the three-surface architecture document
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01m0nvyn4g0xyf31aby8cr9rwm
parent_id: is-01m0nvym0yjmvv43cme79f3cd0
created_at: 2026-08-22T23:12:34.079Z
updated_at: 2026-08-22T23:52:59.152Z
closed_at: 2026-08-22T23:52:59.152Z
close_reason: The surface-parity principle is stated in fdu-design-principles.md with the evidence that both rules were broken while believed to hold; fdu-surface-architecture.md says what each surface is, how the boundary and the harness work, and what each deviation class means; AGENTS.md points at both plus the two traps (--update expands patterns, the artifact is recorded by CI).
---
There is no document that says what the three surfaces are, what each is for, which is authoritative, and how they are kept in agreement. A reader today infers it from Cargo.toml and two spec files.

docs/project/architecture/: what fdu-core is, what fdu adds, what fdu-py binds, and the rules between them. Include the shape of the packages and why (fdu is what a user installs; fdu-core is the engine), the parity harness and what its deviation classes mean, and the one-shot versus session contract, which is a real behavioural difference a caller can hit.
