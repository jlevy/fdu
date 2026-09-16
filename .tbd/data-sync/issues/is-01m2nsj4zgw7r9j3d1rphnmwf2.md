---
type: is
id: is-01m2nsj4zgw7r9j3d1rphnmwf2
title: Add an automatic interactive CLI progress indicator
kind: feature
status: open
priority: 2
version: 1
labels:
  - cli
  - ux
dependencies: []
created_at: 2026-09-16T19:02:11.183Z
updated_at: 2026-09-16T19:02:11.183Z
---
Adopt the reusable progress-indicator baseline being added to tbd rust-cli-rules. For noticeably long one-shot CLI operations, show progress by default only when stderr is an interactive terminal; render only on stderr so report stdout remains composable; suppress automatically for non-TTY, redirected, piped and machine-oriented workflows; provide --no-progress as an unconditional disable; and guarantee cleanup on success, error and interruption. Add deterministic terminal-capability or PTY tests for default-on interactive behavior, non-TTY default-off behavior, --no-progress, stdout separation, machine output without control sequences, and cleanup. Keep this distinct from fdu-m893: that bead emits opt-in reproducible intermediate report frames, while this bead is the ordinary human wait-state indicator. Reconcile design docs that currently say animated progress is never shown, and use the generic tbd guidance rather than copying a project-specific renderer.
