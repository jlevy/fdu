# fdu (Python)

Python bindings for [fdu](https://github.com/jlevy/fdu), a fast, incremental file
roll-up engine.

## Install

```shell
uv tool install fdu   # put the fdu command on your PATH
uvx fdu .             # or run the command once without installing it
uv add fdu            # use the library in a uv project
pip install fdu       # or install the library with pip
```

`uvx fdu@<version> --help` runs one exact release.
Prebuilt `abi3` wheels cover GIL-enabled CPython 3.12 and newer on Linux glibc (x86-64
and arm64), macOS (x86-64 and arm64), and Windows x86-64, so installing needs no Rust
toolchain. Free-threaded CPython, such as `3.14t`, cannot install them and is not
supported; an installer there falls back to building the source distribution.
If uv selects a free-threaded interpreter, pass `--python 3.14` (or `--python 3.12`), as
in `uv tool install --python 3.14 fdu`.

## Use

The public package is `fdu`; `fdu._native` is private build machinery.
The supported API includes typed query and scan options, immutable report sections,
roll-ups, per-path provenance, cache management, refresh results, and change feeds.
`.gitignore` is read by default: tree, summary, extension, and file rows carry their
ignored share, `Selection(ignored=...)` selects one side, and `Status.ignore_rules`
names any file the limits refused.
`ScanOptions` carries the switch (`read_controls`) and both limits (`control_budget`,
`control_line_limit`). Cache status is typed as `CacheState`, with `StaleReason` and
`LeftoverKind`, and `clear_all_caches` returns a `ClearSummary`. Filesystem failures
that prevent an operation from starting remain exceptions.
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
    analysis=fdu.AnalysisOptions(analyze=fdu.Analysis.ALL),
)
print(index.status.complete)
print(index.total().by_extension)

report = index.report(
    fdu.Query(
        views=(fdu.View.TYPES, fdu.View.FAMILIES, fdu.View.DOCUMENTS),
        selection=fdu.Selection(limit=20, size=fdu.SizeMetric.APPARENT),
    )
)
print(report.provenance.freshness)
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
One `Index` object still has `PyO3` runtime borrow exclusion: an overlapping call on
that same object is rejected rather than becoming an unsynchronized shared-index access.
The wheel enables the optional watch dependency and exposes `Index.watch()` as a
closable, event-driven change feed.
Content analysis itself remains one-shot: refresh reanalyzes after metadata
reconciliation, while a watch feed reports metadata changes.

`Analysis` names the same analyzers as the command line’s `--analyze`: `lines`, `code`,
and `words`, with `none` and `all` as totals.
`AnalysisOptions` takes one of them or a comma-separated set such as `"code,words"`,
plus a worker count.
Typed report sections expose stable type/family groups, exact share fractions, line and
word slots, page denominators, coverage outcomes, analyzer provenance, detection source
and confidence, and generated/vendor/documentation flags.
The original extension grouping remains available as the `extensions` view.
The package supports Python 3.12 and newer and builds one `abi3-py312` extension rather
than separate native payloads for every Python minor release.

## Long-lived roots

`fdu.opened` is the direct typed interface to the long-lived engine.
It starts progressive discovery, returns several projections from one coherent version,
exposes exact resumable changes, verifies explicit path sets, accepts discovery
priorities, and joins all native work on close:

```python
from pathlib import Path

from fdu import opened

with opened.OpenedIndex.open(Path("."), opened.OpenedOptions(observe=True)) as index:
    answer = index.read(
        opened.Tree(page=opened.Page(limit=200, max_work=100_000)),
        opened.Diagnostics(),
    )
    cursor = answer.change_cursor
    changed = index.changes(cursor, timeout=1.0)
```

The methods are synchronous and release the GIL during native work.
An async application adapts them with its own executor policy, keeping task and shutdown
ownership visible. The package does not run a private event loop or shell out to the
command line.

The wheel also exposes the native Rust CLI as the `fdu` console script.
Argument parsing, help, streams, color, errors, broken-pipe handling, and exit status
all use the same Rust process boundary as the Cargo-installed binary; there is no Python
CLI reimplementation.

**Status: 0.x.** A new minor release may change the Python API;
[the release process](https://github.com/jlevy/fdu/blob/main/docs/project/guides/release-process.md)
states the compatibility rules.
Building and testing the package from a checkout is covered in
[the repository README](https://github.com/jlevy/fdu#install).

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
