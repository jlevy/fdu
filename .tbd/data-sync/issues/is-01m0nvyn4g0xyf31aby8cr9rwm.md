---
type: is
id: is-01m0nvyn4g0xyf31aby8cr9rwm
title: Reference the parity rules from AGENTS.md
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0nvym0yjmvv43cme79f3cd0
created_at: 2026-08-22T23:12:34.447Z
updated_at: 2026-08-22T23:52:59.161Z
closed_at: 2026-08-22T23:52:59.161Z
close_reason: The surface-parity principle is stated in fdu-design-principles.md with the evidence that both rules were broken while believed to hold; fdu-surface-architecture.md says what each surface is, how the boundary and the harness work, and what each deviation class means; AGENTS.md points at both plus the two traps (--update expands patterns, the artifact is recorded by CI).
---
AGENTS.md is what an agent reads first and is the only file guaranteed to be read. It points at the design principles for engine behaviour and at the performance loop for speed work. It says nothing about the three surfaces, so an agent can add a capability to the CLI without knowing the library must be able to do it too, or change output without knowing a corpus pins it.

Add a short section pointing at both new documents, in the style of the existing ones: what the rule is, and where to read why.
