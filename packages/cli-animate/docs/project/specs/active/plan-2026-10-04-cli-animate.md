# Feature: cli-animate, Terminal Recordings for the Web and Video

**Date:** 2026-10-04 (last updated 2026-10-05)

**Author:** Claude (agent), for the fdu maintainer

**Status:** Phase 1 implemented; follow-ups open (see [Follow-Ups](#follow-ups))

## Overview

`cli-animate` is a small, self-contained TypeScript package with one command line, built
for people and agents.
It runs a scripted terminal session for real, keeps it as an asciicast recording,
replays the recording in a browser, and renders the same recording to video, with a
frame-exact timing check.
It replaces the Python spike in `explorations/terminal-demos/`, which proved the
approach but grew as loose scripts.
It lives at `packages/cli-animate/` so it can later be extracted as its own package and
installed as an agent skill.

## Goals

- One entry point, `cli-animate`, with subcommands for every step and one for the whole
  pipeline. No other script produces a web or video output.
- A declarative, validated scenario file (YAML) describing what to type, what to run,
  and the terminal’s size and look.
- Real execution and real timing: commands run in a recorded PTY, and a receipt records
  each command’s measured wall time beside the cast.
- A fast, human-like typing model grounded in typing research, seeded so re-recordings
  reproduce.
- One render stage for both outputs: the web replay and the video are the same pixels,
  cropped to the terminal window, in Planetaire Mono fetched at build time and verified
  by hash.
- Named, documented encoding profiles with correct colour handling.
- Unit tests in `make check`; an opt-in end-to-end test for the parts that need
  asciinema, Chromium, and ffmpeg.
- One CLI-backed agent skill (rung L1 of `cli-agent-skill-patterns`) that routes to the
  CLI’s own help.

## Non-Goals

- Capturing GUI or browser sessions; only terminals.
- Editing, voice-over, music, or titles beyond the window title.
- Answering terminal queries (cursor position, colour reports) during recording,
  auto-zoom, captions, keystroke overlays, dead-time compression, and output-driven
  waits. These are follow-ups (see [Follow-Ups](#follow-ups)), each tracked as a bead.
- Publishing the package to npm.
  It is a private workspace package until extraction.

## Background

The
[terminal demo recordings research](../../research/research-2026-10-04-terminal-demo-recordings.md)
surveyed the tooling and measured the alternatives.
Its conclusions drive this design:

- Record headless with asciinema 3 and render later.
  Real-time screen capture (VHS) distorted timing and slowed the program being filmed.
- Render by seeking a page on the recording’s clock and screenshotting, as
  `jlevy/squares` does; a mechanical check then proves frame alignment.
- Render with xterm.js: asciinema-player and agg merge fdu’s block-element bars.

The deeper research pass (same brief, “Further research”) adds the colour-tagging,
B-frame, font-readiness, and terminal-query findings used below.

## Design

### Approach

A TypeScript ESM package, Node 22.12 or newer, built with `tsc`, following
`typescript-cli-tool-rules`: Commander for commands, picocolors behind a `--color`
option, `--json` for machine output, and errors that name what failed and how to fix it.
It is an npm workspace of the repository root, so its dependencies sit in the root
lockfile and pass the existing supply-chain check, cool-off, and `npm audit`.

External programs are required, not bundled: `asciinema` 3.x (recording), `ffmpeg` with
libx264 (encoding), and a Chromium that playwright-core can launch (rendering).
`cli-animate doctor` reports which are present and their versions.

### Components

```text
packages/cli-animate/
  package.json          bin: cli-animate; workspace of the root
  LICENSE               MIT, as the repository
  docs/project/         this plan and the research brief, with the package
  src/
    cli/main.ts         the program: global options, command groups, exit codes
    cli/commands/       one module per command
    cli/lib/            output (text or JSON, colour, streams) and command context
    scenario.ts         YAML scenario schema (zod) and loader
    typing.ts           keystroke-timing model
    driver.ts           the in-PTY half: prompt, typing, command execution, markers
    record.ts           runs asciinema around the driver; rewrites markers; receipt
    cast.ts             asciicast v2/v3 reading and writing
    fonts.ts            pinned font manifest, fetch, hash check, stylesheet
    server.ts           local server for the stage page
    render.ts           Playwright frame stepping, master capture, delivery, receipts
    profiles.ts         encoding profiles and H.264 level selection
    verify.ts           frame-exact timing check
    tools.ts            external program discovery
    fsutil.ts           atomic file writes
  stage/stage.html      the replay page (play and capture modes)
  skill/SKILL.md        the agent skill
  examples/fdu/*.yaml   fdu demo scenarios
  tests/                unit tests per module, CLI contract tests, opt-in end-to-end test
```

Generated files go to an output directory (default `out/`, gitignored) or the fonts
cache (`stage/fonts/`, gitignored).
Nothing generated is committed.

### Commands

| Command | Does |
| --- | --- |
| `cli-animate record <scenario> [-o cast]` | Run the scenario for real; write the cast and its receipt |
| `cli-animate render <cast> [-o video] [--profile p] [--master path]` | Capture a cast once and deliver one profile |
| `cli-animate deliver <master> -o video [--profile p]` | Derive another delivery from a kept master |
| `cli-animate verify <cast> <master>` | Check the lossless master frame-exactly against the cast |
| `cli-animate make <scenario> [--out dir] [--gif]` | Fonts, record, capture, verify, deliver, in order |
| `cli-animate serve <cast>` | Serve the replay page for a cast on localhost |
| `cli-animate fonts` | Fetch and verify the pinned fonts |
| `cli-animate doctor` | Report required external programs and versions |
| `cli-animate skill` | Print the agent skill |

Global options: `--json`, `--quiet`, `--verbose`, `--color <when>`. Results go to
stdout, progress and errors to stderr (errors as `{"error": ...}` with `--json`). Exit
codes: 0 success, 1 a failed check (a verify mismatch, which also stops `make` before it
delivers anything; a `doctor` miss), 2 a usage or environment error.
A reader closing stdout early is not an error.

### Scenario File

YAML, validated on load with errors that name the field:

```yaml
title: "fdu: a real machine, real time"
terminal: { cols: 128, rows: 30 }
typing: { seed: 21 }   # wpm defaults to 160
cwd: $FDU_DEMO_TREES
path: [../../../../target/release]   # relative to this file
env: { TIMEFORMAT: "real %3Rs" }
steps:
  - comment: "# The whole root filesystem, page cache warm."
  - run: time du -sxh /
    hold: 1.4
  - run: fdu cpython --analyze code --view languages --limit 6
    before: [fdu cpython --cache-clear]
    clear: true
```

Step fields: `run` or `comment`, `label`, `clear`, `before` (hidden commands), `hold`,
`stop_after` (seconds, then SIGINT), and `during` (background actions at offsets).

### Recording

`record` starts `asciinema rec --headless --window-size CxR` with the driver as its
command. The driver is a hidden subcommand of the same CLI, so both halves share the
schema and typing model.
As in a shell, it prints the next prompt the moment a command exits, then holds, so the
recording shows how long each command took.
At each step boundary it writes an OSC marker that no terminal acts on; afterwards
`record` turns markers into asciicast `m` events and strips them.
The receipt records each command’s wall time, exit status, and offset in the cast, plus
tool versions.

### Typing Model

The inter-key interval is a base (from the target words per minute, 160 by default,
calibrated on a reference text) times a digraph factor times mean-preserving log-normal
noise and a per-token factor, plus occasional stalls and word-initial hesitations.
Enter follows after a log-normal pause with a 250 ms median.
Digraph factors follow skilled-typist studies: hand alternation fastest, same hand
slower, same finger slowest, repeated keys slower for experts, frequent English digraphs
faster, first letter of a word slower, Shift and number-row reaches costlier.
Each character consumes a fixed number of random draws so a seed reproduces a take.
Values and sources are in the research brief.

### Stage and Fonts

`stage.html` loads the cast, renders it with xterm.js in a window frame, and either
plays it (web) or exposes a seek contract (capture).
In capture mode it shrinks to the window plus a margin and reports its size rounded to
even pixels. Fonts come from a manifest pinning Planetaire Mono Text v0.2.0 WOFF2 files
by URL and SHA-256; `fonts` downloads, verifies, and writes `fonts.css`. The stage
refuses to start a capture until `document.fonts.check` confirms every face.

### Capture Once, Deliver Many

Rendering captures once, at 2× by default, into a lossless RGB master; `verify` reads
the master, and every delivery is derived from it by ffmpeg, so what is verified is what
is delivered.

| Profile | Use | Settings |
| --- | --- | --- |
| `master` | `verify`, archive, deriving the others | libx264rgb CRF 0, sRGB tagged |
| `web` (default) | Sharing, docs, social | H.264 High 4:2:0, CRF 16, `-tune animation`, level pinned to the lowest the size and rate allow, no B-frames, explicit BT.709 conversion and tags, `+faststart` |
| `gif` | READMEs | 25 fps, 256-colour palette without dithering, 1× width |

The level is pinned because `-tune animation` raises the reference frames and x264 then
tags levels many hardware decoders refuse.
The conversion is explicit because ffmpeg converts RGB with BT.601 by default while
players read untagged high-definition video as BT.709.

## Implementation Plan

### Phase 1: Package, CLI, and Skill

- [x] Workspace package skeleton: package.json (bin, engines, pinned deps within the
  cool-off), tsconfig, build, root workspace entry, `.gitignore`
- [x] Scenario schema and loader, with error messages naming the field
- [x] Typing model with unit tests (ordering, calibration, seeding)
- [x] Cast reading/writing and marker rewriting with unit tests
- [x] Driver and `record`, including receipts
- [x] Fonts manifest and `fonts`
- [x] Stage page (xterm.js only), capture contract, crop, font readiness
- [x] `render` with encoding profiles and colour handling
- [x] `verify`, `make`, `serve`, `doctor`, `skill`
- [x] Agent skill and package README
- [x] Port the fdu scenarios to YAML; remove `explorations/terminal-demos`
- [x] `make cli-animate-check` (typecheck and unit tests) in `make check`; opt-in
  `make cli-animate-e2e`
- [x] Re-record the showcase, verify, and publish the video for review
- [x] Update the research brief and the pull request
- [x] Bump tryscript to 0.3.0 and remove the temporary fast-glob stand-in
- [x] Move the plan and research brief into the package (`docs/project/`)
- [x] Prompt on command exit; 160 WPM default; CLI split per `typescript-cli-tool-rules`
  (commands/, lib/, command groups, exit codes, JSON errors, stdout EPIPE)

## Testing Strategy

- Unit tests (`node --test`, no external programs): scenario validation, typing model
  properties, cast parsing and marker rewriting, frame mapping, profile arguments.
- End-to-end test (opt-in; needs asciinema, Chromium, ffmpeg): record a two-step
  scenario of `printf` commands, render lossless, and require `verify` to pass.
- Manual review of the published video.

## Rollout Plan

A private workspace package in this repository, laid out as a standalone repository
would be: its own README, licence, docs (`docs/project/`), tests, and skill.
Extraction to its own repository and npm package is a later decision.
Nothing here depends on fdu except the example scenarios in `examples/fdu/`; the
research brief links to fdu’s own guides by URL so the links survive extraction.

## Follow-Ups

Each is a bead under the cli-animate epic:

- Answer terminal queries during recording (own the PTY with a terminal model).
- Captions and chapter titles from cast markers; optional keystroke overlay.
- Dead-time compression for long still stretches, as a render option.
- Output-driven waits (`wait_for` text or settle) instead of fixed holds.
- Capture via `HeadlessExperimental.beginFrame` where available; feed reused frames
  without re-encoding PNGs.
- Poster frames and an animated WebP option (the GIF profile shipped).
- Seam-free block glyphs: Planetaire’s full block stops just short of its advance, so
  browsers show faint seams between cells; fix in the font, or capture with xterm.js’s
  WebGL renderer (which draws block elements itself) once its headless screenshots work.
- Pin H.264 levels in a conformance check on the delivered file (ffprobe), as `squares`
  does, and record per-frame hashes in the receipt.

## Open Questions

- Extraction target: its own repository, or a package under an existing tools repo?

## References

- [Terminal demo recordings research](../../research/research-2026-10-04-terminal-demo-recordings.md)
- `tbd guidelines typescript-cli-tool-rules`, `cli-agent-skill-patterns`,
  `typescript-rules`, `general-testing-rules`, `supply-chain-hardening`
- Epic `fdu-7sb5`; pull request jlevy/fdu#171

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
