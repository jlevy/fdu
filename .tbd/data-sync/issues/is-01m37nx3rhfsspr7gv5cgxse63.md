---
type: is
id: is-01m37nx3rhfsspr7gv5cgxse63
title: Interactive detection and the --progress flag
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies:
  - type: blocks
    target: is-01m37nx5g2ez5hcjrmar93ss7s
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:35.855Z
updated_at: 2026-09-24T08:20:17.639Z
closed_at: 2026-09-24T08:20:17.638Z
close_reason: "Shipped in PR #120 (merged to main as 1d2da61a, 2026-09-24): make check, cross-lint, and CI green on tree 121b0cba."
resolution: null
duplicate_of: null
---
Read stderr is-terminal, TERM, CI and (Windows) the result of anstyle-query's enable_ansi_colors once in main and pass them in (extend the terminal context as urollup does). Add --progress auto|always|never (default auto) with help text that says always never draws into a pipe or file. Gating: non-interactive never draws under any flag; interactive auto draws for text/tree/paths/long only; always for every format; never for none; --docs/--skill/--cache-status/--cache-clear never draw. Tests with injected facts over every combination, asserting exact stderr bytes (nothing for non-interactive).

## Notes

2026-09-23: implemented on claude/progress-cli (e2e82008, 75e68e16), merged into claude/progress-indicator (91714c95). Module crates/fdu/src/progress_line.rs; --progress auto|always|never after --color; gating over 48 fact combinations; renderer pinned per phase, width steps, durations. Sizes use human_bytes (38 GiB above 10 units). Awaiting independent review.
