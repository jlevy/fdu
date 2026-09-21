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

Two published Rust crates, and no more; `fdu-py` is built only into the Python package.
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
behind a build feature because of it, which meant a library caller could produce a
report and not print it.

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

The shim prints through `Report.render`, which calls the same Rust renderer the command
line does. Parity therefore proves that the Python API builds the same request and takes
the same path as the command line.
It does not compare the Python models with the rendered formats, and it does not compare
a warm answer with a cold one.

Parity checks agreement after the fact.
[Model Every Key Concept Explicitly, in One Place](fdu-design-principles.md#model-every-key-concept-explicitly-in-one-place)
asks for it by construction: one request model and one answer shape in the engine, which
each surface parses into and serializes from.
The request half is built that way now: both surfaces fill one surface-neutral
`RequestSpec`, `Request::build` parses it, and each renders whatever refusal comes back
through its own `AxisNames`, so a grammar, a default, and a rule that relates one axis
to another are each stated once.
The answer half is not: several writers serialize a `Report` independently.
[Known Gaps](#known-gaps) lists where they still differ, and
[the explicit core models plan](../specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
tracks the work.

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
  from one constant with the names substituted; `--gitignore-budget` against
  `control_budget`, `--only-ignored` and `--no-gitignore` against `ignored=only` and
  `read_controls`, and the other pairs `KNOBS` in `parity-classes.mjs` elides, from
  `AxisNames`.
- **Discovery surfaces.** `--docs` and `--skill` are static documents; `--version` names
  the surface deliberately, which is what keeps the artifact non-empty.

## One Behavioural Difference Worth Knowing

`fdu.report()` and `fdu.open()` are not the same contract, and the difference is
observable.

`open` retains an index and writes a snapshot: right for a caller asking many questions.
`report` runs the command line’s one-shot contract, retaining the least state the
request needs. An unfiltered summary that turns `.gitignore` observation off is answered
by a transient tier that retains nothing and therefore writes no snapshot; the default
summary reports its ignored share, which needs the index.

Using `open` for a single question caches state the walk never saved, which a later
cache-only read can see.
That was a real defect: a Python run left cache state on a tree that the same command
would not have.

Both observe `.gitignore` by default, as the command line does, and each surface turns
it off its own way: `--no-gitignore`, `ScanOptions(read_controls=False)`, or
`ScanConfig::read_controls`. That keeps one default snapshot scope across all three, and
it is why a selection by ignored state is refused by the same rule everywhere, in each
surface’s names for the two knobs.

## Machine Output Schemas

Machine output names its schema in a `schema` field: at the top of a JSON or YAML
document, in the first record of JSON Lines, and on every watch stream record.
The version is the compatibility promise: changing a field’s name, type, or meaning
bumps it, and a golden fails when the output moves without a bump.
Each identity is a constant in the engine’s
[`report_format`](../../../crates/fdu-core/src/report_format.rs) module, so every
surface emits the same string.

| Schema | Document | Constant |
| --- | --- | --- |
| `fdu.report/7` | A report, one-shot or each one a watch run prints, over an index with no content tier and with no metric section | `REPORT_SCHEMA` |
| `fdu.report/8` | A report over an index that holds a content tier, or with a `types`, `families`, `languages`, or `documents` section | `CONTENT_REPORT_SCHEMA` |
| `fdu.stream/1` | A watch run’s `change` record, with `op` of `upsert`, `remove`, or `invalidate`: one per applied change under the `files` view, and every invalidation | `STREAM_SCHEMA` |
| `fdu.cache/2` | Cache status, a fact about the cache directory rather than about a tree, with the identity of every tier each store holds | `CACHE_SCHEMA` |

The report version follows the content tier the index holds, not the request, so a
Python `Index` opened with analysis emits `fdu.report/8` even for a tree view.
The three families version independently, so a report change never bumps the stream or
cache-status schema, or the reverse.

One `Report` reaches callers through five writers: text, JSON, and YAML in
`report_format.rs`; JSON Lines, which collapses the JSON fragments onto one line by
string replacement; and the Python models, which the public package builds by parsing
the JSON rendering. The native binding also keeps a dict writer of its own that the
public package never uses.
Change records and cache status have writers of their own.
No document yet states each envelope field by field (`fdu-c5v1`), so nothing holds the
writers to one shape; until one does, the renderers in `report_format.rs` and the
goldens under `tests/golden/` are the reference.

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

### Known Gaps

Each item is a way the surfaces fall short of
[Model Every Key Concept Explicitly, in One Place](fdu-design-principles.md#model-every-key-concept-explicitly-in-one-place)
or
[Caching Improves Performance, Never Semantics](fdu-design-principles.md#caching-improves-performance-never-semantics);
[the explicit core models plan](../specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
tracks them. Engine-side gaps are in
[the engine architecture](fdu-engine-architecture.md#known-gaps).

- **Writers disagree.** YAML flattens metric rows that JSON nests under `metrics`, and
  omits `root_raw` and `path_raw`. The unused native dict follows YAML rather than JSON
  and omits a tree node’s `kind`. JSON Lines’ `collapse` rewrites string content, so the
  path `a [ b/f { g }.txt` is emitted as `a [b/f {g}.txt`. `--watch --format yaml` emits
  JSON change records, and text output carries no source or freshness label.
- **Defaults and validation are the request model’s.** One defaults table
  (`Request::DEFAULTS`) decides the size metric, the page denominator, the analyzer set,
  and the view a report displays, and `Request::build` parses every axis from the words
  a caller wrote, so each surface supplies only its own names for them.
  `Request::validate`, `validate_read`, and `validate_delivery` state every rule once —
  a view its content cannot answer, a selection by ignored state a scope never observed,
  a read another analyzer set holds, and the three a watch cannot carry — and each route
  applies them before it reads any stored state.

### Open Questions

- Which opened-lifecycle capabilities should eventually receive an explicit CLI
  presentation, without changing the default one-shot command?
- When another language binding is justified, which parts of the parity harness can be
  replayed unchanged and which differences are intrinsic to that language?
- Which interactive clients, if any, need a stronger bridge than a bounded blocking
  change poll adapted at the application boundary?

### Potential Improvements

- Hold every writer to one field-level schema, delete the unused native dict, and
  compare the Python models with the rendered formats, so writer agreement is tested
  rather than assumed.
- Add warm-history replays to parity, so a golden also checks that a cached answer
  equals the cold one.
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

## List Selection and Presentation

The metadata default view is List, whose automatic human format is the existing bounded
directory tree. Core `ReadSpec.format` and `Query.format` resolve projection before
reading; CLI and Python forward the choice.
Flat Paths/Long and machine List requests materialize matching rows; ordinary unfiltered
Tree keeps its maintained roll-up path.
Full keeps its bounded digest.
Legacy Files preserves name ordering and legacy Tree preserves structured hierarchy
output. Explicit Paths/Long overrides Tree presentation.

Directory report rows measure eligible subtrees before name/kind/size/age selection.
Exclusions win across covered contents; nested matches remain separate rows while
aggregate totals count their union once.
Raw native entries continue to expose inode attributes.
Every flat row carries signed optional age relative to one request instant.
A detached Report owns only its requested projection: serialization cannot recover rows
folded out of a tree, so incompatible re-rendering fails instead of returning a partial
inventory as complete.
The core renderer returns a Result; Python maps it to its existing invalid-argument
boundary.

The [machine-output reference](../../machine-output.md) owns field definitions, formats,
and coverage interpretation.
The [usage guide](../../usage.md) owns command examples.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
