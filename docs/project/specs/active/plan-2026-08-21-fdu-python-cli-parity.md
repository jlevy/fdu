# Feature: Python CLI Parity

**Date:** 2026-08-21

**Author:** fdu project

**Status:** Phase 1 landed; Phase 2 not started

108 of 126 golden sessions reach parity through the Python API alone.
The 18 that differ each carry a named cause, and an unexplained difference fails the run
rather than joining the list.
Two Phase 1 items were deferred to tryscript (`fdu-nluf`, `fdu-ds2x`) and are recorded
as deferred rather than done.

## Overview

Build a test-only executable that accepts fdu’s command line and produces fdu’s output,
implemented over the public Python package, then run the **existing golden corpus**
against it.

fdu ships three ways to ask the same question — the Rust library, the Python package,
and the CLI — and Principle 7 claims they are one capability wearing three faces.
Nothing tests that claim.
The CLI has 129 golden sessions; the Python surface has a handful of hand-written
assertions, which is how five separate defects reached it with `make check` green
throughout.

A parity CLI closes that by construction: every scenario already written applies to
Python for free, and a capability the CLI has that the API lacks makes the shim
unwritable.

## Goals

- Every golden session runs against the Python surface, with no second corpus to
  maintain.
- A gap between the CLI and the Python API becomes unshippable rather than merely
  undetected.
- The run states which binary it exercised, so the result can be *seen* to be about the
  surface it claims.
- The shim stays short.
  Its length is the measurement.

## Non-Goals

- Shipping the shim. It is a test fixture, excluded from the wheel.
- Replacing the Python unit tests.
  Parity proves sameness; the unit tests still pin what the CLI cannot reach, such as
  watch iteration.
- Text parity on the first pass.
  Machine formats are the contract that matters; identical human text is a later,
  stricter tier.

## Background

The Python wrapping carried five defects simultaneously, found by reading rather than by
testing: a second view parser that rejected `largest` and `recent`, a section dispatch
that raised `KeyError` on them, no `bound` on any typed section, a `View.FULL` the
binding refused while listing it as valid, and a fixed default view that returned a
directory tree to a caller who had asked to read every file in the tree.

Every one would have been caught by running a single existing golden session through the
Python API.

The distinction that matters, and the one already got wrong once: a *coverage* test asks
“does each view work in Python”.
Parity asks “does Python produce what the CLI produces”.
The first was written, passed, and would not have caught any of the five.

`Report.as_dict()` already promises “an independent copy of the exact CLI JSON schema” —
a promise never checked against the actual CLI until a throwaway script did it by hand
across nine views. It holds today.
This makes it hold tomorrow.

## Design

### The parity CLI

A single executable, `tests/parity/py/fdu`, that parses fdu’s argv and writes fdu’s
bytes using only the public `fdu` package.

Not a wrapper around the binary, which would test nothing.
It builds `ScanOptions`, `AnalysisOptions`, `Selection`, and `Query` from argv, calls
`open`/`scan`/`report`, and renders the result.
Where the CLI emits JSON it emits `Report.as_dict()`; where the CLI renders text it
renders text.

Two properties it inherits from the name it stands in for: its diagnostics say `fdu:`
rather than `fdu-py:`, and `--version` reports the same string, because the corpus pins
both and because it is impersonating fdu rather than announcing itself.

The shim is the measurement as well as the instrument.
If it needs private helpers, elaborate glue, or a capability the package does not
expose, that is a finding about the API, not an inconvenience to work around.

### What the shim has to be, file by file

```text
tests/parity/
  py/fdu                    executable, named `fdu` so PATH resolves it as one
  py/parity_cli.py          argv -> public Python API -> bytes
  run.mjs                   replay the corpus, diff, compare the deviation file
  deviations-python.diff    the committed artifact
```

`py/fdu` is a two-line launcher; `parity_cli.py` holds everything, and mirrors `cli.rs`
function for function so a reader can check the mapping by name:

