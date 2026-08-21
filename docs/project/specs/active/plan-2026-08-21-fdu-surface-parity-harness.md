# Feature: Surface Parity Harness

**Date:** 2026-08-21

**Author:** fdu project

**Status:** Draft

## Overview

Run the existing golden corpus against every surface, not just the command line.

fdu ships three ways to ask the same question — the Rust library, the Python package,
and the CLI — and Principle 7 claims they are the same capability wearing three faces.
Nothing tests that claim.
The CLI is covered by 129 golden sessions; the other two are covered by a handful of
hand-written assertions, which is why five real defects reached the Python surface and
`make check` stayed green through all of them.

The idea is to make the claim executable: build test-only shims that reimplement the CLI
over the library API and over the Python API, then run the *same* golden files against
each. Any divergence is a parity bug, and it surfaces as a diff in a corpus that already
exists.

## Goals

- Every golden session runs against all three surfaces, with no second corpus to
  maintain.
- A capability the CLI has and an API lacks becomes impossible to ship, because the shim
  cannot be written without it.
- The shims are short.
  Length is the measurement: a shim that needs private helpers or elaborate glue is
  reporting that the public API is not the CLI’s equal.

## Non-Goals

- Shipping either shim.
  They are test fixtures, excluded from the wheel and the crate.
- Byte-identical *human* text from the shims at first.
  Machine formats are the contract that matters; text parity is a later, stricter tier.
- Replacing the existing Python tests.
  The parity run proves sameness; the unit tests still pin behaviour the CLI does not
  reach, such as watch iteration.

## Background

The Python wrapping carried five defects at once, found by reading rather than by
testing: a second view parser that rejected `largest` and `recent`, a section dispatch
that raised `KeyError` on them, no `bound` on any typed section, a `View.FULL` the
binding refused while listing it as valid, and a fixed default view that returned a
directory tree to a caller who had asked to read every file.

Each was the same root cause — the binding held its own copy of grammar or defaults —
and each would have been caught immediately by running one existing golden session
through the Python API.

`Report.as_dict()` already promises “an independent copy of the exact CLI JSON schema”.
That promise had never been checked against the actual CLI. Checking it by hand across
nine views took one short script, and it passes today; the harness makes it permanent
and extends it to every scenario the corpus contains.

## Design

### Three surfaces, one corpus

| Surface | Entry point | What a divergence proves |
| --- | --- | --- |
| CLI | `target/debug/fdu` | the baseline the goldens already pin |
| Rust library | `examples/fdu_via_lib.rs` | the CLI reached past the public API |
| Python package | `tests/parity/fdu_via_py.py` | the binding drifted from the library |

Each shim parses the same argv and writes the same bytes to stdout, using only its
surface’s public API. The Rust shim is the sharper of the two: if it cannot be written
without reaching into `cli.rs`, then “the CLI invents nothing” is false and the library
is missing something.

### Making a fallthrough impossible rather than unlikely

The obvious mechanism — prepend a shim directory to `PATH`, leave the sessions saying
`fdu` — has a failure that would not announce itself.
If the shim is missing, misspelled, or not executable, `PATH` falls through to the real
binary and the whole parity run tests the CLI against itself.
Every session passes.
The run is green and proves nothing, which is worse than not having run it, because now
there is a green check attesting to parity nobody verified.

Two ways to close that, and the difference between them is who pays.

**Name the surface in every session.** Sessions call `$FDU`, always set to `fdu-cli`,
`fdu-py`, or `fdu-rs`, and no executable named `fdu` is on `PATH` during a parity run.
An unset variable or a missing shim is `command not found` on the first session.
This works today with no tryscript change, and it taxes every future reader of the
corpus: `$ $FDU --cache off --view largest project` is meaningfully harder to read than
`$ fdu --cache off --view largest project`, and the golden files are documentation as
much as they are tests.

**Or make tryscript resolve and report it**, which is the option worth building.
The corpus keeps saying `fdu`. A small front-matter addition declares what must resolve
before anything runs, and tryscript prints where it resolved:

```yaml
path:
  - $FDU_SURFACE_BIN     # the only directory supplying `fdu`
requires:
  - fdu                  # must resolve before the first session; abort if it does not
```

```text
resolved fdu -> /…/tests/parity/py/fdu    (12 files, 129 sessions)
```

That is strictly better on three counts.
The guard is enforced once rather than repeated in two hundred session lines, so it
cannot be forgotten in a session added later.
The corpus stays readable.
And the run *states which binary it exercised*, which is the difference between a
harness that is correct and one that can be seen to be correct — the same reason every
fdu report carries its own `source` and `freshness` rather than leaving the reader to
infer them.

We maintain tryscript, so this is a feature to add rather than a constraint to work
around. The `$FDU` form is the fallback if `requires:` turns out not to be worth it.

**A second tryscript bug, found while validating this.** An unset `path:` variable
expands to an empty string and is passed through as an empty `PATH` entry, which POSIX
reads as the current directory.
Verified: with the variable unset, an executable in the working directory was found and
run. A footgun for any tryscript user, and independent of this feature.

### What the shims must handle, and what they may skip

The corpus contains sessions the shims cannot serve: cache lifecycle flags, `--watch`,
`--skill`, `--docs`. A shim exits with a distinct status for “not implemented by this
surface”, and the runner reports those as skipped with a count.
Silence is not acceptable: a shim that quietly passed by doing nothing would make the
whole harness a no-op, which is the one failure mode that would not announce itself.

The skip list is itself evidence.
A capability that stays skipped for a surface is a gap in that surface, and the count
belongs in the runner’s output where it can be argued about.

### API Changes

None to shipped surfaces.
The harness is additive: two shims, one Make target, one front-matter line per golden
file.

## Implementation Plan

### Phase 1

- [ ] tryscript: drop empty `path:` entries, with a test that a bare `$VAR` does not put
  the working directory on `PATH` (`fdu-nluf`)
- [ ] tryscript: add `requires:`, so named commands must resolve before the first
  session and the run reports where each resolved
- [ ] Point the corpus at a single surface directory, so nothing else supplies `fdu`
- [ ] Write the Python shim over the public `fdu` package, covering the argv the corpus
  uses; exit 77 for anything the surface cannot serve
- [ ] Write the Rust shim as an example over the public library API, with the same
  contract, and record anything it cannot express without reaching into `cli.rs`
- [ ] Add `make test-parity`, running the corpus once per surface and reporting passes,
  failures, and skips for each
- [ ] Run it, and file what it finds

## Testing Strategy

The harness is the test.
Its own correctness needs two guards, because a parity runner that passes vacuously is
worse than none:

- a deliberate divergence must fail — break one shim’s view mapping and the run goes red
- the skip count must be asserted, so a shim cannot quietly skip its way to green

## Rollout Plan

`make test-parity` stays outside `make check` until the shims are complete and the skip
list is argued, then joins it.
Keeping it out first means the corpus can be run and its findings triaged without
blocking unrelated work.

## Open Questions

- Should the Rust shim replace `examples/perf_probe.rs`’s role as the library-level
  exerciser, or stay separate?
  They have different jobs — one measures, one compares — but both exist to prove the
  library API is usable without the CLI.
- Does text parity become a tier, or stay a non-goal?
  The human renderer is where the layout rules live, and those are worth pinning across
  surfaces eventually.

## References

- [Design principles: First Principles](../../architecture/fdu-design-principles.md#first-principles)
- [Composable CLI and query surface](plan-2026-08-10-fdu-composable-cli-surface.md),
  Principle 7
- [View vocabulary and the output contract](plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
