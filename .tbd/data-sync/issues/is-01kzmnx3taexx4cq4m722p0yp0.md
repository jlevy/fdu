---
type: is
id: is-01kzmnx3taexx4cq4m722p0yp0
title: CLI UX and zero-install agent skill
kind: epic
status: open
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - cli
dependencies:
  - type: blocks
    target: is-01kzg4bey8nn4k8y1daxc9exhd
  - type: blocks
    target: is-01kzg4bf862ajh8g2tmv5bznng
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01kzmnxar79hx06x01fzfz73yp
  - is-01kzmnxh8812bvnjj5v790jtdy
  - is-01kzmnxr765dahd3cr5j58eews
  - is-01kzmnxy0xvkvazmqvdwsjm20h
created_at: 2026-08-10T01:51:56.233Z
updated_at: 2026-08-10T01:52:33.640Z
---
Implement the approved pre-release CLI surface plan: semantic color and complete help, strict stdout/stderr behavior, stack-safe rendering, one portable agent skill, and an installed-wheel fdu command that delegates to the same Rust process boundary. Publication remains gated by fdu-9cf0.
