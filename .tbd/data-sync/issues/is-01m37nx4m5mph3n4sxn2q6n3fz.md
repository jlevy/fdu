---
type: is
id: is-01m37nx4m5mph3n4sxn2q6n3fz
title: Progress frame renderer
kind: task
status: in_progress
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies:
  - type: blocks
    target: is-01m37nx5g2ez5hcjrmar93ss7s
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:36.740Z
updated_at: 2026-09-23T19:06:33.268Z
---
Pure function from snapshot, elapsed, spinner step, width and color flag to the frame bytes, exactly per the plan's Appearance section: braille dots spinner, phase frames, anstyle palette (cyan spinner and root, bold phase, bright black units/separators/elapsed, green/bright-black analysis bar), human_count/human_bytes, elapsed formats (3.1 s, 1 m 04 s, 1 h 02 m), width measured without color codes with non-ASCII path chars as two columns and the last column empty, and the five-step shrink order. Unit tests pin every phase's text and styled bytes, each width fallback, and the duration formats.

## Notes

2026-09-23: implemented on claude/progress-cli (e2e82008, 75e68e16), merged into claude/progress-indicator (91714c95). Module crates/fdu/src/progress_line.rs; --progress auto|always|never after --color; gating over 48 fact combinations; renderer pinned per phase, width steps, durations. Sizes use human_bytes (38 GiB above 10 units). Awaiting independent review.
