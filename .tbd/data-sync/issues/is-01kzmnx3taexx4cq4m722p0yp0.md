---
type: is
id: is-01kzmnx3taexx4cq4m722p0yp0
title: CLI UX and zero-install agent skill
kind: epic
status: closed
priority: 1
version: 12
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
  - is-01kzmqww3nn7eafa3awhk6r0zx
  - is-01kzmr90w1mew6jnyq61n7e3s3
  - is-01kzmrp7p8sv40v43bw3y4t5wg
created_at: 2026-08-10T01:51:56.233Z
updated_at: 2026-08-10T02:48:05.440Z
closed_at: 2026-08-10T02:48:05.439Z
close_reason: "The CLI UX, shared process boundary, stack-safe rendering, portable agent skill, installed-wheel console command, cross-platform wheel matrix, golden contract, documentation, and review are complete in PR #2. Local make check and all fresh CI checks pass; public registry publication remains separately gated by fdu-9cf0."
---
Implement the approved pre-release CLI surface plan: semantic color and complete help, strict stdout/stderr behavior, stack-safe rendering, one portable agent skill, and an installed-wheel fdu command that delegates to the same Rust process boundary. Publication remains gated by fdu-9cf0.
