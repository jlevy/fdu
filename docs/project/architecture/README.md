# fdu Architecture

## Overview

This directory contains the durable design authorities for fdu.
They describe principles and system boundaries that should remain useful across
implementation plans, pull requests, and releases.

Architecture documents use descriptive, undated filenames and present-tense contracts.
Git records when a decision changed.
Dated plans, research, experiments, and reports record delivery, evidence, and history
under the corresponding `docs/project` directories.

## Document Map

| Document | Owns | Read it when |
| --- | --- | --- |
| [Design principles](fdu-design-principles.md) | Why the system behaves as it does: truth, defaults, bounds, query semantics, performance evidence, and dependency boundaries | Choosing a default, ordering, output shape, trust claim, performance change, or new abstraction |
| [Engine architecture](fdu-engine-architecture.md) | How `fdu-core` owns facts, commits, detached and opened lifecycles, persistence, observation, reads, paging, testing, and shutdown | Changing scanning, indexing, reconciliation, snapshots, live inventory, refresh, journals, continuations, or engine-side tests |
| [Surface architecture](fdu-surface-architecture.md) | How the Rust engine, command line, Python package, parity harness, and application adapters fit together | Adding public capability, changing packaging, exposing a binding, or integrating an interactive client |

Read the principles first.
Then read the engine or surface document for the boundary being changed.
A change spanning both should update both in the same review without copying one
document’s detail into the other.

## Where a Decision Belongs

- Put a durable reason or constraint in the design principles.
- Put engine ownership, data flow, state, and interface contracts in the engine
  architecture.
- Put crate, package, binding, parity, and application-adapter boundaries in the surface
  architecture.
- Put file- and function-level work, sequencing, migration, and acceptance gates in a
  dated plan under [`specs`](../specs/).
- Put investigation evidence in [`research`](../research/) and an evaluated outcome in
  [`reports`](../reports/).
- Put measured performance trials in [`experiments`](../experiments/).

If a plan and an architecture document disagree about a durable boundary, revise and
review the architecture first.
If code and architecture disagree, either bring the code into conformance or make the
design change explicit; do not describe both as authoritative.

## Maintenance

Keep this set small.
Create another architecture document only when a subsystem has an independent audience
and enough stable internal structure that adding it to an existing authority would blur
ownership.

Architecture updates should:

- state contracts rather than branch or rollout status;
- link to detail instead of duplicating it;
- preserve the distinction between principles, engine mechanisms, and public surfaces;
- revise open questions and potential improvements when a decision settles;
- update repository guidance and active-plan references when a file is renamed.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
