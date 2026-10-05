---
name: cli-animate
description: >-
  Record real terminal sessions and turn them into browser replays and verified videos.
  Use when asked to make a terminal demo, screencast, GIF-like clip, or video of a CLI
  running, to show real command speed, or to embed a terminal recording on a web page.
---
# cli-animate

`cli-animate` runs a scripted terminal session for real (commands execute in a PTY under
asciinema; only the typing is simulated, with a fast-human keystroke model), keeps it as
an asciicast, and renders that one recording to a web replay and to video.
Timing in the output is the commands’ real timing, and a lossless render can be checked
frame by frame.

## Before You Start

1. Run `cli-animate doctor`. It must report asciinema 3.x, ffmpeg with libx264, a
   Chromium (`CHROME_PATH`), and the fonts.
   Run `cli-animate fonts` if fonts are missing.
2. Find or write a scenario YAML. Read `cli-animate record --help` and an example such
   as `examples/fdu/showcase.yaml` for the fields.
   Every command in it really runs, in the scenario’s `cwd`, with the user’s
   permissions: read each one before recording.

## Workflow

```sh
cli-animate make path/to/scenario.yaml --out out/ --gif   # record, capture, verify, deliver
cli-animate serve out/scenario.cast                       # watch the replay in a browser
```

Or step by step: `record` → `render --master out/x.master.mp4` (captures once, delivers
the web MP4) → `verify <cast> <master>` → `deliver <master> --profile gif` for more
formats. Add `--json` to any command for machine-readable results.

## Done When

- `make` (or `verify` on the master) reports every change frame on its event’s frame,
  exit status 0.
- The receipt beside the cast (`*.receipt.json`) lists each step with exit code 0 unless
  the scenario meant otherwise, and plausible wall times.
- You have watched the video or replay before sharing it.

## Boundaries

- Speed is evidence only for the machine and conditions recorded.
  Record on quiet hardware with a warm cache unless the demo is about cold behavior, and
  say which.
- Programs that query the terminal (cursor position, colour reports) are not answered
  while recording; full-screen TUIs may misbehave.
- Never put secrets in a scenario or let a recorded command print them: the cast keeps
  every byte of output.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
