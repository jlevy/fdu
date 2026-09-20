---
type: is
id: is-01m2ys9y5ep69vq65z0xs0ax19
title: "H147: keep transient batch recycle at the 3% bar"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels:
  - linux
  - campaign-2
dependencies: []
parent_id: is-01m2ymtwf6fth3a3rk0nn4kw8d
created_at: 2026-09-20T06:50:51.950Z
updated_at: 2026-09-20T06:54:07.923Z
closed_at: 2026-09-20T06:54:07.923Z
close_reason: "H147 accepted (exp-151): quiet linux-v6.12 --no-controls aggregate -4.98% [-5.92%, -4.33%]; RSS flat; default gitignore-on placebo includes zero. Engine kept 5c6e6394."
---
H85 missed its 20% mimalloc bar (exp-150). The same recycle cleared 3% on quiet reconstructible linux-v6.12 --no-controls aggregate (-4.98% [-5.92%, -4.33%]); RSS flat; default gitignore-on placebo includes zero. Keep the engine as H147. Do not lower H85. Unmeasured on macOS.
