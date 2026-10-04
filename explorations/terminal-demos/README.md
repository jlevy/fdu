# Terminal Demo Spike

A spike for the
[terminal demo recordings research](../../docs/project/research/research-2026-10-04-terminal-demo-recordings.md):
run scripted fdu sessions for real, keep them as asciicast recordings, replay them in a
browser, and render them to video from the same recording.
It is not part of the build, the gate, or any release.

## Pipeline

1. `record.py` reads a scenario (TOML) and runs it inside `asciinema rec --headless`.
   Each command is typed at a seeded human cadence, then executed in the recording’s
   PTY, so the cast holds the program’s real output at its real speed.
   It writes `<name>.cast` (asciicast v3, with a chapter marker per step) and
   `<name>.receipt.json` (each command’s measured wall time and where it starts in the
   cast).
2. `web/stage.html` plays a cast with xterm.js (default) or asciinema-player.
   Opened in a browser it is a player; in `mode=capture` it is a frame source.
3. `render.mjs` loads the stage in headless Chromium, seeks it to each frame’s time on
   the cast’s own clock, screenshots it, and pipes the frames to ffmpeg.
   It writes the video and `<name>.video.json`. Frames with no terminal output in their
   interval reuse the previous screenshot.
4. `verify_timing.py` checks a lossless render (`--codec lossless`) frame by frame: the
   picture must change exactly on the first frame at or after each output event.

## Run

Requires asciinema 3.x, ffmpeg with libx264, Node 20 or newer, Python 3.11 or newer, a
release build of fdu, and a Chromium that playwright-core can launch (set `CHROME_PATH`
to use a specific one; playwright-core never downloads a browser).

```shell
cargo build --release -p fdu
cd explorations/terminal-demos
npm ci
python3 record.py scenarios/realtime.toml out/realtime.cast
node render.mjs out/realtime.cast out/realtime.mp4 --fps 60
node render.mjs out/realtime.cast out/realtime-check.mkv --codec lossless
python3 verify_timing.py out/realtime.cast out/realtime-check.mkv
```

`FDU_DEMO_BIN` selects another fdu build directory.
`scenarios/views.toml` reads a shallow CPython clone under `FDU_DEMO_TREES`; its header
gives the clone command.
`render.mjs` also takes `--scale 2` (a 3840×2160 render of the same 1920×1080 stage),
`--codec vp9` (WebM), and `--engine asciinema`.

Dependencies are pinned in `package-lock.json` and were resolved with
`npm install --before` a date 14 days before the spike, which keeps the repository’s
supply-chain cool-off.
`comparisons/realtime.tape` is the VHS version of the real-time scenario, kept for the
comparison in the research.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
