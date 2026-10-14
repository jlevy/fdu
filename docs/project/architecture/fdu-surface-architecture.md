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

The engine has two additive serving lifecycles:

- The batch API (`Index`) waits for the requested scan, reconciliation, and analysis
  work before returning an owned native index.
  Its status records any incomplete coverage.
  Repeated queries and manual refresh operate on that retained value.
- The progressive/live API (`OpenedIndex`) returns shared native state while cold
  discovery proceeds and exposes bounded reads, change polling, refresh, prioritization,
  optional observation workers, and joined close.

One-shot `report()` prepares an answer without promising to retain an index.
Query-bound watch (`Watch`) starts an independent live session from a copy of a batch
index. Iteration applies verified updates and drives persistence; `Watch.report()` reads
only the latest applied state and does not drain pending events or update the original
`Index`. The Rust CLI’s `--watch` uses the same `watch_session::Session`.

These paths use the same facts, reducers, mutation rules, query vocabulary, and runtime
type registry. The ownership and concurrency rules are in
[the engine architecture](fdu-engine-architecture.md).

### Why the Packages Are Shaped This Way

`fdu` is the package a user installs with `cargo install fdu`. It carries the binary and
re-exports the engine, which means `cargo add fdu` also gives a library caller the whole
API: one name to know either way.
`fdu-core` exists so that the command line can depend on the engine *as an external
crate*, which is what makes the boundary real.

Two published Rust crates, and no more; `fdu-py` is built only into the Python package.
The Python binding needs the command line reachable as a library: its console script
compiles `run_process` into the extension module, so `fdu` is a library as well as a
binary.

## How the Boundary Is Enforced

The command line cannot reach a private item, because `crate::` does not resolve across
a crate boundary. That converts a reviewer’s diligence into a compile error.

`make lib-only` adds the dependency half:

```shell
tree="$(cargo tree -p fdu-core --all-features --prefix none)" || exit 1
! printf '%s\n' "$tree" | grep -qE '^(clap|anyhow) '
```

`--prefix none` is required here: without it `cargo tree` indents dependencies with
`├── `, so `^clap` matches nothing and the check can never fire.
Capturing the dependency tree before testing its contents preserves a failing
`cargo tree` status.
A pipeline’s status is its last command’s, so piping that failure straight into `grep`
would report success having checked nothing.

The engine must not acquire the command line’s dependencies, including argument parsing
or its styling types.
Rendering is an engine capability available to library callers.

## How Agreement Is Enforced

