# Python and Rust Architecture Review — 2026-10-13

## Verdict

fdu’s Rust engine lets callers avoid transferring the full inventory into Python when
they need only aggregates or pages.
Scanning, analysis, reconciliation, ordering, and roll-ups belong to the engine.
Python supplies requests and application policy.

The Python boundary still has material costs.
Public reports eagerly serialize their selected result to JSON, decode dictionaries and
lists, and construct typed Python models.
They retain the decoded wire representation and a native report as well.
Opened tree and flat pages also allocate Python objects.
There is currently no public zero-copy inventory export, and rendering a Python report
as JSON Lines returns a complete string rather than an incremental iterator.

The early user’s concern is therefore useful, particularly for complete inventories and
repeated large reports.
Their allocation microbenchmark does not establish that Python overhead dominates fdu’s
filesystem work, causes severe garbage-collection pauses, or creates GIL contention.
Those are workload-dependent claims requiring measurements through the installed Python
API.

**Recommended direction:** keep PyO3 and the Rust-owned engine; improve the report
boundary and its measurements before considering a different binding technology or
columnar export.

## Scope and Evidence

This review combines source inspection with an independent technical and adversarial
review by a `gpt-6-astra` sub-agent at `xhigh` reasoning.
The inspected revision is
[`f39bf134`](https://github.com/jlevy/fdu/tree/f39bf13466fe2a4a433f09307cc1f5bfdf395e9c).
The Python binding, core, and benchmark sources are unchanged at the documentation
branch’s base, `a8f13e54`.

The governing contracts are the
[surface architecture](../architecture/fdu-surface-architecture.md),
[engine architecture](../architecture/fdu-engine-architecture.md), and
[design principles](../architecture/fdu-design-principles.md).
This is an assessment of those contracts and their implementation, not a new engine
design or a measured performance verdict.

No builds, runtime reproductions, or benchmarks were run for this review.
The required external scratch volume was full and refused creation of the task
directory. Findings below identify confirmed source behavior; predicted failures and
performance effects are labeled separately.
Existing tests are evidence of intended coverage, not new test results.

## What Crosses the Boundary Today

### Package and engine ownership

[`fdu-py/Cargo.toml`](../../../crates/fdu-py/Cargo.toml) builds a PyO3 0.29 `cdylib`,
using `abi3-py312`. [Maturin packaging](../../../crates/fdu-py/pyproject.toml) installs
the native module as `fdu._native` beside the public Python models and API. The package
requires Python 3.12 or later.

`fdu-core` owns the engine; the command-line crate and Python binding consume it.
PyO3 is confined to the binding crate.
The Python wheel’s console entry calls the actual Rust command-line implementation
through the extension module, so invoking the wheel’s `fdu` command does not exercise
Python report-model construction.
See [`main`](../../../crates/fdu-py/src/lib.rs) and the
[surface architecture](../architecture/fdu-surface-architecture.md#the-three-surfaces).

Stable-ABI packaging reduces the number of interpreter-version wheels needed per
platform. It does not remove object conversion costs.
fdu’s current wheel configuration targets GIL-enabled CPython; the Makefile explicitly
refuses free-threaded interpreters for its wheel test environments.
Broader free-threaded support would require its own packaging and concurrency
validation.
[PyO3 distribution documentation](https://pyo3.rs/v0.29.0/building-and-distribution.html)
explains the stable-ABI tradeoff.

### Inventory ownership and result ownership are different

| Operation | Native ownership and work | Python result |
| --- | --- | --- |
| `fdu.open()` / `fdu.scan()` | A retained Rust `Index`; native opening runs detached from Python | A handle, not one Python object per indexed file |
| `Index.total()` / `rollup()` | Rust roll-ups | Dictionaries converted into typed values; extension tallies can grow with cardinality |
| `Index.children()` | Direct children of one directory | All children materialized; this legacy operation has no page argument |
| `fdu.report()` | One-shot engine contract; retains only the state that execution needs | An eagerly materialized selected report |
| `Index.report()` / `Watch.report()` | A finished native report from retained state | The same eager report conversion |
| `OpenedIndex.read()` with `Tree` or `Flat` projections | Coherent native projections with page and work bounds | Dictionaries/lists followed by Python models for the returned pages |
| `OpenedIndex.read()` with an `Aggregate` projection | Maintained aggregate where supported, otherwise capped work | A small typed answer with coverage and version information |
| Opened `ReportProjection` | Native report subject to work allowance and query selection | Eager report conversion, including a native report clone |
| Change polling | Native journal and version semantics | Materialized returned commits or changes |

The detached `Index`, progressively serving `OpenedIndex`, and one-shot report have
different lifetime contracts.
`fdu.open()` and `OpenedIndex.open()` should not be treated as interchangeable.
The engine architecture owns their discovery, persistence, and shutdown semantics.

### Reports have an eager conversion pipeline

The ordinary report producers in [`_api.py`](../../../crates/fdu-py/python/fdu/_api.py)
follow this sequence:

```text
Rust scan or retained query
  -> finished native Report
  -> complete JSON String, converted to Python str
  -> decoded Python dict/list graph
  -> frozen, slotted Python models
```

[`Report`](../../../crates/fdu-py/python/fdu/_models.py) keeps the models, the `_wire`
dictionary, and a renderer closure holding the native report handle.
The JSON string is an intermediate allocation; it need not remain live after decoding.
`Report.as_dict()` makes an independent copy of the nested wire containers.
Some scalar objects are shared, so this establishes multiple representations without
establishing a fixed memory multiplier.

The native handle serves a correctness purpose: every rendering describes the same
immutable answer, even after the index changes.
The wire graph preserves the exact machine schema.
The Python models provide typed access and lossless path handling.
Those are useful properties, but eager creation of all three is unnecessary for a caller
that only wants to forward rendered text.

In particular, `fdu.report(...).render(...)` already paid the initial JSON and model
conversion before `render` begins.
Choosing JSON Lines as the final format does not make this path stream.
The opened report path in
[`opened_binding.rs`](../../../crates/fdu-py/src/opened_binding.rs) additionally clones
the native report while converting the borrowed response.

Costs grow with the selected answer.
A summary can stay small even over millions of files; a complete files report can
produce one Python row per selected file.
`Files` intentionally enumerates completely unless the caller supplies a limit.
Silently changing that contract to return twenty rows would lose data for inventory and
incremental-sync callers.
See `ViewSpec::default_limit` in
[`query_report.rs`](../../../crates/fdu-core/src/query/query_report.rs).

## Choosing an API by Use Case

Convenient Python should encourage efficient native use while preserving deliberate
access to complete results.
The API should explain each cost rather than prohibit the more expensive choice or
silently change the answer.

| Use case | Current approach and cost |
| --- | --- |
| Standalone command or shell pipeline | Use the Rust executable; no Python runtime or Python row objects |
| PyPI-installed command | A small Python launcher invokes the same Rust CLI; no Python report-model conversion |
| One summary or selected report | `fdu.report()` runs native work, then materializes the selected answer in Python |
| Repeated questions over one tree | Retain a native index and run selection and aggregation there; convert only requested answers |
| Interactive pages and live changes | Use `OpenedIndex` reads and change polls; release consumed pages and handle version invalidation |
| Complete inventory for Python processing | Explicitly request complete results and accept materialization, or consume and release pages with coherent-version handling |
| Render and forward a large answer | Current Python reports still build models first; a supported native render-only path is a proposed improvement |

## Assessing the User’s Argument

The strongest claim is that fast traversal does not guarantee fast delivery of a large
Python object graph.
That is correct. A files-per-second CLI result and end-to-end Python agent latency
measure different operations.
Filesystem cache state, result cardinality, path lengths, selection, and how long the
caller retains answers can change which stage dominates.

The supplied function times a list comprehension containing 200,000 flat dictionaries.
It exercises Python allocation and iteration.
It does not cross PyO3, deserialize JSON, scan files, build fdu’s typed models, measure
another thread’s progress, or run an agent loop.
Literal keys and the fixed numeric values can be reused; real metadata and paths have
different distributions.
It also omits result destruction from the reported interval.

`gc.collect()` before the timer does not attribute later elapsed time to collection.
CPython’s cyclic collector supplements reference counting, and dictionaries containing
only atomic keys and values can be untracked.
The example therefore does not establish severe cyclic-GC pauses.
Allocation, reference counting, and teardown still cost CPU; fdu’s nested containers and
models have a different tracking profile.
[Python GC documentation](https://docs.python.org/3.12/library/gc.html) describes these
distinctions.

GIL contention likewise requires competing Python threads.
A single-thread allocation loop does not measure contention.
Separately, a synchronous API can block an asyncio event-loop thread even when its
native work releases the GIL: releasing the interpreter does not suspend the calling
coroutine.

The correct response is to acknowledge allocation costs, describe which answers fdu
exports, and measure the actual client workload.
Neither “Rust makes it free” nor “200,000 dictionaries prove the wrapper is slower than
disk” is supported.

## Adversarial Findings

### A1 — High scalability exposure: eager report representations

All public report producers eagerly create the wire graph and typed models and retain
the native report. Large complete reports and repeated retained reports can therefore
accumulate native and Python memory together.
Slotted dataclasses reduce instance overhead but still allocate objects.

**Improvement:** expose a supported render-only operation or immutable native report
handle, with model and wire conversion requested explicitly or performed lazily.
Preserve snapshot identity, shared-schema parity, and the independent-copy contract of
`as_dict()`. Move an owned opened report into its handle instead of cloning it.
This is a structural optimization proposal; its latency and memory benefit remain
unmeasured. Tracking: `fdu-5ve4`.

### A2 — Medium responsiveness exposure: native report work holds the GIL

The binding detaches scans, opening, refresh, watch waits, and opened engine reads.
However, the legacy `Index.report_handle` and watch report construction,
`PyOneShot::render`, and the one-shot `pending.join()` execute while attached.
The opened report branch serializes and clones after its detached engine read has
returned.

On GIL-enabled CPython, these native phases can exclude other Python threads while they
run. Subsequent dictionary and model construction is also interpreter work.
Existing detach coverage cannot support a claim that the entire public operation allows
other Python threads to progress.

**Improvement:** detach native construction, serialization, and blocking joins while
preserving PyO3 borrowing and native lock ordering.
Reattach for Python-object work.
Add concurrency regressions at the actual report boundaries; retain explicit application
executor ownership for async use.
[PyO3 parallelism documentation](https://pyo3.rs/v0.29.0/parallelism.html) explains what
detachment provides.
Tracking: `fdu-rlqg`.

### A3 — Medium correctness gap: opened reports bypass deep-tree decoding

The ordinary report path calls `_loads_object`, whose decoder retries iteratively on
`RecursionError`. The opened report branch calls standard-library `json.loads` directly.
The existing [`deep-report test`](../../../crates/fdu-py/tests/test_models.py) covers a
4,000-level synthetic report through the ordinary helper.

**Expected failure from source, not runtime-reproduced here:** a sufficiently deep
opened tree report can raise `RecursionError` during decoding even when the report fits
the native work allowance.
The precise depth depends on the interpreter and current call stack.
Unlimited tree depth, breadth, and limit with zero minimum share expose the path.
A portable regression should use a synthetic native report or a lowered recursion limit
in an isolated process, rather than depend on filesystem path limits.

**Improvement:** use the same deep-tree-capable decoder for both lifecycles, or remove
JSON parsing from the native boundary.
Equivalent opened and detached reports must decode to the same shape.
Remove the report clone as well: `TreeNode` cloning is recursively derived, whereas
destruction already has an iterative implementation.
Increasing Python’s recursion limit does not address that native recursion.
Native stack exhaustion was not reproduced.
Tracking: `fdu-v80c`.

### A4 — Medium evidence gap: CLI and native probes omit Python conversion

`python-open-query` exists in the benchmark schema vocabulary, but the review found no
corresponding implemented scenario or recorded run.
Native probes in
[`realtree/measure.py`](../../../explorations/benchmarks/realtree/measure.py) cover
retained queries, opened reports, and rendering.
They do not cover Python dictionaries, models, paths, or retained wire graphs.
Wheel CLI timings include startup but call the Rust CLI directly.

**Improvement:** benchmark the installed public Python API before publishing claims
about Python throughput, heap cost, GC pauses, or responsiveness.
Tracking: `fdu-88o1`.

### Bounded pages do not establish bounded total memory or constant query cost

[`Page`](../../../crates/fdu-py/python/fdu/opened.py) defaults to 256 rows and 100,000
work units. The [core contract](../../../crates/fdu-core/src/engine_contract.rs) caps a
page at 4,096 rows, a page work allowance at 1,000,000, and a coherent read at 16
projections. Sixteen page projections can therefore return up to 65,536 rows, plus
envelopes. Variable-length paths and nested metadata mean a row cap is not a byte cap.

`ReportProjection` does not inherit the page row cap.
Its [`report_projection`](../../../crates/fdu-core/src/opened/read.rs) checks a work
estimate and then builds the selected report.
Work units are neither a wall-clock deadline nor a heap-allocation budget.

A caller holding every page can still retain the entire inventory in Python.
The Rust index grows with inventory size.
Optional `max_files` restricts retained regular files; it does not impose a general
entry-count or byte budget.
Journal and continuation limits constrain their own native stores, not all client
memory.

Small output does not imply equally small native preparation.
`root_file_rows` collects candidates, sorts them, and then truncates.
Some selections require subtree passes.
Conversely, maintained aggregate counts and portable-name paging have efficient native
paths. The appropriate complexity statement depends on the projection and selection.

Continuations bind to an engine version, are consumable and evictable, and can become
invalid after discovery or refresh commits.
They do not provide a durable snapshot stream across a changing tree.
A selective flat read can exhaust its work allowance and return `Limit` instead of a
resumable partial page.
Applications must handle refusals, version changes, and bounded retries explicitly.

Expensive projections can also delay commits while holding the native read boundary.
Cancelling a Python executor future does not by itself cancel native work; finite poll
timeouts, explicit close, and joined worker ownership remain necessary.
`OpenedIndex.close()` shuts down the shared root for all aliases; it is not cancellation
of one request.

### Design documents were reconciled with the implementation

The governing documents and Python guide now state eager result ownership, distinguish
display limits from operation limits, and qualify GIL-release and query-cost claims.
The engine document now describes the actual first-unvisited flat cursor and
single-frame tree continuation, work-limit outcomes for aggregates, and the distinction
between a recent-files report and a dedicated recency page.
The format principle now respects a report’s retained projection.
Existing installed-wheel opened tests are acknowledged.

The binding module’s statement that methods return whole results without cursors is too
broad now that opened continuations exist.
The opened Python class’s statement that all substantial native work releases the GIL
omits report conversion.
Those source-comment corrections remain with their implementation follow-ups so comments
and behavior stay consistent.
The dated review records evidence; the architecture documents remain the governing
contracts.

## Integration Choices and Tradeoffs

Binding technology and result representation are independent decisions.
PyO3 can expose a native handle, objects, bytes, or buffers.
A C ABI can still allocate every Python dictionary.
Changing the call mechanism alone does not remove materialization.

| Approach | Benefits | Costs and fit for fdu |
| --- | --- | --- |
| PyO3 with eager Python models | Convenient typed API, familiar values, current schema parity | Per-row allocation and JSON round-trip; suitable for modest selected answers |
| PyO3 with a lazy native report handle | Avoids unused models and wire graphs; preserves one immutable answer | Native result remains live; accessed values still need conversion; preferred first improvement |
| Direct native classes or direct model construction | Can remove JSON and intermediate dictionaries | Each object still allocates; per-field calls can add overhead; schema mapping needs conformance coverage |
| Bounded native batches/pages | Amortizes calls and bounds each returned group; largely available today | Total consumption still costs per row; retaining pages and live-version invalidation require care |
| Raw text/bytes or chunked writers | Avoids row models for render-and-forward clients | One returned string still buffers the whole result; actual streaming needs backpressure, cancellation, and error semantics |
| Buffer protocol or NumPy columns | Few Python objects for numeric data; efficient bulk computation | Requires a compatible export layout, explicit lifetimes, nullability, and separate variable-length path storage |
| Arrow record batches | Columnar analytics and interoperability; compatible buffers can be shared | Export packing still costs work; schema and dependency burden; useful when consumers stay columnar |
| C ABI with `ctypes` or CFFI | Reusable boundary for several languages | Manual ABI, errors, ownership, and versioning; no automatic reduction in Python heap allocations |
| Subprocess JSON Lines | Process isolation and an explicit stream; reuses CLI behavior | Startup, encoding, pipes, and decoding; collecting all stdout or rows defeats streaming |
| Service or shared memory | Can share an inventory among multiple processes | Additional lifecycle, transport, access-control, stale-state, and cleanup contracts; justify with a multi-client need |
| Pure Python | Simpler implementation and deployment for small workloads | Metadata objects still allocate; gives up existing native ownership, reducers, and traversal advantages |

For buffers, **zero-copy** should name the exact boundary where a copy is avoided.
fdu’s Rust tree and owned paths are not automatically a contiguous table that Python can
borrow. A numeric export might need packing once into an owned buffer; Python can then
view it without copying.
Returning `bytes` or using `memoryview` is not, by itself, proof that no earlier copy
occurred. Exported storage must outlive every view and remain valid through refresh and
close. Holding a native index lock for arbitrary consumer lifetimes would undermine
serving and shutdown.
[Python’s buffer protocol](https://docs.python.org/3.12/c-api/buffer.html) specifies the
exporter and release relationship.

Arrow’s [C Data Interface](https://arrow.apache.org/docs/format/CDataInterface.html)
shares compatible columnar buffers within a process; its
[C Stream Interface](https://arrow.apache.org/docs/format/CStreamInterface.html)
supports batches.
Building those columns from current fdu structures still requires work.
A schema must preserve raw paths, timestamp units, missing values, coverage, version
identity, and ordering.
Converting the resulting table back into Python row dictionaries would reintroduce the
object cost.

[CFFI](https://cffi.readthedocs.io/en/latest/overview.html) supports compiled API and
ABI modes; [ctypes](https://docs.python.org/3.12/library/ctypes.html) calls C-compatible
libraries. Either would require designing a C-facing Rust contract rather than exposing
Rust layouts directly.
That investment makes sense for cross-language consumers, not as a remedy for an
unchanged Python result graph.

For a [subprocess](https://docs.python.org/3.12/library/subprocess.html), distinguish
incremental pipe consumption from `communicate()` or capture-all usage.
Process isolation can keep native work away from the application’s interpreter, but
parsing each line into Python objects still incurs allocation.
Streaming is primarily a peak-memory and backpressure property, not a promise of zero
conversion CPU.

## Measurements Needed Before Further Optimization

Run through an installed wheel whose native payload is pinned.
Follow the [performance loop](../guides/performance-loop.md) for paired comparisons,
regimes, recording, and publication.
Correctness and parity tests remain separate from timing evidence.

Vary **inventory size** and **returned result size** independently.
Include 200,000 files as a useful test point, without treating it as a universal
crossover threshold.

| Workload | Question it answers |
| --- | --- |
| Cold process/import and cold scan | What does the first public Python answer cost? |
| Filesystem-warm scan versus retained repeated queries | How much of an agent loop avoids traversal? |
| Summary, default tree, and complete files | How does selected output cardinality affect conversion? |
| Native handle, serialization, decoding, models, `as_dict()`, rendering | Which stage consumes time and memory? |
| Wide directories, deep trees, long and non-UTF-8 paths | Do bounds, decoding, and path preservation hold for difficult shapes? |
| Paged consumption with immediate dropping versus retained pages | Does peak memory follow the consumption policy? |
| Selective reads and updates during pagination | What do work refusal and continuation invalidation cost? |
| Repeated reports, forwarding, and teardown | What stays live, and what does reclamation cost? |

Record wall and CPU time, returned rows and bytes, peak and retained process memory,
Python allocation accounting, GC callback durations, and progress of a competing Python
thread. Measure event-loop delay separately with the application’s intended executor
adapter. `tracemalloc` misses native allocations; RSS alone cannot attribute them.
Instrumentation itself needs a baseline because allocation tracing changes timings.

Expose native timing boundaries only through benchmark instrumentation or a deliberate
public diagnostic contract; a private-handle benchmark alone does not establish public
API performance. Validate identical answers, snapshot versions, and lossless paths.
Record interpreter, build profile, platform, filesystem cache state, and artifact
identity. Report distributions for repeated loops and include destruction, rather than
one construction-only elapsed value.

## Recommended Work Order

1. Fix the deep-tree decoding inconsistency with a cross-lifecycle regression
   (`fdu-v80c`).
2. Detach avoidable native report work and joins; remove the opened report clone while
   preserving ownership and concurrency rules (`fdu-rlqg`, `fdu-5ve4`).
3. Add the installed-wheel boundary measurements alongside API design work (`fdu-88o1`).
4. Provide a supported render-only/native snapshot path, preserving eager values for
   callers who request them (`fdu-5ve4`).
5. Use measurements to decide whether direct models, export byte budgets, top-k query
   preparation, or columnar batches merit their additional contracts.

For present-day agent loops, request the summaries and pages actually needed, keep
aggregation and selection in Rust, and release previous results promptly.
Applications materializing a complete report should budget for its full Python
representation. Paged enumeration can bound client retention when consumed pages are
released, but must handle continuation invalidation.
Clients forwarding rendered output would benefit most directly from the proposed
render-only path.

## Suggested Reply to Early Users

Good point: traversal speed and the cost of constructing a Python result are different
measurements. The standalone CLI is Rust and needs no Python runtime.
The PyPI command uses a small Python launcher into the same native CLI.

Python also provides a library API. Traversal, analysis, observation, retained indexes,
selection, and aggregation stay in Rust.
Callers can request summaries, bounded pages, or incremental changes without
constructing a Python object for every indexed file.
Complete inventories remain available when needed, with their associated conversion
cost; paged consumption can keep client retention bounded when pages are released and
version changes are handled.

The current Python report API eagerly converts its selected answer through JSON and
typed models, so it is not zero-copy.
We’re tracking a render-only path, remaining GIL gaps, and installed-wheel benchmarks.
The aim is efficient Rust operations with a convenient Python interface, while keeping
complete materialization an explicit choice.
Your example illustrates allocation work; actual GC pauses and FFI costs need separate
measurement.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
