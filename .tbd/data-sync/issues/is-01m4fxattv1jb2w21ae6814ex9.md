---
type: is
id: is-01m4fxattv1jb2w21ae6814ex9
title: "Roots: named-roots request model, validation, overlap refusal"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxaw4tnmt77j099fhnk8j6
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:02.778Z
updated_at: 2026-10-09T09:06:03.190Z
---
Engine type beside Basis: non-empty ordered roots, each with label = PathBuf::from_iter(path.components()) (label_raw when not UTF-8) and canonical path. Validate existence/directory before any scan (today's errors and exit codes). Refuse equal or nested roots naming both labels: on Unix compare each root's (dev, ino) against every other root's ancestor chain; everywhere also compare canonical paths component-wise. Derive one per-root Request from one spec with today's identity. Cache: the multi-root entry takes a cache directory and derives each root's snapshot with default_cache_path_in; a single explicit snapshot file with several roots is refused.
