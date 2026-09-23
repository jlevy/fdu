---
type: is
id: is-01m37nx5g2ez5hcjrmar93ss7s
title: Progress ticker, clearing, and wiring into reports and watch
kind: task
status: in_progress
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies:
  - type: blocks
    target: is-01m37nx6y2x4hvcwj3a4sp6sqk
  - type: blocks
    target: is-01m37nx80t02jdgc65nqdt1tqa
  - type: blocks
    target: is-01m37nx941bb680s8bm713qy6c
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:37.633Z
updated_at: 2026-09-23T19:06:34.475Z
---
Ticker thread: 500 ms first-frame delay as a timed receive on the stop channel (stopping returns at once), 80 ms redraws, width re-read per frame via terminal_size_of(stderr) (80 when unknown), stop drawing after the first failed write, never hide the cursor. Stop, join and erase before any stdout/stderr write (report, warnings, errors, performance line), plus an unwind guard. Wire into the one-shot report and the watch's initial scan (stop at first paint). Tests with an injected clock and zero delay: a run under 500 ms writes no progress bytes; the erase precedes the first warning and the error; broken stdout pipe still follows today's rule; a failed stderr write never changes the exit status.
