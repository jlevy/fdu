# fdu Surface Architecture

## Overview

fdu answers directory-inventory questions through three surfaces.
This document owns their package boundaries, public responsibilities, parity contract,
and the boundary between the synchronous engine and an interactive application.

The rules themselves are stated in [the design principles](fdu-design-principles.md)
under *One Engine, and Surfaces That Cannot Disagree With It*.
[The engine architecture](fdu-engine-architecture.md) owns retained facts, commits,
serving lifecycles, paging, and shutdown.
This document explains how those engine capabilities reach users without acquiring a
second implementation.

## The Three Surfaces

| Crate or package | What it is | Who uses it |
| --- | --- | --- |
| `fdu-core` | The engine. Scanning, the index, the cache, queries, and rendering. | Rust callers, and the other two surfaces |
| `fdu` | The command line, plus a re-export of the engine | Anyone running `fdu`, and `cargo add fdu` |
| `fdu` (PyPI) | The Python binding, over the same engine | Python callers, and `uvx --from fdu fdu` |

`fdu-core` is authoritative.
The other two present it; neither may know something it does not.

The engine has two additive serving lifecycles, not two engines:

- the detached lifecycle returns a complete `Index` or `Report` after the work needed
  for that answer finishes;
- the opened lifecycle returns one shared `OpenedIndex` while cold discovery proceeds
  and exposes bounded reads, change polling, refresh, prioritization, and joined close.

Both use the same facts, reducers, exact commit path, query vocabulary, and runtime type
registry. The ownership and concurrency rules are in
[the engine architecture](fdu-engine-architecture.md).

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

`tests/golden/*.tryscript.md` records complete command-line product sessions.
The same applicable corpus is replayed against the Python surface through a shim
(`tests/parity/py/parity_cli.py`) that serves fdu’s argv using only the public Python
package — not a wrapper around the binary, which would test nothing.

Cases the Python API cannot own, such as the Rust argument parser’s help or a static
document not shipped in the wheel, are declined explicitly by name in `run-parity.mjs`.
The harness asserts that every other golden participates.
It never relies on a remembered session count, which would become stale as the corpus
grows.

Differences land in `tests/parity/deviations-python.diff`, committed and reviewed like
any golden. Each is matched against a named class in `scripts/parity-classes.mjs`, and
**a difference matching none fails the run**.

A class is a claim about the API and needs the same scrutiny as changing behaviour.
When one stops explaining anything it is deleted rather than kept: its matcher would
still match, so it would silently absorb the next real regression.

### What deviation classes mean

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

The Python package mirrors the opened lifecycle as five synchronous operations.
Calls that block or perform substantial native work release the GIL, but the package
does not add an async runtime or hide long-lived polls in Python’s shared executor.

An async application adapts that synchronous surface at its own boundary.
That adapter may own an event-loop bridge, root generations, and application cache
invalidation, but it may not become another inventory engine: it does not walk the
filesystem, retain an entry replica, rebuild roll-ups, or invent fingerprint and paging
semantics. The bridge bounds in-flight polls and native results and joins its worker on
close.

An explicitly selected adapter fails visibly when its package or API is unavailable; it
does not silently substitute another provider.
Cross-package conformance installs the exact engine revision under test rather than
using a moving branch or sibling source checkout.

The file-type registry is a value a caller supplies, not a constant the engine hides,
and that makes it a three-surface capability rather than a flag.
The engine parses the shared registry document itself and derives the identity it
reports from the parsed content, so a caller cannot assert one identity beside different
rules. Parsing it costs the standalone command no dependency: the profile has a narrow
reader in the engine rather than a general-purpose configuration library, and the
compiled default registry remains the answer when no document is supplied.

The command line does not need to expose every lifecycle immediately.
“The command line invents nothing” means a CLI capability must come from the engine; it
does not require every additive library capability to become a default flag before a
client has proven it.

## Future Considerations

### Open Questions

- Which opened-lifecycle capabilities should eventually receive an explicit CLI
  presentation, without changing the default one-shot command?
- When another language binding is justified, which parts of the parity harness can be
  replayed unchanged and which differences are intrinsic to that language?
- Which interactive clients, if any, need a stronger bridge than a bounded blocking
  change poll adapted at the application boundary?

### Potential Improvements

- Generalize the parity runner to register another public binding without copying the
  golden corpus or expected output.
- Reduce deviation classes whenever public types can carry the missing information
  directly.
- Add packaging-boundary conformance sessions for the opened lifecycle after its public
  types stabilize.

## References

- [Architecture index](README.md)
- [Design principles](fdu-design-principles.md) — the rules and why they are
  load-bearing
- [Engine architecture](fdu-engine-architecture.md) — facts, ownership, commits,
  lifecycles, reads, paging, and shutdown
- [Python CLI parity](../specs/done/plan-2026-08-21-fdu-python-cli-parity.md) — the
  harness and what it found
- [The command line on the public API](../specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md)
  — the crate split, and why a test-only Rust shim was the wrong instrument

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
