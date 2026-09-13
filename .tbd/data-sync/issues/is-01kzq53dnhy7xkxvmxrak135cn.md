---
type: is
id: is-01kzq53dnhy7xkxvmxrak135cn
title: "Snapshot format: versioned journal-cursor section"
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53dw7bawahp6m77sx9gd4
  - type: blocks
    target: is-01kzrt4tjjx3jj50w8cvb1wgfp
created_at: 2026-08-11T00:56:00.432Z
updated_at: 2026-09-13T21:16:42.524Z
---
Phase 1 of the FSEvents-scoped revalidation plan, revised 2026-09-13. Use the next available format version at integration: main uses v2 and the open engine stack already uses unrelated v3 fields without a cursor. Add an optional journal-cursor section (platform tag, volume UUID, applied event ID, pre-scan capture time), absent-cursor stub on unsupported paths, and fail-closed decoding. Keep incompatible legacy snapshots as clean misses unless an explicit supported reader is justified and tested. Include platform-neutral cursor/gate types and changed-scope normalization; cover all current G1-G12 rows and FullHistory overlap. Publish the applied cursor atomically with its reconciled inventory and never advance it past unapplied work. This format work does not itself implement immutable daily comparison checkpoints; see fdu-8ybz and the linked checkpoint plan.
