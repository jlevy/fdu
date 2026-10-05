# cli-animate

Record real terminal sessions, replay them in a browser, and render them to video, with
a frame-exact timing check.

One recording feeds every output.
`record` runs a scripted scenario for real inside `asciinema rec --headless`, typing
each command with a fast-human keystroke model, and writes an asciicast with chapter
markers plus a receipt of each command’s measured wall time.
The stage page (`stage/stage.html`) replays the cast with xterm.js in Planetaire Mono.
Rendering steps that same page frame by frame in headless Chromium on the recording’s
own clock and captures it once, into a lossless master; every delivery (a web MP4, a
README GIF) is derived from the master, so the web replay and the videos are the same
pixels and keep the commands’ real timing.
`verify` proves it on the master, frame by frame.

The design, and why each piece is the way it is, is in
[the plan](../../docs/project/specs/active/plan-2026-10-04-cli-animate.md) and
[the research brief](../../docs/project/research/research-2026-10-04-terminal-demo-recordings.md).
It is a workspace package of the fdu repository, written to be extracted later; nothing
in it depends on fdu.

## Requirements

Node 22.12 or newer, asciinema 3.x, ffmpeg with libx264, and a Chromium that
playwright-core can launch (`CHROME_PATH` or `--chrome`). `cli-animate doctor` checks
them all.

## Use

From the repository root (`npm ci` installs the workspace):

```sh
npm run build --workspace cli-animate
alias cli-animate="node packages/cli-animate/dist/src/cli/main.js"

cli-animate doctor
cli-animate fonts
cli-animate make packages/cli-animate/examples/fdu/showcase.yaml --out out/
cli-animate serve out/showcase.cast
```

| Command | Does |
| --- | --- |
| `record <scenario> [-o cast]` | Run the scenario for real; write the cast and `*.receipt.json` |
| `render <cast> [-o video] [--profile web\|gif\|master] [--master path]` | Capture the cast and deliver one profile; write `<video>.json` |
| `deliver <master> -o video [--profile web\|gif]` | Derive another delivery from a kept master, without capturing again |
| `verify <cast> <master>` | Check the lossless master frame-exactly against the cast |
| `make <scenario> [--out dir] [--gif]` | Fonts, record, capture, verify, and deliver in one step |
| `serve <cast>` | Serve the replay page on localhost |
| `fonts` | Fetch Planetaire Mono Text v0.2.0, verified by SHA-256 |
| `doctor` | Report required programs and assets |
| `skill` | Print the agent skill |

Every command takes `--json`, `--quiet`, and `--color <when>`.

## Scenarios

A scenario is YAML; unknown keys and bad values are errors that name the field.
A relative `cwd` and relative `path` entries resolve against the scenario file’s
directory, and both expand environment variables (an unset one is an error).
`COLUMNS` and `LINES` are set from `terminal`.

```yaml
title: "fdu: a real machine, real time"
terminal: { cols: 128, rows: 30 }
typing: { seed: 21 }   # wpm defaults to 160
path: [../../../../target/release]
cwd: $FDU_DEMO_TREES
env:
  TIMEFORMAT: "\e[2mreal %3Rs\e[0m"
steps:
  - comment: "# Nothing is sped up."
  - label: du
    run: time du -sxh /
    hold: 1.4
  - run: fdu cpython --analyze code --view languages --limit 6
    before: [fdu cpython --cache-clear]   # hidden, not recorded
    clear: true
```

Step fields: `run` or `comment`, `label`, `clear`, `before`, `hold` (seconds to wait
after the next prompt appears), `stop_after` (seconds, then SIGINT, for long-running
commands), and `during` (`{ at, run }` background actions while the command runs).
Top-level fields also include `shell`, `setup`, `lead_in`, and `tail`. The fdu demos are
in `examples/fdu/`.

As in a shell, the next prompt is printed the moment a command exits, so how long a
command takes is visible in the recording.
Typing defaults to 160 WPM, shaped by the keystroke model in `src/typing.ts`; set
`typing.wpm` to change it and `typing.seed` to get a different but reproducible take.

## Encoding Profiles

Capture is at 2× by default (sharp glyph edges after chroma subsampling).

| Profile | For | Settings |
| --- | --- | --- |
| `master` | `verify`, archive, deriving the others | Lossless RGB H.264 (libx264rgb CRF 0), sRGB tagged |
| `web` (default) | Sharing anywhere | H.264 High 4:2:0, CRF 16, level pinned to the lowest the size and rate allow, no B-frames, explicit BT.709 conversion and tags |
| `gif` | READMEs | 25 fps, 256 colours without dithering, 1× width |

The web profile’s colour handling matters: ffmpeg converts RGB with BT.601 by default,
while players read untagged high-definition video as BT.709, which shifts saturated
terminal colours.

## Layout

```text
src/            engine: scenario, typing, driver, record, cast, fonts, render, profiles, verify
src/cli/        the command line: main.ts, one module per command in commands/, shared helpers in lib/
stage/          the replay page (play and capture modes)
skill/          the agent skill
examples/fdu/   fdu demo scenarios
tests/          unit tests per module, CLI contract tests, and the opt-in end-to-end test
```

The engine reports through return values and callbacks; the command line owns all
output. The exceptions are deliberate: `driver.ts` is the recorded session itself, and
`record` mirrors that session to stderr unless asked to be quiet.

## Develop

```sh
npm run test --workspace cli-animate   # clean build, typecheck, unit and CLI tests (in `make check`)
npm run e2e --workspace cli-animate    # end to end through asciinema, Chromium, ffmpeg (`make cli-animate-e2e`)
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
