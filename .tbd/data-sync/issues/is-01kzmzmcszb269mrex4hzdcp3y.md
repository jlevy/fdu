---
type: is
id: is-01kzmzmcszb269mrex4hzdcp3y
title: Use directory-entry-relative metadata in the portable walker
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzg49rw1p40pjc18feb9ghpv
created_at: 2026-08-10T04:41:56.286Z
updated_at: 2026-08-10T04:49:02.554Z
closed_at: 2026-08-10T04:49:02.553Z
close_reason: Implemented in all three portable enumeration paths, covered by non-following symlink regression, validated by 42 exact-oracle paired samples and a complete make check (all Rust feature matrices, 26 golden blocks, 56 performance tests, docs/audits/Python/wheel/uvx). Evidence is self-contained in docs/project/research/research-2026-08-09-portable-direntry-metadata.md.
---
The release probe measured a 100k balanced baseline of 577.5 ms scan-producer, 693.0 ms scan-index, and 834.7 ms unchanged revalidation on the same warm APFS corpus. Replace path-based symlink_metadata(item.path()) with DirEntry::metadata() in all directory enumeration loops so the standard library can use the already-open directory handle and avoid allocating/re-resolving an absolute path per entry. Preserve no-follow semantics, exact engine digest, partial errors, and cross-platform behavior; accept only with same-corpus before/after medians and full make check.

## Notes

Implemented DirEntry::metadata in scan, observation-only revalidate, and applying reconcile, retaining full paths only for errors. Added a Unix regression proving a directory symlink is retained and its external target is not traversed. Seven alternating old/new release-binary pairs on one exact balanced-100k corpus all matched the engine oracle. Paired median changes: scan-producer -8.24%, scan-index -7.80%, unchanged revalidate -6.84%. A separate six-point batch sweep found no stable benefit, so batch size remains 1024. Evidence: docs/project/research/research-2026-08-09-portable-direntry-metadata.md. Awaiting full make check.
