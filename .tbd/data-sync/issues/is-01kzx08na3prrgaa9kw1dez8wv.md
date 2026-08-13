---
type: is
id: is-01kzx08na3prrgaa9kw1dez8wv
title: "H63: Derive macOS bulk metadata from report requirements"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:26:55.810Z
updated_at: 2026-08-13T10:25:01.180Z
closed_at: 2026-08-13T10:25:01.179Z
close_reason: Measured and rejected as exp-042; composition missed the wall gate, so H62/H63 engine prototypes were reverted.
---
After H62, test a macOS-only getattrlistbulk request/parser for transient summary that omits index-only ctime/inode/device/flags fields unless semantic scope requires them, while retaining name/type/file apparent size/file allocated size/file mtime and fail-closed portable fallback. Add returned-bitmap, malformed-record, mount/firmlink, one-filesystem, symlink, partial-error, and full-vs-compact equivalence tests. Keep the portable backend unchanged.

## Notes

Rejected as exp-042 after a 16-pair mutation-free screen on the self-contained 901,963-entry APFS tree. H62+H63 versus committed H59 changed wall +1.857% with 95% CI [-1.959%, +4.558%]; user CPU -50.958%, RSS -39.697%, minor faults -32.318%, system CPU unclear. Identical semantic hash, zero invalid/mismatch/drift/mutation. The duplicate reduction walker and macOS parser were reverted.