| `cli.rs` | `parity_cli.py` | Public API it leans on |
| --- | --- | --- |
| `Cli::run` | `main(argv)` | dispatch and exit code |
| `parse_format` | `parse_format` | `--format` value, passed through |
| `parse_analysis` | `parse_analysis` | `AnalysisOptions(analyze=...)` |
| `parse_cache_policy` | `parse_cache` | `CachePolicy` |
| `parse_query` | `build_query` | `Query`, `Selection`, `ScanOptions` |
| `run_cache_lifecycle` | `run_cache_lifecycle` | `cache_status`, `list_caches`, `clear_cache`, `clear_all_caches` |
| the watch loop | `run_watch` | `Index.watch`, `WatchOptions` |
| `report_format::render` | `render` | **missing — see below** |
| `finish` | `exit_code` | `Status.complete`, `--allow-partial` |

View resolution, default derivation, and `full` expansion are deliberately absent from
that table: they live in the library now, and the binding calls them.
If the shim had to reimplement any of the three, that would be the drift this harness
exists to catch.

### The gap this design turned up before it was built

**The Python API cannot render text.** There is no `render` anywhere in the package or
the native stub — the typed surface returns structured values and `as_dict()`, and stops
there.

That matters because most of the corpus is text sessions.
A shim serving them would have to reimplement the human renderer in Python: the ten-cell
bars, the label padding, the view headers, the bound notes, the performance footer, the
colour rules. Hundreds of lines duplicating presentation, and the test would then be
measuring the reimplementation rather than the API — a harness that fails when two
renderers disagree about spacing, while staying silent about the report being wrong.

So Phase 1 adds `Report.render(format, color)` to the Python API, a thin binding over
the renderer the CLI already uses.

This is not test scaffolding.
It closes a real gap: today a Python caller who wants fdu’s own output has to shell out
to the binary, which is the same admission the console script already makes —
`fdu:_main` calls `_native.main()`, so the `fdu` command the wheel installs has never
exercised a line of the Python API. Making the renderer reachable is what lets Python be
the CLI’s equal rather than a data source beside it.

It also shrinks the shim to what it should be: argv in, API calls, bytes out.

**A collision worth naming.** That console script means installing the wheel puts an
`fdu` on `PATH` that is the Rust CLI. During a parity run it must not be reachable, or
the run silently measures the binary against itself — the same failure the deviation
file catches after the fact and the naming rule prevents outright.

### The shape of `Report.render`

```python
report.render(Format.TEXT, color=False) -> str
```

A **method**, beside `as_dict()`, because both are serializations of the same value and
splitting them across a method and a module function would make the pair harder to find
than either alone.

`Format` joins the twelve `StrEnum`s the package already exports, so `render("json")`
and `render(Format.JSON)` are the same call — the pattern `View` and `Analysis` already
set — and the native `contract()` gains a `formats` key that `public_smoke` checks for
parity.

`color` is a plain `bool`, not the CLI’s `auto | always | never`. Resolving `auto` means
asking whether stdout is a terminal, and a library does not own stdout; the caller does.
The shim resolves `--color` the way the CLI does and passes the answer in, which is the
correct division rather than a shortcut.

**The body only.** `render` returns the report, not the performance footer.
The footer is transient telemetry the schema deliberately excludes, and the walk counts
behind it are not on `Report`, so a Python caller cannot produce it.
Every text session therefore deviates by exactly that line.

That is recorded once, as a class, rather than as a hundred identical hunks:

```text
tests/parity/deviations-python.diff
  class: performance-footer-absent  (94 text sessions)
    -Performance: walked … total [PERF_TIME]
```

Collapsing it keeps the deviation file readable, which is the property the whole design
rests on — a file nobody reads is a file nobody reviews.
The count is the interesting part anyway: it says how much of the corpus is text, and it
is the standing argument for exposing the telemetry later if that trade ever looks worth
making.

### What the shim may decline

With the renderer exposed and cache and watch already public, the skip list is short,
and each entry is a decision rather than an oversight:

| Flag | Why |
| --- | --- |
| `--help` | clap’s rendering; the Python package has no argument parser to render |
| `--docs`, `--skill` | static documents the package does not carry |

Everything else in the corpus is serveable.
That is the measurement: a skip list of three discovery surfaces says the Python API is
close to complete, and any growth in that list is a regression worth arguing about.

### One corpus, one expected output, and a checked-in deviation file

