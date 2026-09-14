---
type: is
id: is-01m2h2wcesdhz6v7g1nz507qc3
title: "PR #58 review PR58-REC-1: exp-104 asserts a Windows roll-up lookup bug the shipped PathBuf map cannot have"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m2h2t7k65srszyhv02tag8ba
created_at: 2026-09-14T23:08:51.540Z
updated_at: 2026-09-14T23:27:39.603Z
closed_at: 2026-09-14T23:27:39.602Z
close_reason: "bd8ed0b: exp-104 no longer claims a Windows rollup() defect; What was tried says only the byte-keyed candidate needed normalization, since Path Hash and Eq are component-wise. fdu-cfpa closed as not a defect."
resolution: null
duplicate_of: null
---
PR #58, P2. `docs/project/experiments/exp-104-hash-the-content-roll-up-map-by-path-bytes-instead-of-compon.md`, "Why it fails" paragraph 2 (at 3a67552), says `rollup()` silently misses roll-ups on Windows.

`crates/fdu-core/src/content/content_index.rs:171` keys `rollups` by `PathBuf`, and `:201-202` looks up with `&Path`. The standard library's `Hash for Path` skips separator bytes (`is_sep_byte` accepts both `/` and `\` on Windows), and `PartialEq for Path` compares `components()`, as `content_index.rs:107-110` says. Only the byte-keyed H103 candidate needed `rollup()` to normalize.

Fix: correct the paragraph and the PR body; close `fdu-cfpa`, which files the non-bug.
