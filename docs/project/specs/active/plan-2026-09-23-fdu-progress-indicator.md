# Plan: An Interactive Progress Indicator

**Date:** 2026-09-23

**Status:** Approved design, implemented; the real-terminal smoke test (`fdu-3xiz`) and
the documentation (`fdu-n6bd`) are outstanding

**Tracking:** `fdu-vngp`

## Overview

A long `fdu PATH` on a large tree prints nothing until the report is ready, so a person
cannot tell a slow scan from a hung one.
This plan adds a one-line progress indicator on stderr while an interactive human run is
working. It is on by default only when a person is watching, off everywhere else, and
controlled by `--progress auto|always|never`.

The engine exposes the progress facts; the command line only decides whether and how to
draw them. That is the
[command line invents nothing](../../architecture/fdu-design-principles.md) rule applied
to a wait-state display.

## Goals

- Show directories, files, and bytes walked, the current phase, and elapsed time during
  a one-shot report, and analyzed files with a percentage during content analysis, in
  the same layout for every phase.
- Draw only for a person at an interactive terminal.
  Every non-interactive run draws nothing, whatever the flag says.
- Wait 500 ms before the first frame, so a fast run shows no indicator at all.
- Never interleave with output: the line is cleared before any stdout or stderr write,
  on success, on error, on panic, and on Ctrl-C.
- Cost nothing measurable when off, and stay within noise when on.
- Leave every golden, parity recording, and machine document unchanged.

## Non-Goals

- A progress callback in the Python package.
  The engine handle is designed so one can follow; it is not part of this plan.
- Streaming intermediate report frames.
  That is `fdu-m893`, which is renamed off `--progress` by this plan.
- A current-path display, an estimated time remaining, or a percentage for metadata
  walks, which have no trustworthy denominator.
- A full-screen or multi-line display; an interactive TUI stays a non-goal.

## Background

The help guide says the command “never prompts, pages, or animates progress”, and today
the engine gives the command line nothing to observe while a report runs:
`prepare_report` takes only the request and delivery, the cold scan has no sink, warm
reconciliation discards its commits and emits almost none for an unchanged tree, and the
`FDU_COUNTERS` instrumentation is off by default and folds per-thread counts only when
workers finish. The only live progress type, `DiscoveryProgress`, belongs to opened
roots, which a report does not use.

Two local tools informed the design.
dust (`attic/dust`, v1.2.4) runs a ticker thread over relaxed atomics, delays its first
frame by 100 ms, and installs a Ctrl-C handler that moves off the line and exits.
It also never checks for a terminal, writes “Aborting” to stdout, clears by byte count
so a wrapped line leaves residue, panics when stderr closes, and updates shared counters
per file even with progress disabled.
urollup gates on stderr being a terminal, suppresses machine formats, ignores write
errors on the progress line, and tests the gating with an injected terminal context and
exact stderr bytes. It shows only a static message and leaves the line on interruption.

The tbd guidelines (`rust-cli-rules`, `python-cli-patterns`,
`typescript-cli-tool-rules`, `error-handling-rules`, `general-testing-rules`) require
progress on stderr, suppressed when that stream is not a terminal or `CI` is set, exit
status 130 for an interruption, ignored write errors on diagnostic streams, and injected
terminal facts and clocks in tests.
urollup’s research and design (its Rust CLI engineering baseline, decisions 29 and 30,
and its manual terminal QA) add the machine-format rule and the rule that `NO_COLOR`
affects color only. None of these sources detects agents from environment variables for
presentation, and this plan does not either: an agent shell observed in this repository
runs with `TERM=dumb` and no terminal, which the interactive rule below already
excludes.

## Design

### Decisions

Recorded with the maintainer on 2026-09-23:

- The flag is `--progress auto|always|never`, default `auto`, mirroring `--color`.
- Non-interactive use always disables the indicator.
  No flag can draw it into a pipe, a file, a log, or CI.
