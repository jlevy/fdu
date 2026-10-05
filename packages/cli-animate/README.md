# cli-animate

Record real terminal sessions, replay them in a browser, and render them to video, with
a frame-exact timing check.

## Why It Exists

A demo of a command-line tool is usually asked to do two things at once: look clean and
professional, and tell the truth about what the tool does and how fast it does it.
The existing tools each do part of that, and the gaps between them are where demos go
wrong:

- **Recorders capture the truth but not a script.** asciinema records a real session
  exactly, but someone has to type it live, take after take, and the recording keeps
  every hesitation and typo.
- **Scripted recorders distort time.** VHS scripts the typing, but it films a browser
  terminal on the wall clock while the command runs.
  Measured on fdu in a Linux container, it produced 25 fps when 60 was requested, turned
  14 s of scripted typing and sleeps into a 10.2 s video, and slowed the program being
  filmed from 200 ms to 278 ms.
  A speed demo made that way is not evidence of speed.
- **Renderers disagree.** asciinema-player and agg draw block and shade characters as
  full-cell fills, so bar charts made of `█▓░` merge into one shape.
  The browser replay and the GIF made from the same cast can also look different, since
  they are different renderers with different fonts.
- **Encoding defaults are wrong for terminal video.** A plain ffmpeg conversion from
  screenshots uses BT.601 colour for high-definition video that players read as BT.709,
  shifting saturated terminal colours.
  The x264 tuning that suits terminal content (`-tune animation`) raises the reference
  frames until the stream is tagged with a level many hardware decoders refuse.

cli-animate composes asciinema, xterm.js, Chromium (through Playwright), and ffmpeg into
one pipeline that closes those gaps:

- **Scripted, but real.** A YAML scenario says what to type; the commands then run for
  real in a recorded PTY, and a receipt records each one’s measured wall time.
  Only the typing is simulated, by a keystroke model fitted to fast-typist data and
  seeded so a re-recording reproduces.
  The next prompt appears the instant a command exits, so its duration is visible.
- **Rendered on the recording’s clock.** Video is made by seeking a page frame by frame
  on the cast’s timeline, never by filming it, so nothing is dropped or compressed and
  the recorded program is never slowed.
- **Proved, not assumed.** `verify` checks that every picture change in the lossless
  master lands on the frame of the event that caused it, and `make` refuses to deliver a
  video that fails.
- **One look everywhere.** The browser replay and every video come from the same stage
  page, in the same font (Planetaire Mono, fetched at build time and pinned by hash), so
  what you embed and what you publish are the same pixels.
- **Correct deliveries.** Capture once into a lossless master, then derive a web MP4
  (explicit BT.709, the lowest legal H.264 level, no B-frames) and a README GIF from it.
- **Built for agents as well as people.** One command (`make`) runs the whole pipeline,
  every command has `--json`, `doctor` checks the environment, and `skill` prints an
  agent skill.

If timing does not matter and a decorative GIF is all you need, VHS alone is simpler.

## How It Works

One recording feeds every output.
`record` runs a scripted scenario for real inside `asciinema rec --headless`, typing
each command with the keystroke model, and writes an asciicast with chapter markers plus
a receipt. The stage page (`stage/stage.html`) replays the cast with xterm.js.
Rendering steps that same page frame by frame in headless Chromium and captures it once,
into a lossless master; every delivery is derived from the master, and `verify` checks
the master frame by frame.

The design, and why each piece is the way it is, is in
[the plan](docs/project/specs/active/plan-2026-10-04-cli-animate.md); the survey of
about 45 tools and the measurements behind each choice are in
[the research brief](docs/project/research/research-2026-10-04-terminal-demo-recordings.md).
It is developed as a workspace package of the fdu repository and laid out to be
extracted as its own repository: nothing in it depends on fdu except the example
scenarios in `examples/fdu/`.

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
terminal: { cols: 128, rows: 30, font_size: 18 }   # font_size defaults to 22
typing: { seed: 21 }   # wpm defaults to 220
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
Top-level fields also include `shell`, `setup`, `lead_in`, and `tail`. Set
`terminal.font_size` (CSS pixels, default 22) with `cols` so the widest line fits: a
terminal that wraps a report’s lines hides its columns.
The fdu demos are in `examples/fdu/`, `build-tree.yaml` and `linux.yaml` being the
simplest.

As in a shell, the next prompt is printed the moment a command exits, so how long a
command takes is visible in the recording.
Typing comes in bursts, as people type commands: fast runs within a word and irregular
pauses between words, with an occasional longer think pause.
It averages 220 WPM by default (`src/typing.ts` documents the model); set `typing.wpm`
to change the speed and `typing.seed` to get a different but reproducible take.

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
docs/project/   the plan (specs/active/) and the research brief with its evidence (research/)
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
