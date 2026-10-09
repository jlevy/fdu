---
type: is
id: is-01m4fxattv1jb2w21ae6814ex9
title: "Roots: named-roots request model, validation, overlap refusal"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxaw4tnmt77j099fhnk8j6
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T08:44:02.778Z
updated_at: 2026-10-09T16:17:19.647Z
started_at: 2026-10-09T12:42:14.501Z
closed_at: 2026-10-09T16:17:19.646Z
close_reason: Roots/NamedRoot with labels, validation before any scan with one root's errors, overlap refused by path and (dev,ino) identity (fc407d6e); RootsRequest with one shared request and now, per-root snapshot paths under one cache directory, one snapshot file refused over several roots (bb0dc086)
resolution: null
duplicate_of: null
---
Engine type beside Basis: non-empty ordered roots, each with label = PathBuf::from_iter(path.components()) (label_raw when not UTF-8) and canonical path. Validate existence/directory before any scan (today's errors and exit codes). Refuse equal or nested roots naming both labels: on Unix compare each root's (dev, ino) against every other root's ancestor chain; everywhere also compare canonical paths component-wise. Derive one per-root Request from one spec with today's identity. Cache: the multi-root entry takes a cache directory and derives each root's snapshot with default_cache_path_in; a single explicit snapshot file with several roots is refused.
