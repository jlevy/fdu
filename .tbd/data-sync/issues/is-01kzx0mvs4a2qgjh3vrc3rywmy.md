---
type: is
id: is-01kzx0mvs4a2qgjh3vrc3rywmy
title: "H64: Derive a selected-total-only execution plan"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - design-gate
dependencies: []
parent_id: is-01kzwk20kyaxajq254tee8apts
created_at: 2026-08-13T07:33:35.651Z
updated_at: 2026-08-13T10:28:29.303Z
---
Design and prototype a typed total projection that requests only the selected apparent or allocated byte total, retains no index, and gathers no counts, recency, extensions, or unused size metric. Keep the existing summary contract unchanged and avoid a benchmark-only fast flag. Establish a clean Rust/Python/CLI surface in a separate functionality decision if required, then compare the derived scanner with dumac on the million-entry APFS tree. Preserve FDU path-counted semantics and disclose dumac hard-link/symlink differences; accept the engine mechanism only with exact oracle parity and >=3% paired wall improvement.

## Notes

Queued immediately after the reopened bounded H65 composition screen. Semantic design remains selected total as a View-axis value, not a fast flag.