The corpus is not duplicated and the expected bytes are not duplicated.
`tests/golden/*.tryscript.md` stays exactly as it is — the Rust CLI’s recorded output,
reviewed and committed as now.

The parity run replays the same sessions against the shim, captures what it produced,
and diffs it against the Rust recording.
That diff is the artifact:

```text
tests/parity/deviations-python.diff
```

It is committed, reviewed like any golden, and it is the specification of *how the two
surfaces legitimately differ*. Nothing else about the corpus changes, and no expected
output is written twice.

This inverts the safety property in the way that matters.
A naive parity run treats “no differences” as success — which is exactly what a
fallthrough to the real binary produces, so the most dangerous failure looks identical
to the best outcome.
Here the committed deviation file is **non-empty by construction**: the shim reports a
different `--version` build string, and the two help renderers differ in formatting.
If a run produces an empty diff, the shim did not run, and the harness fails.

Four outcomes, all distinguishable:

| Run produces | Means |
| --- | --- |
| the committed deviations, exactly | parity holds |
| an empty diff | the shim never ran — a fallthrough, or the wrong binary |
| extra hunks | the Python surface drifted; a parity bug, in the diff |
| missing hunks | a known deviation was fixed — update the file, visibly, in review |

The size budget the golden guidelines set — small enough to review in a pull request —
argues for the same thing from the other side.
Two full corpora of expected output would be thousands of duplicated lines nobody reads;
a deviation file that a reviewer can read top to bottom in a minute is the artifact
worth having, and it shrinks as parity improves.

### What counts as a legitimate deviation

Only presentation of the surface’s own identity, and only where the corpus pins it:

- the `--version` build string, which names the surface
- help and `--docs` layout, where two renderers wrap differently

Everything else is a bug.
Machine formats — `json`, `jsonl`, `yaml` — must be byte-identical, because
`Report.as_dict()` already promises “an independent copy of the exact CLI JSON schema”
and that promise is either true or it is not.
Report content, ordering, bounds, exit codes, and diagnostics are the contract, not
presentation, and a difference in any of them belongs in a bug rather than in the
deviation file.

The file is reviewed with that rule in hand.
A hunk that is not a version string or a help-layout artifact is a finding, and the
review question is always “why is this allowed” rather than “does this look plausible”.

### Making a fallthrough impossible rather than unlikely

The deviation file catches a fallthrough after the fact, which is enough to fail the run
but tells the reader little.
Probing tryscript 0.2.0 rather than reasoning about it changed this section, and the
answer removed work instead of adding it.

**What the probe found.** tryscript inherits the parent environment, and expands
arbitrary environment variables in `path:`, not just `$TRYSCRIPT_GIT_ROOT`. It also
*prepends* those entries to the inherited `PATH` rather than replacing them, and an
`env: PATH:` override does not replace it either.

That last point is a defect in the corpus as it stood, not a limitation to design
around. Every golden selected its build with one `path:` entry.
If that entry ever failed to resolve, lookup continued into the inherited `PATH` and
found whatever `fdu` was installed there — `~/.cargo/bin/fdu` on a developer machine —
and the suite passed while testing a different binary.
Filed as `fdu-9h2w`.

**The corpus names the directory, not the command.** Sessions invoke a bare `fdu` and
declare `path: - $FDU_BIN`, which tryscript expands.

The first attempt put the full path in a variable and had sessions invoke `$FDU`.
Windows rejected every one of them:

```text
'$FDU' is not recognized as an internal or external command
```

tryscript runs sessions through `cmd.exe` there, which wants `%FDU%`. A shell variable
in a session command line is not portable, so the corpus has to be readable by `/bin/sh`
and `cmd.exe` alike, and a bare command name is the only form both read the same way.
That is a design error worth recording rather than quietly fixing: the harness pins two
implementations of one CLI across three platforms, so anything in the corpus that is not
shell-agnostic will fail on one of them.

The split therefore follows who does the resolving:

| Who runs the binary | How it is named | Why |
| --- | --- | --- |
| a session command line | bare `fdu`, directory from `path: - $FDU_BIN` | must parse under `sh` and `cmd.exe` |
| a node helper (`tests/golden/bin/*.mjs`) | `process.env.FDU`, full path with extension | spawns directly; no PATH lookup to control |

