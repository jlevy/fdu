# Feature: Python CLI Parity

**Date:** 2026-08-21

**Author:** fdu project

**Status:** Draft

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
Two mechanisms make it structural and legible instead.

**No bare `fdu` on `PATH` during a parity run.** Each surface installs under its own
name, so a missing shim is `command not found` on the first session rather than a silent
substitution.

**And tryscript reports what it resolved**, which is the feature worth adding:

```yaml
path:
  - $FDU_SURFACE_BIN     # the only directory supplying the command
requires:
  - fdu                  # must resolve before the first session; abort otherwise
```

```text
resolved fdu -> /…/tests/parity/py/fdu   (12 files, 129 sessions)
```

The guard is then enforced once rather than repeated in two hundred session lines, the
corpus stays readable, and the run *states which binary it exercised* — the difference
between a harness that is correct and one that can be seen to be correct, which is the
same reason every fdu report carries its own `source` and `freshness` rather than
leaving a reader to infer them.

We maintain tryscript, so this is a feature to add rather than a constraint to route
around.

### API Changes

None to shipped surfaces.
The harness is additive: one shim, one Make target, and one front-matter line per golden
file.

## Implementation Plan

### Phase 1

- [ ] Add `Report.render(format, color)` to the Python API over the existing renderer,
  so the package can produce fdu’s own output rather than only structured values
  (`fdu-z84z`)
- [ ] tryscript: drop empty `path:` entries, with a test that a bare `$VAR` does not put
  the working directory on `PATH` (`fdu-nluf`)
- [ ] tryscript: add `requires:`, so named commands must resolve before the first
  session and the run reports where each resolved (`fdu-ds2x`)
- [ ] Write `tests/parity/py/parity_cli.py` against the table above, with `fdu:`
  diagnostics and exit 77 for `--help`, `--docs`, and `--skill` (`fdu-0len`)
- [ ] Add the runner: replay the corpus, diff against the Rust recording, compare with
  `tests/parity/deviations-python.diff` (`fdu-5clp`)
- [ ] Commit the first deviation file and review every hunk against the rule that only a
  version string or help layout is legitimate (`fdu-5clp`)
- [ ] Add `make test-parity` with its anti-vacuity guards (`fdu-szti`)

### Phase 2

- [ ] The same shim over the public Rust library API, with its own deviation file.
  Sharper than the Python one: if it cannot be written without reaching into `cli.rs`,
  then “the CLI invents nothing” is false and the library is missing something.

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
