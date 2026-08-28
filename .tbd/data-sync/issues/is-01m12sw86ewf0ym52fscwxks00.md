---
type: is
id: is-01m12sw86ewf0ym52fscwxks00
title: Untrusted sidecar path validated with is_absolute in content_cache.rs
kind: bug
status: open
priority: 2
version: 2
labels:
  - engine-correctness
dependencies: []
created_at: 2026-08-27T23:46:26.125Z
updated_at: 2026-08-28T00:15:03.828Z
---
crates/fdu-core/src/content/content_cache.rs validates an untrusted sidecar path with relative_path.is_absolute(). This is the same defective question PR #47 commit 5ace86c corrected elsewhere: is_absolute answers false for a Windows rooted path with no drive prefix, and '..' slips past it on every platform including Unix.

PR #47 identified this fourth instance and deliberately filed it rather than fixing it, on the grounds that it was outside that PR's subject and wants a fixture of its own.

This one is live in main today, independent of the opened-root rewrite. Fix it with the same component-based rule (crate::index::path_is_representable) and give it its own fixture covering absolute, rooted-without-drive, and '..' inputs.
