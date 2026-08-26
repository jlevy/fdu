---
type: is
id: is-01m0xqneptb5mdg0bck6vxxxhb
title: Plan clean fdu opened-root inventory engine rewrite
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-26T00:31:34.105Z
updated_at: 2026-08-26T01:07:53.637Z
closed_at: 2026-08-26T01:07:53.636Z
close_reason: Produced and validated the linked rewrite plan; implementation remains open under fdu-snej.
resolution: null
duplicate_of: null
---
Design the preferred fdu and MetaBrowser boundary from first principles for a fresh implementation PR. Use PR #47 as evidence rather than a compatibility constraint; reconcile the implemented MetaBrowser PR #74 contract, current fdu design principles, review findings, and open correctness beads. Produce one active plan spec that separates the minimum vertical slice from measured or consumer-driven follow-on capabilities.

## Notes

Completed the clean joint fdu/MetaBrowser rewrite plan at docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md. Reviewed MetaBrowser PR #74 at exact head 3183888808b366b5ba1c381dec1cbb18b49d969e. The plan preserves the coordinator/provider and five-operation handle seam; replaces dual live ownership, observation-derived deltas, semantic exact-prefix max_files, exact runtime remainders, signed self-contained page tokens, and caller-asserted registry fingerprints; and adds explicit MetaBrowser changes plus a dedicated installed-artifact end-to-end integration phase. Implementation epic: fdu-snej. Full fdu make check and final docs-format-check passed.
