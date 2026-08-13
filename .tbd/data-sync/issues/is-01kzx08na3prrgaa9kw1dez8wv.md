---
type: is
id: is-01kzx08na3prrgaa9kw1dez8wv
title: "H63: Derive macOS bulk metadata from report requirements"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:26:55.810Z
updated_at: 2026-08-13T07:33:36.284Z
---
After H62, test a macOS-only getattrlistbulk request/parser for transient summary that omits index-only ctime/inode/device/flags fields unless semantic scope requires them, while retaining name/type/file apparent size/file allocated size/file mtime and fail-closed portable fallback. Add returned-bitmap, malformed-record, mount/firmlink, one-filesystem, symlink, partial-error, and full-vs-compact equivalence tests. Keep the portable backend unchanged.

## Notes

Healey/dumac source audit suggests a strict summary parser can avoid copying names for files and can omit index-only ctime/inode, while preserving names for directories, type, the requested size fields, mtime for rich summary, and mount/firmlink/device fields required by scope semantics. Re-screen 64 vs 128 KiB after reducing record width; exp-029/039 rejected 256 KiB for the full-index record, not 128 KiB for a narrower record. Parse into per-directory staging and commit only after the whole buffer stream validates so portable fallback stays atomic.
