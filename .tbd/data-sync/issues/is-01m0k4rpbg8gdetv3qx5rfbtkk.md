---
type: is
id: is-01m0k4rpbg8gdetv3qx5rfbtkk
title: Add largest and recent as documented aliases over files
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:48:52.975Z
updated_at: 2026-09-16T16:50:15.112Z
closed_at: 2026-09-16T16:50:15.111Z
close_reason: "a6b670c (PR #39): largest and recent are ViewSpec presets (size and mtime order, 20 rows, regular files only), overridable by --sort and --limit, and --docs states the equivalence. Resolved in fdu-core, not the CLI layer, and --kind cannot widen them to directories."
resolution: null
duplicate_of: null
---
Add `largest` and `recent` as named presets over the files machinery:

  largest ≡ files --sort size  --limit 20 --kind file
  recent  ≡ files --sort mtime --limit 20 --kind file

Directories are excluded because `tree` already reports directory sizes; a `largest` that
lists directories duplicates it at a coarser grain and pushes the actual files out of the
window.

Implement them as defaults resolved at the CLI layer, so `ViewSpec` gains two values that
project through the same code path and any explicit `--sort`, `--limit`, or `--kind`
overrides them. They are aliases, and the documentation should say exactly that -- one
line each, with the equivalence spelled out, so a reader learns the composition rather
than memorising two more views.

## Notes

recent bounds by count alone; a time window is --modified-since, a different question that composes.
