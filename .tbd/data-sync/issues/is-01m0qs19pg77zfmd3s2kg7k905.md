---
type: is
id: is-01m0qs19pg77zfmd3s2kg7k905
title: Progressive goldens for both traversal orders, and the tagged fixture
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T17:00:04.176Z
updated_at: 2026-08-23T17:00:04.176Z
---
The headline test this whole plan exists to enable. Today the ONLY test of traversal order is that both orders produce identical engine digests — a proof the order does not matter to the final answer, which is the opposite of the property the order exists for. The consumer-visible property is that under breadth-first the top-level children grow together while under depth-first one completes while its siblings read zero, and a progressive golden over a small fixture shows exactly that in a diff. Record at --threads 1 with each order over a fixture with several top-level subtrees, --progress-at depth (not entries:N, whose frame count depends on tree size and would blow the size budget). Monotonicity asserted as a relation across frames rather than by eye. Also add the tagged fixture the plane goldens need: a tree with a .gitignore including a negation, under tests/golden/fixtures/. Every unstable value gets a named pattern, never a bare [..] elision — [CLOCK], [ALLOCATED], [MTIME_NS], [STAMP], [DIR_BYTES] already exist. make portability rejects anything --update expanded into a literal, a failure this corpus has had twice.