This is not a return to the original defect.
What made that a defect was the directory being a literal that could silently fail to
resolve while looking correct.
`run-golden.mjs` preflights the surface and refuses to start when it is missing;
verified by hiding `target/debug/fdu` with an installed build still on `PATH`, where the
run now stops with the path it wanted instead of testing the installed copy and passing.

Both surfaces are still *named* `fdu`, so every program-name string the corpus pins —
usage lines, diagnostics — matches without the corpus knowing which surface is running.

**What remains tryscript’s to fix.** `path:` prepends to the inherited `PATH` rather
than being authoritative for command resolution, so the preflight is fdu guarding
against a tryscript behaviour.
Either an authoritative path mode or a `requires:` declaration that asserts what
resolved would close it (`fdu-ds2x`, `fdu-z7sp`).

`jlevy/tryscript#51` fixes a real adjacent inconsistency — `path:` expanded `$VAR` while
`env:` did not, so a test could name a directory by absolute path but never a file — and
adds `TRYSCRIPT_EXE`. It does **not** remove this runner, contrary to what this spec
claimed before Windows disproved it.

`scripts/run-golden.mjs` owns the surface choice, because CI runs `npm run test:golden`
directly and would miss anything set only in the Makefile.
Its remaining job is irreducible while parity exists: two surfaces over one corpus means
something has to choose, and it sets `$FDU_BIN`, preflights, and says which surface it
resolved. `scripts/check-golden-invocations.mjs` polices the rule that replaced the old
one — every session declares exactly one path entry and it is `$FDU_BIN`, and no helper
names `fdu` literally.

### What the harness measured on its first run

88 of 126 sessions reached parity immediately: report bodies byte-identical across every
view, every format, every selection axis, through the Python API alone.

The 38 that did not are the measurement, and each is a tracked gap rather than an
accepted difference:

| Sessions | Gap | Bead |
| --- | --- | --- |
| 16 | cache status and watch records have no renderer | `fdu-1kw3` |
| 6 | the binding’s copy of the `full` diagnostic drifted | `fdu-gw5b` |
| 2 | list grammar (duplicates, empty entries) is not exposed | `fdu-jozr` |
| 1 | `--version` names the surface | deliberate |
| rest | consequences of the above | — |

Two findings came out of writing the shim rather than running it, and both are the same
root cause the epic already had: **the binding keeps its own copies of things the
library owns.** `contract()` hard-coded a view list that had drifted out of
`ViewSpec::ALL` order, and the parity assertion never noticed because Python had been
written from the same copy (`fdu-ggux`). The `full` diagnostic is hand-copied and has
lost a clause (`fdu-gw5b`). `contract()` now derives from `ViewSpec::ALL`.

**The performance footer** is excluded from comparison rather than recorded.
It reports walk telemetry the report schema deliberately excludes, so a `Report` does
not carry the counts behind it and the Python surface cannot print it.
Left in, it failed 44 sessions and buried the artifact under whole re-printed blocks.
The parity corpus is therefore the same sessions with that one line dropped, generated
per run and never committed.

### API Changes

None to shipped surfaces.
The harness is additive: one shim, one Make target, and one front-matter line per golden
file.

## Implementation Plan

### Phase 1

Landed. 108 of 126 golden sessions reach parity through the Python API alone.

- [x] Add `Report.render(format, color)` to the Python API over the existing renderer,
  so the package can produce fdu’s own output rather than only structured values
  (`fdu-z84z`)
- [x] Write `tests/parity/py/parity_cli.py` against the table above, with `fdu:`
  diagnostics (`fdu-0len`)
- [x] Add the runner: replay the corpus, diff against the Rust recording, compare with
  `tests/parity/deviations-python.diff` (`fdu-5clp`)
- [x] Commit the first deviation file and review every hunk (`fdu-5clp`)
- [x] Add `make test-parity` with its anti-vacuity guards (`fdu-szti`)

Two Phase 1 entries were **deferred rather than done**, and both are tryscript-side:

