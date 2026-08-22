# fdu Surface Architecture

**Date:** 2026-08-22

**Author:** fdu project

**Status:** Current

## Overview

fdu answers one question — how is size distributed across a tree — through three
surfaces. This document says what each one is, which is authoritative, and how they are
kept from disagreeing.

The rules themselves are stated in [the design principles](fdu-design-principles.md)
under *One Engine, and Surfaces That Cannot Disagree With It*. This is the how.

## The Three Surfaces

| Crate or package | What it is | Who uses it |
| --- | --- | --- |
| `fdu-core` | The engine. Scanning, the index, the cache, queries, and rendering. | Rust callers, and the other two surfaces |
| `fdu` | The command line, plus a re-export of the engine | Anyone running `fdu`, and `cargo add fdu` |
| `fdu` (PyPI) | The Python binding, over the same engine | Python callers, and `uvx --from fdu fdu` |

`fdu-core` is authoritative.
The other two present it; neither may know something it does not.

### Why the packages are shaped this way

`cargo install fdu` has to work, because that is what someone who knows the tool will
type — and the previous layout answered it with cargo’s own *“there is nothing to
install … it has no binaries”*.

So `fdu` is the package a user installs.
It carries the binary and re-exports the engine, which means `cargo add fdu` also gives
a library caller the whole API: one name to know either way.
`fdu-core` exists so that the command line can depend on the engine **as an external
crate**, which is what makes the boundary real.

Two packages, and no more.
The Python binding needs the command line reachable as a library — its console script
compiles `run_process` into the extension module — which is why `fdu` is a library as
well as a binary.

## How the Boundary Is Enforced

The command line cannot reach a private item, because `crate::` does not resolve across
a crate boundary. That converts a reviewer’s diligence into a compile error.

`make lib-only` adds the dependency half:

```shell
! cargo tree -p fdu-core --all-features | grep -qE '^(clap|anyhow) '
```

The engine must not acquire the command line’s dependencies.
It had: `report_format` took its ANSI colour types from `clap::builder::styling`, so the
engine compiled an argument parser to name three colours — and rendering had to hide
behind a feature because of it, which meant a library caller could produce a report and
not print it.

## How Agreement Is Enforced

`tests/golden/*.tryscript.md` records what the command line prints for 126 sessions.
The same corpus is replayed against the Python surface through a shim
(`tests/parity/py/parity_cli.py`) that serves fdu’s argv using only the public Python
package — not a wrapper around the binary, which would test nothing.

Differences land in `tests/parity/deviations-python.diff`, committed and reviewed like
any golden. Each is matched against a named class in `scripts/parity-classes.mjs`, and
**a difference matching none fails the run**.

A class is a claim about the API and needs the same scrutiny as changing behaviour.
When one stops explaining anything it is deleted rather than kept: its matcher would
still match, so it would silently absorb the next real regression.

### What the surviving classes mean

- **Each surface names its own parameter.** There is no `--view` in Python, so its
  diagnostics name the parameter.
  Everything after the label is byte-identical.
- **Output carrying walk telemetry.** The report envelope deliberately excludes it, so a
  `Report` cannot reproduce the performance footer or a note quoting bytes read.
- **The same rule in each surface’s knob names.** `--scan-depth` against `max_depth`,
  from one constant with the names substituted.
- **Discovery surfaces.** `--docs` and `--skill` are static documents; `--version` names
  the surface deliberately, which is what keeps the artifact non-empty.

## One Behavioural Difference Worth Knowing

`fdu.report()` and `fdu.open()` are not the same contract, and the difference is
observable.

`open` retains an index and writes a snapshot: right for a caller asking many questions.
`report` runs the command line’s one-shot contract, retaining the least state the
request needs — an unfiltered summary is answered by a transient tier that retains
nothing and therefore writes no snapshot.

Using `open` for a single question caches state the walk never saved, which a later
cache-only read can see.
That was a real defect: a Python run left cache state on a tree that the same command
would not have.

## References

- [Design principles](fdu-design-principles.md) — the rules and why they are
  load-bearing
- [Python CLI parity](../specs/active/plan-2026-08-21-fdu-python-cli-parity.md) — the
  harness and what it found
- [The command line on the public API](../specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md)
  — the crate split, and why a test-only Rust shim was the wrong instrument

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
