# fdu (Python)

Python bindings for [fdu](https://github.com/jlevy/fdu), a fast, incremental file
roll-up engine.

The public package is `fdu`; `fdu._native` is private build machinery.
The supported API includes typed query and scan options, immutable report sections,
roll-ups, per-path provenance, cache management, refresh results, and change feeds.
Filesystem failures that prevent an operation from starting remain exceptions.
Errors that make a scan partial remain structured data on `Status`, so callers can use
the covered result without losing the reason it is incomplete.
The
[roll-up adapter example](https://github.com/jlevy/fdu/blob/main/crates/fdu-py/examples/rollup_adapter.py)
shows one scan serving several application-owned summaries without parsing terminal
output or adopting fdu’s machine schema as the application’s internal model.

```python
from pathlib import Path

import fdu

index = fdu.open(
    Path("/path/to/tree"),
    cache=fdu.CachePolicy.AUTO,
    scan=fdu.ScanOptions(one_filesystem=True),
    analysis=fdu.AnalysisOptions(profile=fdu.AnalysisProfile.FULL),
)
print(index.status.complete)
print(index.status.freshness)
print(index.total().by_extension)

report = index.report(
    fdu.Query(
        views=(fdu.View.TYPES, fdu.View.FAMILIES, fdu.View.DOCUMENTS),
        selection=fdu.Selection(limit=20, size=fdu.SizeMetric.APPARENT),
    )
)
print(report.sections)
print(report.as_dict())

mark = index.clock
result = index.refresh()
print(result.status, index.since(mark).changes)
```

Every method is bulk: it returns a whole structured result in one call rather than a
cursor Python iterates.
Open, scan, and the native reconciliation phase of refresh run with the GIL released, so
unrelated Python threads and independent indexes can progress.
Content analysis streams every eligible file through EOF. Binary data, invalid UTF-8,
and unsupported SLOC languages remain visible as coverage without making the operation
partial; I/O failures and files changed during a read remain operational errors.
One `Index` serves concurrent readers, including while a refresh is running: the
reconciliation takes the engine’s write lock per wave rather than for the whole sweep,
so a reader is served between waves instead of rejected for the duration.
A server can hold one index and answer requests from a thread pool while it updates.

The wheel enables the optional watch dependency and exposes `Index.watch()` as a
closable, event-driven change feed.
A watch is thread-affine: it belongs to the thread that opened it and is not shareable,
which the binding enforces rather than merely documents.
For an asyncio server, `fdu.aio.watch_batches(index, options)` is the handoff -- a
worker thread that opens the watch, drains it, and closes it, yielding the same typed
batches on the event loop with real backpressure.
A live UI should set `WatchOptions.interval` near its frame budget: the interval bounds
how long one pull blocks before returning empty-handed, not how quickly a change is
seen. `WatchOptions.poll_interval` selects periodic restat instead, for network and FUSE
mounts that accept a native watch and then deliver nothing.
`examples/sse_resume.py` maps `Index.since(clock)` and `ChangeSet.truncated` onto
Server-Sent Events `Last-Event-ID` resume, including the branch that matters: a client
further behind than the journal can replay must resync rather than be sent an incomplete
set it has no way to detect.
Content analysis itself remains one-shot: refresh reanalyzes after metadata
reconciliation, while a watch feed reports metadata changes.

`AnalysisProfile` covers the same `none`, `basic`, `code`, `documents`, and `full`
profiles as the Rust CLI, and `AnalysisOptions` carries worker concurrency.
Typed report sections expose stable type/family groups, exact share fractions, line and
word slots, page denominators, coverage outcomes, analyzer provenance, detection source
and confidence, and generated/vendor/documentation flags.
The original extension grouping remains available as the `extensions` view.
The package supports Python 3.12 and newer and builds one `abi3-py312` extension rather
than separate native payloads for every Python minor release.

The wheel also exposes the native Rust CLI as the `fdu` console script.
Argument parsing, help, streams, color, errors, broken-pipe handling, and exit status
all use the same Rust process boundary as the Cargo-installed binary; there is no Python
CLI reimplementation.
`make python-smoke` installs the built wheel into an isolated environment and runs both
the module contract and a direct local-wheel `uvx` check.

After publication, an exact reviewed release can run without a persistent install:

```shell
uvx fdu@<version> --help
```

That registry command is conditional until the first release is actually on PyPI.

**Status: pre-release**, not yet published to PyPI.

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