`tests/golden/*.tryscript.md` records complete command-line product sessions.
The same applicable corpus is replayed against the Python surface through a shim
(`tests/parity/py/parity_cli.py`) that serves fdu’s argv using only the public Python
package. A wrapper around the binary would not exercise the Python API.

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
The request half follows that model: both surfaces fill one surface-neutral
`RequestSpec`, `Request::build` parses it, and each renders whatever refusal comes back
through its own `AxisNames`, so a grammar, a default, and a rule that relates one axis
to another are each stated once.
The answer uses one ordered value traversal for JSON, JSON Lines, and YAML, and the
Python models decode that same report shape.
Parser-backed conformance tests compare all machine formats, including adversarial
strings and lossless raw paths.
[The explicit core models plan](../specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
records the model and its acceptance checks.

Differences land in `tests/parity/deviations-python.diff`, committed and reviewed like
any golden.
Each is matched against a named class in `scripts/parity-classes.mjs`, and *a
difference matching none fails the run*.

A class is a claim about the API and needs the same scrutiny as changing behaviour.
When one stops explaining anything it is deleted rather than kept: its matcher would
still match, so it would silently absorb the next real regression.

### What Deviation Classes Mean

- **Each surface names its own parameter.** There is no `--view` in Python, so its
  diagnostics name the parameter: `tip: show it: --view code` is
  `tip: show it: view code`. Everything after the label is byte-identical.
- **Bound and analysis tips name the same setter.** The one bound tip lifts every bound
  that hid something, as flags on one command line (`--depth=all --limit=all`) or as one
  call’s keyword arguments (`depth=all, limit=all`), and `full`’s omitted-views tip is
  `tip: include them: --analyze all` or `analyze all`; the matchers accept exactly those
  translations.
- **Output carrying walk telemetry.** The report envelope deliberately excludes it, so a
  `Report` cannot reproduce the performance footer or a note quoting bytes read.
- **The same rule in each surface’s knob names.** `--scan-depth` against `max_depth`,
  from one constant with the names substituted; `--gitignore-budget` against
  `control_budget`, `--ignored=only` and `--no-gitignore` against `ignored=only` and
  `read_controls`, and the other pairs `KNOBS` in `parity-classes.mjs` elides, from
  `AxisNames`.
- **Discovery surfaces.** `--docs` and `--skill` are static documents, and
  `--install-skill` writes the second to disk; `--version` names the surface
  deliberately, which is what keeps the artifact non-empty.

## One Behavioural Difference Worth Knowing

`fdu.report()` and `fdu.open()` are not the same contract, and the difference is
observable.

`open` retains an index and participates in caching under its delivery policy: useful
for a caller asking many questions.
Snapshot writes depend on cache policy, completeness, and the work performed.
`report` runs the command line’s one-shot contract, retaining the least state the
request needs.
Under the default `CachePolicy.AUTO` a metadata report writes no snapshot,
and an unfiltered summary is answered by a transient tier that retains no index,
classifying each entry against `.gitignore` as it counts it when observation is on.

Choosing `open` for a single question can retain and cache state that a one-shot report
would not retain, which a later `stale_ok` read can see.
Use `report` for the command line’s one-shot policy and `open` when a retained index is
needed.

The two also differ on content views, for the same reason.
`report` builds its own basis, so `Query(views=(View.CODE,))` runs the code analyzer,
exactly as `--view code` does on the command line.
An index holds the analyzers it was opened with for every read it serves, so
`index.report` never adds one: the same query on an index opened without `code` raises
`InvalidArgumentError` naming the analyzer.
The rule is the request model’s (`Request::build` against `Request::read`), not either
surface’s, so a Rust caller meets the same split.

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
A version no release has emitted yet is a draft that may still change in place, as
[the release process](../guides/release-process.md) states.
Each identity is a constant in the engine’s
[`report_format`](../../../crates/fdu-core/src/report_format.rs) module, so every
surface emits the same string.

| Schema | Document | Constant |
| --- | --- | --- |
| `fdu.report/11` | A report over one root or several, including its request, status, per-tier provenance, and any requested metric units; over several roots, `roots` beside a null `root`, a tree `total` and `trees`, and a `root` position on rows, errors, and refusals | `REPORT_SCHEMA` |
| `fdu.stream/2` | A watch run’s `change` record, with `op` of `upsert`, `remove`, or `invalidate`: one per applied change under the `files` view, and every invalidation | `STREAM_SCHEMA` |
| `fdu.cache/3` | Cache status, a fact about the cache directory rather than about a tree, with the identity of every tier each store holds | `CACHE_SCHEMA` |

The three families version independently, so a report change never bumps the stream or
cache-status schema, or the reverse.

One iterative field walk in `report_format.rs` feeds JSON, JSON Lines, and YAML sinks;
the command line writes those sinks directly to its output stream.
Python builds its public models by parsing the same JSON rendering.
Change records and cache status have writers of their own.
The declared field presence rules, renderers, and goldens under `tests/golden/` hold the
wire shape together.

## Python Result Ownership

Python is both a distribution surface and a typed library over the native engine.
The standalone Rust executable requires no Python runtime; the PyPI console script uses
a small Python launcher into the same native command-line implementation.

The Python APIs serve different caller lifecycles:

| API | Use it for | Ownership and current behavior |
| --- | --- | --- |
| One-shot `fdu.report()` | One summary or selected report | Waits for the requested work and returns an eager selected report; retains only the native state that execution needs |
| Batch API: `fdu.open()` / `fdu.scan()` returning `Index` | Requested content analysis, repeated snapshot queries, manual refresh, or selected bulk results | Waits for the requested work; retains an owned Rust index with explicit coverage; `open` applies cache policy |
| Progressive/live API: `fdu.opened.OpenedIndex.open()` | Interactive discovery, bounded pages, coherent reads, and live changes | Shared Rust state and owned discovery/optional observation workers; current delivery is cache-off and has no content analysis |
| Query-bound watch: `Index.watch()` returning `Watch` | Repeating one metadata query and consuming selection-filtered changes | Independent native copy, fixed query and report/persistence policy; iteration drives updates and persistence, and the original `Index` is unchanged |

“Batch” describes when the operation returns, not an export of the whole inventory.
The `Index` handle keeps its inventory in Rust; queries convert only requested answers.
Manual refresh can mutate it, and incomplete scans remain visible in its status.
A watch’s relative-time selection windows stay fixed after query construction, while
report ages use the instant of each report call.
Its report reflects the latest state applied by iteration, not pending events.

Selection, aggregation, ordering, and reconciliation remain engine operations.
`OpenedIndex` owns ongoing discovery and optional observation: Python submits control
and query requests, reads selected projections, and polls changes.
Per-file traversal and maintenance do not require Python callbacks or exported metadata
objects. The boundary is coarse operations and requested results, rather than one
crossing per indexed file.
A summary or bounded page can therefore keep Python conversion small even when the
native inventory is large; native discovery and query costs still belong to the engine.
Exact change consumption under discovery or heavy churn can return more information, so
the abstraction does not promise low result volume for every workload.
The ordinary usage pattern should request summaries, selected rows, bounded pages, or
incremental changes rather than build an application-owned inventory replica.
Complete results remain supported when the caller needs them; their allocation cost must
not be hidden by silent truncation.

Public reports currently construct a complete JSON rendering of the selected answer,
decode dictionaries and lists, and build typed Python models eagerly.
The returned report retains both the wire graph and an immutable native rendering
handle. `as_dict()` additionally copies its nested containers.
Scalar values can be shared, so these representations do not imply a fixed memory
multiplier. Tree and flat opened pages also allocate dictionaries, lists, and typed
Python values. Neither path is a zero-copy export.
Python `Report.render(JSONL)` returns a complete string.

Paged enumeration can bound client retention when consumed pages are released; retaining
every page accumulates Python memory.
Page row and work caps are operation bounds, not a byte-memory guarantee, and
`ReportProjection` has its own work allowance rather than the page row cap.
Continuations depend on the current engine version and do not preserve a historical
inventory for complete export.

The [Python and Rust review](../reviews/review-2026-10-13-python-rust-boundary.md)
records current conversion costs, integration tradeoffs, and measurement gaps.
A supported render-only native handle is proposed there; it is not a current public API.

### Current Copy and Sharing Behavior

There is no public zero-copy inventory, page, or report-data export today.
Moving an owned Rust value into a Python handle or sharing native state through `Arc`
avoids an inventory transfer, but does not expose its storage as a Python buffer.

| Path | Native ownership and copies | Python allocation |
| --- | --- | --- |
| `open()` / `scan()` | Move the owned Rust index into its handle | A handle, without one Python metadata object per file |
| `OpenedIndex` ownership | Shared native authority; native handle clones share state | A native handle, without an exported inventory |
| Opened Tree/Flat/Lookup | Construct owned requested rows and path/attribute values, not borrowed views of live entries | Dictionaries/lists and typed models; ordinary models do not retain the whole temporary conversion graph |
| Aggregate/state | Owned small answers and envelopes | Small dictionaries and models; no per-inventory-row export |
| Reports | Own the selected projection; opened conversion additionally clones the selected report | Rust JSON string converted to Python string, decoded containers, and models; wire graph and native report remain retained |
| `Report.as_dict()` | No further native report clone | Copies every nested dictionary/list container; immutable keys and scalar values can remain shared |
| Changes | Copy matching retained commits into an owned response | Materialized change dictionaries and typed batches, not views into the journal |
| Query-bound `Index.watch()` / `Watch.report()` | Clone the inventory into an independent session; each report snapshots that inventory before projection; due persistence can snapshot it too | Watch creation returns a handle; its reports then use the eager report pipeline |

The serialized report string is an intermediate, not another permanently retained
representation. Ordinary pages release their raw conversion containers after decoding,
while `Report` deliberately keeps its wire graph.
A native inventory clone copies owned entries and collections but can still share `Arc`
resources. These differences prevent a fixed memory multiplier or a claim that every
returned value is copied byte for byte.

The implementation boundaries are `PyIndex` and `PyOneShot` in
[`lib.rs`](../../../crates/fdu-py/src/lib.rs), page/report conversion in
[`opened_binding.rs`](../../../crates/fdu-py/src/opened_binding.rs), and `_copy_json`
and `Report` in [`_models.py`](../../../crates/fdu-py/python/fdu/_models.py).
Query-bound watch snapshots are explicit in `Session::report` and
`IndexHandle::snapshot`.

Future improvements are tracked independently: native/lazy report ownership
(`fdu-5ve4`), query-bound watch snapshots (`fdu-xhge`), optional bulk buffers or
columnar export (`fdu-g20s`), and public-boundary measurements (`fdu-88o1`). Owned
page/change conversion can also be simplified after measuring its temporary containers
(`fdu-1yxw`). A packed immutable batch may copy data once from the index and then share
its finished buffers with Python without another data copy.
That would make that handoff zero-copy, not the entire query.
Any such API needs explicit lifetime after refresh/close, release ownership,
retained-memory accounting, and lossless path/version/coverage semantics.
It must not keep a live index read lock for an arbitrary Python consumer lifetime.

### Rationale and Binding Practice

A coarse language boundary amortizes call overhead and avoids creating Python objects
for facts an application never consumes.
Keeping selection and aggregation beside the native inventory also gives them the
engine’s ordering, coverage, and version semantics without rebuilding those rules in
Python. Zero-copy export is one possible optimization for consumers that need bulk data;
avoiding an unnecessary export is the ordinary goal.

For example, an opened root holding 200,000 files can answer a maintained aggregate or a
100-row page without constructing 200,000 Python metadata objects.
This describes result conversion, not a guarantee that Rust examines only 100 candidates
or finishes within a fixed time.
Projection and selection determine native preparation cost.

When adding or changing a Python capability:

1. Put inventory work and its semantic request/result types in `fdu-core`. Python
   validates and submits a coarse request; it does not walk files, invoke a callback per
   entry, or reconstruct roll-ups from exported rows.
2. Keep retained state behind a native handle.
   Repeated batch queries can use `Index`; progressive and live clients can use
   `OpenedIndex` projections and state/change polling.
   A caller repeating a fixed metadata query can use `Watch` and consume its iterator to
   apply updates. Handle reset and version invalidation explicitly.
3. Return the information the caller requested: aggregates, selected rows, bounded
   pages, or changes. Release consumed pages when complete enumeration need not remain in
   Python. Do not confuse a row cap with a byte cap or a work estimate with a deadline.
4. Preserve explicit complete-result paths for callers that need Python-side processing.
   State their materialization and retention costs; do not silently truncate an
   enumeration. With the current eager row/model representation, larger results create
   and retain more Python objects.
5. Separate native work from Python-object construction.
   Release the GIL while native computation and blocking waits can run without Python
   objects, preserve borrowing and lock-order rules, and then convert the selected
   answer. Current report exceptions are recorded in the boundary review; synchronous
   calls still need application-owned adaptation when used from an event loop.
6. Verify native/Python answer agreement and installed-wheel behavior.
   Benchmark the complete public call, including conversion and teardown, with inventory
   size and result size varied independently.
   CLI throughput alone cannot validate this boundary.

These rules guide the convenient path; they do not require every caller to minimize
allocations or prohibit complete exports.
The costs should follow a deliberate request.

## Interactive Client Boundary

The Python package mirrors the opened lifecycle with synchronous operations.
Native batch scanning/reconciliation, opened operations, and watch waits release the
GIL. Query-bound watch startup, some report preparation and rendering, and Python result
construction currently hold it.
The package does not add an async runtime or hide long-lived polls in Python’s shared
executor.

An async application adapts that synchronous surface at its own boundary.
That adapter may own an event-loop bridge, root generations, and application cache
invalidation, but it may not become another inventory engine: it does not walk the
filesystem, retain an entry replica, rebuild roll-ups, or invent fingerprint and paging
semantics. The bridge bounds in-flight polls and native results and joins its worker on
close. Cancelling an executor future does not itself cancel native work.
Closing an `OpenedIndex` shuts down its shared root for all aliases, rather than one
request.

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

- Add warm-history replays to parity, so a golden also checks that a cached answer
  equals the cold one.
- Generalize the parity runner to register another public binding without copying the
  golden corpus or expected output.
- Reduce deviation classes whenever public types can carry the missing information
  directly.
- Expand cross-surface lifecycle conformance beyond the installed-wheel smoke tests that
  already cover opened projections, continuations, refresh, history, and shared close.

## References

- [Architecture index](README.md)
- [Design principles](fdu-design-principles.md): the rules and their rationale
- [Engine architecture](fdu-engine-architecture.md): facts, ownership, commits,
  lifecycles, reads, paging, and shutdown
- [Python CLI parity](../specs/done/plan-2026-08-21-fdu-python-cli-parity.md): the
  harness and what it found
- [The command line on the public API](../specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md):
  the crate split and the test-shim decision

## List Selection and Presentation

The metadata default view is List, whose automatic human format is the existing bounded
directory tree. Core `ReadSpec.format` and `Query.format` resolve projection before
reading; CLI and Python forward the choice.
Flat Paths/Long and machine List requests materialize matching rows; ordinary unfiltered
Tree keeps its maintained roll-up path.
Full keeps its bounded digest.
The Files preset uses name ordering and the Tree preset presents structured hierarchy
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
