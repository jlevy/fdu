---
type: is
id: is-01kzx08na3prrgaa9kw1dez8wv
title: "H63: Derive macOS bulk metadata from report requirements"
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:26:55.810Z
updated_at: 2026-08-13T10:08:32.183Z
---
After H62, test a macOS-only getattrlistbulk request/parser for transient summary that omits index-only ctime/inode/device/flags fields unless semantic scope requires them, while retaining name/type/file apparent size/file allocated size/file mtime and fail-closed portable fallback. Add returned-bitmap, malformed-record, mount/firmlink, one-filesystem, symlink, partial-error, and full-vs-compact equivalence tests. Keep the portable backend unchanged.

## Notes

H63 begins from the temporary H62 prototype solely as a combined screen. H62 alone is rejected. Implement a separate strict macOS summary bulk reader that requests returned attrs/error/name/device/type/mtime/flags plus logical+allocated sizes, omitting ctime/inode. Preserve directory names and mount/firmlink/one-filesystem fallback; file names may be parsed/validated without allocation. Compare the complete composition against committed H59 and revert both if wall misses 3%.
