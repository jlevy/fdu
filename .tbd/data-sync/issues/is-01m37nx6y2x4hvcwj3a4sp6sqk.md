---
type: is
id: is-01m37nx6y2x4hvcwj3a4sp6sqk
title: Ctrl-C handling while the indicator draws
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies:
  - type: blocks
    target: is-01m37nx80t02jdgc65nqdt1tqa
  - type: blocks
    target: is-01m37nx941bb680s8bm713qy6c
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:39.102Z
updated_at: 2026-09-23T17:44:48.762Z
---
Install a handler only on an interactive run that will draw. On Ctrl-C: take the ticker lock, mark stopped, erase the line if a frame shows, write 'fdu: interrupted' to stderr (error style), then take the default action: on Unix restore SIGINT's default disposition and re-raise (shell sees 130, calling scripts stop); on Windows end as the console default does. After the indicator stops, skip the message and take the default action. No new unsafe in fdu: choose signal-hook (safe registration and emulate_default_handler) or ctrlc after confirming Windows behavior; release older than 14 days; command-line crate only; update deny/supply-chain records. Tests: handler order against an injected terminal, no frame after stop.
