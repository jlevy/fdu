---
type: is
id: is-01m3kfjb11y1k4xncszacwnaaa
title: "Docs layer on top of #143: rigorous pass over README and usage docs"
kind: task
status: closed
priority: 1
version: 5
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T07:44:44.573Z
updated_at: 2026-09-28T13:21:21.336Z
closed_at: 2026-09-28T13:21:21.335Z
close_reason: "#144 landed on main 6ec77163 via gh stack merge 148 (stack 141 layers 1-11)"
resolution: null
duplicate_of: null
---
User request 2026-09-28: continue on top of PR #143 (claude/readme-quickstart) and edit/improve the docs, applying tbd common-doc-guidelines rigorously (clarity, accuracy, no duplication, calibrated claims, formatting rules). Branch claude/docs-polish from #143's head; also carries the gitignored-parenthetical fix (fdu-pzis) and regenerated README examples. Check every claim against the current CLI (0.2.0 cache semantics: --cache auto|on|off, --stale-ok).

## Notes

2026-09-28: first pass landed as PR #144 (claude/docs-polish, stack 141 layer 6): Quick Start heading (no 'drop-in' overclaim), bold-colon inline headings in Other Ways to Install, 'actually' filler cut, both quick-start examples regenerated. Speed section text waits on the macOS rerun (fdu-3ivx); version literals on the release layer.