- [ ] tryscript: drop empty `path:` entries, with a test that a bare `$VAR` does not put
  the working directory on `PATH` (`fdu-nluf`) — still a real robustness issue, but no
  longer on this critical path: the corpus carries no `path:` entry that can be empty.
- [ ] tryscript: add `requires:`, so named commands must resolve before the first
  session and the run reports where each resolved (`fdu-ds2x`) — superseded in practice.
  `scripts/run-golden.mjs` preflights the binary and states which surface it resolved,
  so the guarantee exists; having tryscript enforce it would move the check off fdu,
  which is still worth doing and is why the bead stays open.

Neither was dropped silently, and neither blocks the harness.

**Two things this plan specified that the implementation did not do**, recorded here
rather than quietly diverged from:

1. The spec said the shim would exit **77** for `--help`, `--docs`, and `--skill`. It
   exits **2** with a one-line declination on stderr, because 77 is not a code fdu uses
   anywhere and a reader comparing the two surfaces would have had to learn a private
   convention to interpret it.
   The runner excludes those sessions by name instead, which is visible in
   `scripts/parity-classes.mjs` rather than encoded in an exit status.
2. The spec said only a version string or help layout could be a legitimate deviation.
   That rule survived contact with reality as **four** named classes, not two — each
   mechanically matched, and an unexplained difference fails the run.
   The additions are surfaces naming their own parameters, and notes carrying walk
   telemetry the report schema excludes.
   Both are correct on each side; neither was foreseen here.

### Phase 2

Not started. Tracked under `fdu-luwc`, which stays open for it.

- [ ] The same shim over the public Rust library API, with its own deviation file.
  Sharper than the Python one: if it cannot be written without reaching into `cli.rs`,
  then “the CLI invents nothing” is false and the library is missing something.

This is worth more now than when it was written.
Phase 1 found seven definitions the CLI had copied from the library and five
capabilities only the CLI could reach; a Rust-side shim is the instrument that would
have caught them without a second language in the way.

## Testing Strategy

The harness is the test, so its own correctness needs guards — a parity runner nobody
can trust is worse than none, because it produces a green check attesting to something
nobody verified.

- **The deviation file is non-empty by construction.** An empty diff means the shim
  never ran, so the check that would otherwise be the most dangerous failure — a silent
  fallthrough looking exactly like perfect parity — becomes the loudest.
- **Surface identity is structural.** No bare `fdu` on `PATH`, so a missing shim is
  `command not found` on the first session rather than a substitution, and `requires:`
  prints what resolved.
- **Sensitivity.** A deliberate divergence must fail: break the shim’s view mapping and
  extra hunks appear.
- **Skip accounting.** Skips are counted and asserted, so a shim cannot skip its way to
  green.

Verified the hard way once already: a check that the Python view test caught a real
regression was invalid, because it edited the source while the test ran the installed
wheel. Breaking the installed copy was the only thing that proved anything.
The same care applies to every guard above — each one is only worth having if it has
been seen to fail.

## Rollout Plan

`make test-parity` stays outside `make check` until the shim is complete and its skip
list is argued, then joins it.
Keeping it out first lets the corpus run and its findings be triaged without blocking
unrelated work.

## Open Questions

- Does text parity become a later tier?
  The human renderer is where the layout rules live, and those are worth pinning across
  surfaces eventually.
- Should the Rust shim absorb `examples/perf_probe.rs`’s role as the library-level
  exerciser? Different jobs — one measures, one compares — but both exist to prove the
  library API is usable without the CLI.

## References

- [Design principles: First Principles](../../architecture/fdu-design-principles.md#first-principles)
- [Composable CLI and query surface](plan-2026-08-10-fdu-composable-cli-surface.md),
  Principle 7
- [View vocabulary and the output contract](plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md)
- [tbd guidelines: golden testing](../../../.tbd/docs/guidelines/golden-testing-guidelines.md)
  — the size budget and the “easy to diff” requirement are what argue for a deviation
  file rather than a second corpus
- Beads: `fdu-luwc` (epic), `fdu-ds2x` (tryscript `requires:`), `fdu-nluf` (the
  tryscript `PATH` bug), `fdu-0len` (the parity CLI), `fdu-5clp` (the deviation file),
  `fdu-szti` (`make test-parity`)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
