---
type: is
id: is-01kzmnxar79hx06x01fzfz73yp
title: Harden CLI color, help, and stream behavior
kind: feature
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - cli
dependencies:
  - type: blocks
    target: is-01kzmnxh8812bvnjj5v790jtdy
  - type: blocks
    target: is-01kzmnxr765dahd3cr5j58eews
  - type: blocks
    target: is-01kzmnxy0xvkvazmqvdwsjm20h
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T01:52:03.334Z
updated_at: 2026-08-10T02:09:05.992Z
closed_at: 2026-08-10T02:09:05.991Z
close_reason: Implemented the shared Rust process boundary, semantic anstyle palette, --color contract with NO_COLOR/FORCE_COLOR precedence, complete help, strict stdout/stderr diagnostics, pluralization, and real-process regression coverage. Exact goldens and make check pass.
---
Use red-green tests to replace the overlapping --no-color surface with --color=auto|always|never, implement documented CLI/NO_COLOR/FORCE_COLOR precedence, share semantic anstyle definitions with clap help and human output, move human warnings to stderr, correct singular/plural labels, preserve broken-pipe and exit behavior, and make help the complete human/agent invocation contract. JSON and skill output remain ANSI-free.
