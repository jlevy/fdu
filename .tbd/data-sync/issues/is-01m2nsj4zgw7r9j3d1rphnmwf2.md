---
type: is
id: is-01m2nsj4zgw7r9j3d1rphnmwf2
title: Add an automatic interactive CLI progress indicator
kind: feature
status: in_progress
priority: 2
version: 12
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies: []
child_order_hints:
  - is-01m37nx1sgt52kpj7w1sb2t6z6
  - is-01m37nx2ra4v4f584gv1x3pn9z
  - is-01m37nx3rhfsspr7gv5cgxse63
  - is-01m37nx4m5mph3n4sxn2q6n3fz
  - is-01m37nx5g2ez5hcjrmar93ss7s
  - is-01m37nx6y2x4hvcwj3a4sp6sqk
  - is-01m37nx80t02jdgc65nqdt1tqa
  - is-01m37nx941bb680s8bm713qy6c
  - is-01m37nxa8vshzngyvzw9ax67a0
created_at: 2026-09-16T19:02:11.183Z
updated_at: 2026-09-23T17:44:50.783Z
---
Adopt the reusable progress-indicator baseline being added to tbd rust-cli-rules. For noticeably long one-shot CLI operations, show progress by default only when stderr is an interactive terminal; render only on stderr so report stdout remains composable; suppress automatically for non-TTY, redirected, piped and machine-oriented workflows; provide --no-progress as an unconditional disable; and guarantee cleanup on success, error and interruption. Add deterministic terminal-capability or PTY tests for default-on interactive behavior, non-TTY default-off behavior, --no-progress, stdout separation, machine output without control sequences, and cleanup. Keep this distinct from fdu-m893: that bead emits opt-in reproducible intermediate report frames, while this bead is the ordinary human wait-state indicator. Reconcile design docs that currently say animated progress is never shown, and use the generic tbd guidance rather than copying a project-specific renderer.

## Notes

2026-09-23 design approved with the maintainer and written as docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md: engine Progress handle polled by the CLI; --progress auto|always|never (auto: stderr TTY, TERM!=dumb, human formats only); dust-style Ctrl-C handler (ctrlc, CLI crate only, installed only when drawing) that clears the line, writes 'fdu: interrupted' to stderr, exits 130; fdu-m893 moves off --progress.
