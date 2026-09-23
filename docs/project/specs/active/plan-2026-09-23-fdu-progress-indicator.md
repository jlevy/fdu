# Plan: An Interactive Progress Indicator

**Date:** 2026-09-23

**Status:** Approved design, not yet implemented

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
  a one-shot report, and analyzed files with a percentage during content analysis.
- Draw only when stderr is an interactive terminal, the terminal is not `dumb`, and the
  report format is a human one; draw nothing otherwise.
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

The tbd `rust-cli-rules` guideline requires progress on stderr, suppressed when that
stream is not a terminal.

## Design

### Decisions

Recorded with the maintainer on 2026-09-23:

- The flag is `--progress auto|always|never`, default `auto`, mirroring `--color`.
- `auto` suppresses the indicator for machine formats (`json`, `jsonl`, `yaml`) even on
  a terminal, because agent harnesses run commands under a pseudo-terminal and a redrawn
  line would pollute their captured stderr.
  `--progress always` overrides.
- Ctrl-C follows dust’s model, corrected: while the indicator is active, a handler
  clears the line, writes `fdu: interrupted` to stderr, and exits 130.
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

pub enum ProgressPhase { Loading, Scanning, Revalidating, Analyzing, Saving }
```

It enters through variants beside today’s entry points rather than through `Delivery`,
which stays a plain comparable value:
`prepare_report_with_progress(request, delivery, &progress)`, following
`prepare_report_with_scan_diagnostics`, plus a progress argument on `Session::start` for
the watch’s initial scan.
`refresh` is left for the Python callback that would use it.

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

- **Gating.** `auto` draws only when stderr is a terminal, `TERM` is not `dumb`, and the
  format is text, tree, paths, or long.
  `always` draws regardless; `never` never draws.
  `NO_COLOR`, `FORCE_COLOR`, and `--color` do not affect progress.
  Commands that do no walk (`--docs`, `--skill`, `--cache-status`, `--cache-clear`)
  never draw.
- **Timing.** A ticker thread waits 500 ms before the first frame, so fast runs never
  flash, then redraws about eight times a second.
- **Frame.** One line on stderr, written as `\r\x1b[2K` plus the text, truncated to the
  terminal width in display columns so it never wraps:
  `scanning ~/wrk · 412,309 files · 1,204 dirs · 38.2 GiB · 3.1s` and
  `analyzing · 12,044 / 50,110 files (24%) · 7.9s`. The spinner is ASCII, and the cursor
  is never hidden.
- **Clearing.** Before any write to stdout or stderr (report, warning, error, or the
  performance line) the ticker is stopped and joined and the line cleared.
  A guard does the same on unwind.
  Write errors on the progress line are ignored; they never fail the run.
- **Watch.** The indicator runs during the initial scan and stops when the first report
  paints; after that the watch repaint is the progress.
- **Ctrl-C.** A handler is installed only when the indicator will draw, so piped, CI,
  and Python-subprocess runs keep the default signal behavior.
  It clears the line, writes `fdu: interrupted` to stderr, and exits with status 130.
  The `ctrlc` crate provides it on Unix and Windows; the chosen release must be more
  than 14 days old, per the supply-chain policy.
  It adds `nix`, and `dispatch2` and `block2` on macOS, to the command-line crate only;
  `windows-sys` 0.61 is already a dependency.

### Documentation Changes

The `--docs` guide’s “never … animates progress” sentence becomes a statement of the
gating rule. `docs/usage.md` gains the flag and the exit status 130 for an interrupted
interactive run.
The design principles note that a wait indicator is presentation over an
engine fact; the engine architecture records that progress counts work, not index state;
the CHANGELOG and the agent skill describe the flag.

## Implementation Plan

### Phase 1: Engine Handle and Command-Line Indicator

- [ ] Add `Progress`, `ProgressSnapshot`, and `ProgressPhase` to `fdu-core`, with
  per-batch updates on the cold walk, the summary walk, and warm reconciliation, the
  analysis counter, and the load and save phases.
- [ ] Add `prepare_report_with_progress` and the `Session::start` argument.
- [ ] Add `--progress auto|always|never`, the ticker, the frame, the clearing guard, and
  the Ctrl-C handler to `cli.rs`.
- [ ] Rename `fdu-m893`’s planned flags off `--progress` in its bead.
- [ ] Update the documents listed above.
- [ ] Record the performance comparison in the experiment ledger.

## Testing Strategy

- **Engine.** Counters are monotonic across snapshots taken during a run, and at
  completion equal the report’s walked totals on the cold, warm, summary, and analysis
  routes. A run with no handle produces the same report bytes.
- **Gating.** Following urollup, the terminal facts are injected: `auto` with a terminal
  draws, without one does not, machine formats do not, `TERM=dumb` does not, `never`
  never draws, and `always` draws into a pipe.
  Tests assert exact stderr bytes.
- **Ordering.** With a zero delay, a run that warns asserts that the clear sequence
  precedes the first warning, and a failing run that it precedes the error.
- **Real terminal.** One Python `pty` smoke test runs the built binary in a
  pseudo-terminal and asserts a drawn frame, a clean final line, and an exit status of
  130 after an interrupt.
- **Unchanged surfaces.** Goldens run with piped streams and must not change; the
  existing tests that require empty stderr under `--color always` keep passing.
- **Performance.** Interleaved, paired `make perf-compare` on a real tree:
  `--progress never` must be indistinguishable from main, and `--progress always` within
  noise. The result is recorded with `make perf-record`, with its regime.

## Open Questions

- Whether a later version shows the directory being read, which helps diagnose a stalled
  network mount but needs a sampled path shared with the walker.
- Whether the 500 ms delay and eight-per-second refresh need tuning after use on real
  trees.

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
