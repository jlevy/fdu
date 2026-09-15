---
type: is
id: is-01m2kbng6hw8zacm3rzb0kfeqt
title: Cache clear cannot reclaim snapshots from an older format, so the 0.1.0 format bump strands them
kind: bug
status: in_progress
priority: 1
version: 2
labels:
  - release
dependencies: []
created_at: 2026-09-15T20:20:52.047Z
updated_at: 2026-09-15T21:44:04.119Z
---
Found while drafting the 0.1.0 release notes (PR #64). On main, `--cache-clear` and `--cache-clear=all` never delete a snapshot file they do not recognize.

PR A (#63) bumps the snapshot `FORMAT_VERSION`, and fdu-apbl's fingerprint change already cold-rescans. So after upgrading, every existing snapshot is unrecognized and neither command can reclaim it. The plain-text `--cache-status` also prints "No cached snapshots." when every file is unrecognized, which hides them. The draft notes tell users to find them with `--cache-status=all --format json` and delete them by hand, which is poor for a first release.

Fix direction:
- `--cache-clear` recognizes fdu snapshot files of any older format by container magic and naming, and removes them.
- `--cache-status` reports stale-format files instead of "No cached snapshots.".
- It never deletes files that are not fdu snapshots.
- Test with an old-format snapshot beside a current one.

Relates to fdu-558j and fdu-apbl. Parent: fdu-gjc2.
