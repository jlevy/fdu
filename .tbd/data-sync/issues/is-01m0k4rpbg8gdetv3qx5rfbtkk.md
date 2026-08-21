---
type: is
id: is-01m0k4rpbg8gdetv3qx5rfbtkk
title: Add largest and recent as documented aliases over files
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:48:52.975Z
updated_at: 2026-08-21T21:50:33.496Z
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
