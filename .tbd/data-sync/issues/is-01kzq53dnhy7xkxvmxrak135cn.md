---
type: is
id: is-01kzq53dnhy7xkxvmxrak135cn
title: "Snapshot format: versioned journal-cursor section"
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53dw7bawahp6m77sx9gd4
  - type: blocks
    target: is-01kzrt4tjjx3jj50w8cvb1wgfp
created_at: 2026-08-11T00:56:00.432Z
updated_at: 2026-09-15T00:31:54.486Z
---
Phase 1 of the FSEvents-scoped revalidation plan, revised 2026-09-13. Use the next available format version at integration: main uses v2 and the open engine stack already uses unrelated v3 fields without a cursor. Add an optional journal-cursor section (platform tag, volume UUID, applied event ID, pre-scan capture time), absent-cursor stub on unsupported paths, and fail-closed decoding. Keep incompatible legacy snapshots as clean misses unless an explicit supported reader is justified and tested. Include platform-neutral cursor/gate types and changed-scope normalization; cover all current G1-G12 rows and FullHistory overlap. Publish the applied cursor atomically with its reconciled inventory and never advance it past unapplied work. This format work does not itself implement immutable daily comparison checkpoints; see fdu-8ybz and the linked checkpoint plan.

## Notes

2026-09-14 (PR #55 review, 4727de0): corrections to the description above. main (dda7e6a) writes snapshot format version 3 (crates/fdu-core/src/snapshot.rs:60), which has no replay cursor; "main uses v2" is stale, and the next available version is still the rule. The FSEvents plan now names the section replay_cursor, the module history_replay, the type ReplayCursor, the G5 constant max_cursor_age, and the build feature history-replay, so none collides with the index journal (crates/fdu-core/src/opened/journal.rs).

2026-09-15 (PR #55 delta review 5204152578, 55ce4a3): container and gate decisions from the checkpoint plan.
- The snapshot is the working inventory, and the replay cursor stays in it: both are derived state under the cache's VERSION + FAIL FAST contract, with one publication boundary. A release upgrade (engine_fingerprint), --cache-clear, or a scope change discards both, and the next open takes G2. Checkpoints live in a separate store and are unaffected.
- The typed-gap section shares this snapshot. It may land at the same or an adjacent FORMAT_VERSION; take the next available one. With it, save persists Coverage::Partial(Inaccessible) only when every incomplete directory is a recorded gap (today save refuses any non-Complete coverage, snapshot.rs:189 at dda7e6a).
- The cursor UUID is the FSEvents database UUID from FSEventsCopyUUIDForDevice, which changes on purge or wrap. It is not the filesystem volume UUID that checkpoints record.
- G3 also sweeps when the root's device number differs from the one the snapshot recorded: a renumbered st_dev changes every Fingerprint, and a scoped refresh would leave entries under two device numbers.
