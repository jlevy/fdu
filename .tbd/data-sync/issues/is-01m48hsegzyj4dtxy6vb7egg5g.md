---
type: is
id: is-01m48hsegzyj4dtxy6vb7egg5g
title: Re-time the macOS performance index on a quiet host at 20 rounds before quoting it
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T12:07:37.751Z
updated_at: 2026-10-06T12:07:37.751Z
---
The Phase 1 macOS index cells (PR #176) ran on an uncontrolled host (CPU above the 25% quiet gate at 48-99% of sample boundaries) and 9 of 11 at 12 rounds where the spec asks 20 under an hour (review E1, E4 on #176). The score is labelled exploratory. Before the score appears in release notes or the README, re-time every component cell quiet or controlled-interactive at 20 rounds with the in-repo driver, and replace the cells.
