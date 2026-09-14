---
type: is
id: is-01m12sw86ewf0ym52fscwxks00
title: Untrusted sidecar path validated with is_absolute in content_cache.rs
kind: bug
status: closed
priority: 2
version: 5
labels:
  - engine-correctness
  - stack-followup
dependencies: []
created_at: 2026-08-27T23:46:26.125Z
updated_at: 2026-09-14T02:59:03.389Z
closed_at: 2026-09-14T02:59:03.388Z
close_reason: "f00bad5, 324048c: the sidecar parser rejects any record whose path has a component other than Normal or CurDir (private record_path_stays_inside_root in content_cache.rs, not the index helper #51 removes), so .., rooted-without-drive and drive-relative paths make the sidecar a clean miss; fixture test covers absolute, .., nested .., and on Windows drive-absolute, rooted and drive-relative paths plus a restoring relative path."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/content/content_cache.rs validates an untrusted sidecar path with relative_path.is_absolute(). This is the same defective question PR #47 commit 5ace86c corrected elsewhere: is_absolute answers false for a Windows rooted path with no drive prefix, and '..' slips past it on every platform including Unix.

PR #47 identified this fourth instance and deliberately filed it rather than fixing it, on the grounds that it was outside that PR's subject and wants a fixture of its own.

This one is live in main today, independent of the opened-root rewrite. Fix it with the same component-based rule (crate::index::path_is_representable) and give it its own fixture covering absolute, rooted-without-drive, and '..' inputs.

## Notes

2026-09-13 (PR #48 review 5192314101, prior findings): still live at c853f7c, and listed in #48's open technical debt. The rewrite does not widen it, because opened-root observation paths and snapshot names are validated separately.
