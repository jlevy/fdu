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

The engine has two additive serving lifecycles, not two engines:

- the existing one-shot lifecycle returns a complete `Index` or `Report` after the work
  needed for that answer finishes;
- the target opened-root lifecycle returns one shared `OpenedIndex` while cold discovery
  proceeds and exposes bounded reads, change polling, refresh, prioritization, and
  joined close.

Both use the same facts, reducers, exact commit path, query vocabulary, and runtime type
registry. The opened-root ownership and concurrency rules are in
[the opened-root architecture](arch-2026-08-25-fdu-opened-root.md).

### Why the packages are shaped this way

`cargo install fdu` has to work, because that is what someone who knows the tool will
type — and the previous layout answered it with cargo’s own *“there is nothing to
install … it has no binaries”*.

So `fdu` is the package a user installs.
It carries the binary and re-exports the engine, which means `cargo add fdu` also gives
a library caller the whole API: one name to know either way.
`fdu-core` exists so that the command line can depend on the engine **as an external
crate**, which is what makes the boundary real.

Two Rust crates, and no more.
The Python binding needs the command line reachable as a library — its console script
compiles `run_process` into the extension module — which is why `fdu` is a library as
well as a binary.

## How the Boundary Is Enforced

The command line cannot reach a private item, because `crate::` does not resolve across
a crate boundary. That converts a reviewer’s diligence into a compile error.

`make lib-only` adds the dependency half:

```shell
tree="$(cargo tree -p fdu-core --all-features --prefix none)" || exit 1
! printf '%s\n' "$tree" | grep -qE '^(clap|anyhow) '
```

`--prefix none` is load-bearing: without it `cargo tree` indents dependencies with
`├── `, so `^clap` matches nothing and the check can never fire.
Capturing before testing is load-bearing for the same kind of reason — a pipeline’s
status is its last command’s, so piping a failing `cargo tree` straight into `grep`
would report success having checked nothing.

The engine must not acquire the command line’s dependencies.
It had: `report_format` took its ANSI colour types from `clap::builder::styling`, so the
engine compiled an argument parser to name three colours — and rendering had to hide
behind a feature because of it, which meant a library caller could produce a report and
not print it.

## How Agreement Is Enforced

`tests/golden/*.tryscript.md` records what the command line prints for 129 sessions.
The same corpus is replayed against the Python surface through a shim
(`tests/parity/py/parity_cli.py`) that serves fdu’s argv using only the public Python
package — not a wrapper around the binary, which would test nothing.

Parity compares 126 of those 129. Three are declined by name in `run-parity.mjs`,
because they render clap’s own help and usage errors or a static document the package
does not carry. The two numbers are different on purpose, and a report quoting one where
it means the other is the kind of drift this document exists to prevent.

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

## Interactive Client Boundary

The Python package mirrors the opened-root engine as five synchronous operations.
Calls that block or perform substantial native work release the GIL, but the package
does not add an async runtime or hide long-lived polls in Python’s shared executor.

An async application adapts that synchronous surface at its own boundary.
That adapter may own an event-loop bridge, root generations, and application cache
invalidation, but it may not become another inventory engine: it does not walk the
filesystem, retain an entry replica, rebuild roll-ups, or invent fingerprint and paging
semantics.

The command line does not need to expose every lifecycle immediately.
“The command line invents nothing” means a CLI capability must come from the engine; it
does not require every additive library capability to become a default flag before a
client has proven it.

## References

- [Design principles](fdu-design-principles.md) — the rules and why they are
  load-bearing
- [Opened-root architecture](arch-2026-08-25-fdu-opened-root.md) — the target live
  owner, commit, read, journal, and client boundaries
- [Python CLI parity](../specs/done/plan-2026-08-21-fdu-python-cli-parity.md) — the
  harness and what it found
- [The command line on the public API](../specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md)
  — the crate split, and why a test-only Rust shim was the wrong instrument

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
