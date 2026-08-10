---
type: is
id: is-01kzmyvzzhag70nv3fh7rfhec7
title: Exclude symlinks and special nodes from regular-file roll-ups
kind: bug
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-10T04:28:36.720Z
updated_at: 2026-08-10T04:36:01.409Z
---
The independent contract-corpus engine oracle found that Index::contribution groups File, Symlink, and Other together even though RollUp documents files/bytes/allocated/newest_mtime/by_ext as regular-file-only metrics. On Unix a retained symlink therefore increments files and adds the link-target byte length and allocation to CLI totals. Add focused index and filesystem scan tests, split regular-file contribution from retained non-file nodes, verify snapshot/revalidation behavior, and keep the probe oracle strict.

## Notes

Fixed Index::contribution so only EntryKind::File contributes files/bytes/allocated/newest_mtime/by_ext; Symlink and Other remain retained records with zero regular-file roll-up contribution. Added insertion plus file-to-symlink kind-transition regression coverage. The exact contract probe now agrees with the independent manifest oracle. Awaiting full make check before closure.