- `auto` also suppresses the indicator for machine formats (`json`, `jsonl`, `yaml`) on
  an interactive terminal, because some agent harnesses run commands under a
  pseudo-terminal and a redrawn line would pollute their captured stderr.
  `--progress always` lifts only this rule.
- A run that finishes within 500 ms never shows the indicator.
- The indicator is a colored animated-dots spinner in fdu’s existing palette, specified
  exactly under [Appearance](#appearance).
- Ctrl-C follows dust’s model of a handler that moves the terminal off the progress
  line, corrected where dust is wrong: the message goes to stderr, not stdout, and fdu
  then dies by the default interrupt action instead of exiting normally.
  An exit that merely reports status 130 makes a calling shell script continue, because
  shells stop a script only when the child was killed by the signal.
- The indicator takes `--progress`; `fdu-m893`’s report frames move to another name.

### Engine: a Polled Progress Handle

`fdu-core` gains a public `Progress` handle: cheap to clone, `Send + Sync`, created by
the caller and passed to the routes that do long work.

```rust
pub struct Progress { /* Arc<ProgressCells> */ }

impl Progress {
    pub fn new() -> Self;
    /// A consistent-enough view for display: each counter is monotonic, and the phase
    /// is the one most recently entered.
    pub fn snapshot(&self) -> ProgressSnapshot;
}

pub struct ProgressSnapshot {
    pub phase: ProgressPhase,
    pub directories: u64,
    pub files: u64,
    pub bytes: u64,
    /// Content files analyzed and the candidates known when analysis began.
    pub analysis: Option<(u64, u64)>,
}

pub enum ProgressPhase { Starting, Loading, Scanning, Revalidating, Analyzing, Saving }
```

A fresh handle reports `Starting` until the route enters its first phase, and the last
phase entered persists after the route returns.

It enters through variants beside today’s entry points rather than through `Delivery`,
which stays a plain comparable value:
`prepare_report_with_progress(request, delivery, &progress)`, following
`prepare_report_with_scan_diagnostics`, and `Session::start_with_progress` for the
watch’s initial scan.
Inside the engine the handle rides on `ScanConfig::progress`, an observer field that
changes neither what a walk produces nor how it produces it, so it is no part of the
scan scope or of any snapshot identity: a run with a handle and one without are the same
scan. `refresh` is left for the Python callback that would use it.

**What it counts:** work done, never index state.
The engine architecture keeps an in-progress cold build unobservable, and progress does
not change that: it reports how much the walk has read, not what the answer is.
Counts may include entries a retry rereads; the display calls them walked, not found.

**Hot-path cost:** walker workers already keep local counts in their `ScanReport`. They
add the deltas to shared counters once per batch they already hand to the sink, never
per entry, and the shared counters sit on separate cache lines.
Without a handle, the cost is one `Option` check per batch.
Content analysis updates its counter in the result loop that already runs on the caller
thread, where the candidate total is known before the first file.

**Invariant:** when a route completes, the handle’s files and bytes equal the report’s
own walked totals. This makes the counters testable exactly, not merely plausibly.

Nothing calls back into the caller.
A ticker thread polls, so there is no re-entrancy, no callback cost on worker threads,
and a later Python binding can poll the same way across the FFI boundary.

### Command Line: Drawing and Gating

`cli.rs` owns every presentation decision, as it does for color.

- **Interactive.** A run is interactive only when stderr is a terminal, `TERM` is set
  and is not `dumb`, `CI` is unset or empty, and, on Windows, virtual terminal
  processing could be enabled for the console (`anstyle-query`’s safe
  `enable_ansi_colors`, already a dependency through clap).
  A legacy console that refuses it would print the erase sequence as text, so it counts
  as non-interactive. These facts are read once, in `main`, and passed in, so tests set
  them directly. A non-interactive run never draws, installs no signal handler, and
  behaves exactly as today, whatever `--progress` says.
- **Gating.** On an interactive run, `auto` draws for text, tree, paths, and long output
  and not for machine formats; `always` draws for every format; `never` never draws.
  Commands that do no walk (`--docs`, `--skill`, `--cache-status`, `--cache-clear`)
  never draw. `NO_COLOR`, `FORCE_COLOR`, and `--color` never turn the indicator on or
  off; they decide only whether it is colored, as under [Appearance](#appearance).
- **Timing.** A ticker thread waits 500 ms before the first frame.
  A run that finishes sooner stops the ticker before it draws, so it shows no indicator
  and writes no bytes to stderr.
  The wait is a timed receive on the stop channel, not a sleep, so stopping returns at
  once and a fast run never pays the delay on exit.
  After that the ticker redraws every 80 ms, one spinner frame per redraw.
  A snapshot still in `Starting` draws nothing: the ticker keeps waiting until the
  engine names a phase, rather than show one it invented.
- **Frame.** One line on stderr, written as `\r\x1b[2K` plus the frame, exactly as
  specified under [Appearance](#appearance).
  The cursor is never hidden.
- **Clearing.** Before any write to stdout or stderr (report, warning, error, or the
  performance line) the ticker is stopped and joined and the line cleared.
  A guard does the same on unwind.
  Write errors on the progress line are ignored and never change the exit status; after
  the first failed write the ticker stops drawing.
- **Watch.** The indicator runs during the initial scan and stops when the first report
  paints; after that the watch repaint is the progress.
- **Ctrl-C.** A handler is installed only on an interactive run that will draw, so every
  other run keeps today’s default signal behavior exactly.
  When Ctrl-C arrives, the handler takes the ticker’s lock, marks it stopped so no later
  frame is drawn, erases the line if a frame is showing, and writes `fdu: interrupted`
  to stderr. It then ends the process the way the default action would: on Unix it
  restores the default `SIGINT` disposition and raises the signal again, so the shell
  reports 130 and a calling script stops; on Windows it ends as the console’s default
  Ctrl-C handling does.
  A Ctrl-C after the indicator has stopped, while the report is written, skips the
  message and takes the same default action.
  The handler adds no `unsafe` code to fdu, and each platform gets the crate whose safe
  API reproduces its default action, both in the command-line crate only and both in
  releases more than 14 days old.
  On Unix it is `signal-hook` 0.4.4 (published 2026-04-04): its `Signals` iterator hands
  `SIGINT` to a thread of fdu’s own, which may write to stderr, and its
  `emulate_default_handler` restores the default disposition and raises the signal
  again, which is death by `SIGINT` exactly; `ctrlc` there would add `nix` and, on
  macOS, a Grand Central Dispatch binding, for the same signal.
  On Windows it is `ctrlc` 3.5.2 (published 2026-02-10), what dust uses: its console
  handler runs a closure on a thread of its own, and the closure exits with
  `STATUS_CONTROL_C_EXIT`, the status the console’s default handling ends a process
  with; `signal-hook` there reaches only the C runtime’s `SIGINT`, whose default is an
  exit status of 3 rather than the console’s, and gives a handler no thread to write
  from. An interruption can leave the report cut short on stdout and a cache staging
  file, which the cache already recognizes and removes; no snapshot is partially
  published.

### Appearance

One line, redrawn in place on stderr.
Each frame starts with a braille “dots” spinner that advances one cell per 80 ms redraw:

```text
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

Every phase uses the same slots in the same order: spinner, root, phase word, the
phase’s facts, elapsed time.
The root comes first because it is what the run is about and it stays put for the whole
run; the phase word is padded to the width of the longest one (`Revalidating`), so the
facts do not jump when the phase changes.
There is no bar: a walk has no known total, and the one phase that has one shows it as a
percentage in the facts slot.
The frame for each phase, shown here in plain text:

```text
⠹ ~/wrk/github  Loading       0.6 s
⠼ ~/wrk/github  Scanning      412,309 files · 12,041 dirs · 38 GiB  3.1 s
⠼ ~/wrk/github  Revalidating  412,309 files · 12,041 dirs · 38 GiB  1.4 s
⠧ ~/wrk/github  Analyzing      24%  12,044 / 50,110 files  7.9 s
⠏ ~/wrk/github  Saving        8.1 s
```

**Colors** reuse the palette the report already uses, through `anstyle`, which is
already a dependency:

| Part | Style | Why this style |
| --- | --- | --- |
| Spinner | cyan | the report’s directory hue |
| Phase word | bold | reads first |
| Root path | cyan | the same path styling as the report’s directory rows |
| Numbers and sizes | default | the facts |
| Units and separators (`files`, `dirs`, `·`, `/`) | bright black | recede behind the numbers |
| Elapsed time | bright black | the performance line’s hue |
| Percentage | default | a number |

**Numbers** use the report’s own formatters: counts with thousands separators
(`human_count`), sizes in binary units (`human_bytes`), which shows a decimal only below
ten of a unit, so `3.2 GiB` but `38 GiB`, exactly as the report’s rows do.
Elapsed time has one decimal below a minute (`3.1 s`), then `1 m 04 s`, then `1 h 02 m`.

**Percentage.** A whole percentage, right-aligned in four columns (` 7%`, ` 24%`,
`100%`), shown only during content analysis, the one phase with an exact denominator.
It never reaches `100%` before the last file is applied.
An analysis of nothing, which the engine reports as `0 / 0` when the content sidecar
answered every candidate, is `100%`: there is nothing left to do.

**Color rule.** The frame is colored when stderr color is on under the same rule that
colors fdu’s warnings: `--color`, then `NO_COLOR`, then `FORCE_COLOR`. With color off,
the frame keeps the same text and animation, with no escape sequences beyond
`\r\x1b[2K`.

**Width.** A frame never wraps.
The width is stderr’s own terminal width (`terminal_size_of`, already a dependency
through clap), read again for every frame so a resize takes effect at once, and 80
columns when unknown.
The frame is measured without its color codes, counts each non-ASCII character of the
root path as two columns, and leaves the last column empty.
When it is wider than that, it shrinks in this order until it fits:

1. The phase word’s padding is dropped.
2. The root path is elided in the middle with `…`, down to 12 columns.
3. The `dirs` count is dropped.
4. The bytes are dropped.
5. Below 20 columns, only the spinner and the phase word are drawn, without the root.

**End of run.** The line is erased before the report or any message is written.
No summary replaces it, because the report’s own performance line states the totals.
A run interrupted with Ctrl-C erases the line, then writes `fdu: interrupted` in the
error style, colored under the same rule.

A runnable preview of these frames, colors, and timings is attached to the pull request
that introduced this plan.

### Documentation Changes

The `--docs` guide’s “never … animates progress” sentence becomes a statement of the
gating rule, and the `--progress` help says outright that `always` never draws into a
pipe or file, unlike `--color always`. `docs/usage.md` gains the flag and the
interrupted-run behavior.
The design principles note that a wait indicator is presentation over an engine fact;
the engine architecture records that progress counts work, not index state; the
CHANGELOG describes the flag.
The agent skill says only that agents need no flag, because the indicator never draws
for them, and points to `--docs` rather than restating the rules.

## Implementation Plan

One phase, tracked as beads under `fdu-vngp`. Engine work and the pure command-line
pieces can proceed in parallel; the ticker joins them.

| Bead | Work | Depends on |
| --- | --- | --- |
| `fdu-hlb1` | `Progress` handle and walk counters (cold, summary, warm), with the equality invariant | — |
| `fdu-mhx0` | Load, analyze, and save phases; `prepare_report_with_progress`; `Session::start_with_progress` | `fdu-hlb1` |
| `fdu-p1gr` | Interactive detection, injected terminal facts, `--progress`, gating tests | — |
| `fdu-vpdw` | Frame renderer, exactly per [Appearance](#appearance) | — |
| `fdu-hjjj` | Ticker, 500 ms delay, clearing and ordering, one-shot and watch wiring | `fdu-mhx0`, `fdu-p1gr`, `fdu-vpdw` |
| `fdu-9286` | Ctrl-C handler with the default interrupt action | `fdu-hjjj` |
| `fdu-3xiz` | Real-terminal smoke test and manual terminal QA | `fdu-hjjj`, `fdu-9286` |
| `fdu-n6bd` | Documentation, help text, and the help golden | `fdu-hjjj`, `fdu-9286` |
| `fdu-2e8o` | Performance comparison, recorded in the experiment ledger | `fdu-hlb1`, `fdu-mhx0` |

`fdu-m893`’s planned flags are renamed off `--progress`, recorded in its bead.

## Testing Strategy

- **Engine.** Counters are monotonic across snapshots taken during a run, and at
  completion equal the report’s walked totals on the cold, warm, summary, and analysis
  routes. A run with no handle produces the same report bytes.
- **Gating.** Following urollup, the terminal facts are injected.
  Every non-interactive combination (stderr not a terminal, `TERM` unset or `dumb`, `CI`
  set) writes nothing to stderr under all three flag values.
  On an interactive run, `auto` draws for human formats and not for machine formats,
  `always` draws for both, and `never` draws nothing.
  Tests assert exact stderr bytes.
- **Delay.** With an injected clock, a run that finishes before 500 ms writes no
  progress bytes, and one that finishes after writes its first frame at 500 ms.
- **Appearance.** Frame rendering is a pure function of a snapshot, elapsed time,
  spinner step, width, and color flag; unit tests pin each phase’s exact text, the
  styled bytes, the width fallbacks in order, and the elapsed-time formats.
- **Ordering.** With a zero delay, a run that warns asserts that the clear sequence
  precedes the first warning, and a failing run that it precedes the error.
- **Interruption.** Against an injected terminal, the handler’s order is asserted (stop,
  erase, message, default action), and no frame is drawn after the stop.
- **Pipes.** With the indicator active and stdout closed early, the run still follows
  today’s broken-pipe rule, and a failed stderr write never changes the exit status.
- **Real terminal.** One Python `pty` smoke test runs the built binary in a
  pseudo-terminal, with `TERM` set and `CI` removed from its environment and under a
  timeout, and asserts a drawn frame, a clean final line, and death by `SIGINT` after an
  interrupt. It is skipped on Windows, where Python has no `pty`.
- **Manual QA.** `tests/qa/cli-installed-e2e.qa.md` gains a terminal phase: a large tree
  shows the indicator, a small one shows nothing, redirected stderr shows nothing,
  Ctrl-C leaves a clean prompt, a resized window shrinks the frame, and no line is left
  on screen. A leftover line fails the check.
- **Unchanged surfaces.** Goldens run with piped streams and must not change; the
  existing tests that require empty stderr under `--color always` keep passing.
- **Performance.** Interleaved, paired `make perf-compare` on a real tree.
  A run with no progress handle must be indistinguishable from main, and the performance
  probe with a handle attached (the engine’s share of the cost, since measured runs are
  never interactive) must be within noise.
  The result is recorded with `make perf-record`, with its regime.

## Open Questions

- Whether a later version shows the directory being read, which helps diagnose a stalled
  network mount but needs a sampled path shared with the walker.
- Whether the 500 ms delay and the 80 ms redraw need tuning after use on real trees.
- Whether an ASCII spinner is needed for terminals without braille glyphs.
  The report already draws its bars with `█`, so this plan assumes the same Unicode
  support.

## References

- `fdu-vngp`, `fdu-m893`
- [Design principles](../../architecture/fdu-design-principles.md)
- [Engine architecture](../../architecture/fdu-engine-architecture.md)
- [Surface architecture](../../architecture/fdu-surface-architecture.md)
- dust v1.2.4 in `attic/dust` (`src/progress.rs`, `src/main.rs`); urollup’s design
  decision 30 and its injected-terminal tests

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
