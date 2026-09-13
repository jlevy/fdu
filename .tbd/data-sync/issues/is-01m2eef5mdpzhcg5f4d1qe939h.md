---
type: is
id: is-01m2eef5mdpzhcg5f4d1qe939h
title: "PR #52 review BUILD-1: a duplicate readdir name fails the default detached scan"
kind: bug
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:33:38.189Z
updated_at: 2026-09-13T22:35:47.071Z
---
PR #52 review BUILD-1 (High). crates/fdu-core/src/index.rs:1440-1452 and 3628-3643 at afbb2ee. The default fdu PATH takes the detached lane (scan.rs:3509). An enumerator can return one name twice while its directory changes (ext4 htree rename during getdents64, NFS/SMB/FUSE cookies, getattrlistbulk on a changing APFS directory). record_detached_entry pushes both, and push_directory returns UnsupportedScanConfig('detached scan produced a duplicate child name'), or 'duplicate directory path' for a directory, so the CLI exits 1 blaming configuration. The streaming reducer re-upserts and succeeds. A duplicated directory is also walked once per observation, so its repeated listing (and every listing below it) would then fail with UnknownAncestry. Fix: in push_directory, sort by name and fold duplicates before allocating, keeping the last observation as upsert_beneath does; drop the duplicate-directory error; accept the repeated walk below a folded directory. Adopt the review's proof test.
