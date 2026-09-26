# fdu (Python)

Python bindings for [fdu](https://github.com/jlevy/fdu), a fast, incremental file
roll-up engine.

## Install

```shell
uvx --no-build --python 3.12 fdu@latest .
uv tool install --no-build --python 3.12 fdu
uv add fdu
pip install fdu
```

The first command runs fdu once; the second keeps it on `PATH`.
`uv tool upgrade --no-build fdu` updates that install; `uvx fdu@<version> --help` runs
one exact release. For coding agents, run
`uvx --no-build --python 3.12 fdu@latest --install-skill` from the target project.
It writes the agent skill without installing the command.
The skill uses `fdu` on `PATH` when present and otherwise `uvx fdu@latest`;
`fdu --skill` prints it.
The [repository README](https://github.com/jlevy/fdu#install) has the details.
Prebuilt `abi3` wheels cover GIL-enabled CPython 3.12 and newer on Linux glibc (x86-64
and arm64), macOS (x86-64 and arm64), and Windows x86-64, so installing needs no Rust
toolchain. Free-threaded CPython, such as `3.14t`, cannot install them and is not
supported; an installer there may fall back to building the source distribution.
`--no-build` makes uv fail instead of compiling when no compatible wheel exists.
If uv selects a free-threaded interpreter, pass `--python 3.14` (or `--python 3.12`). An
`exclude-newer` policy in uv can filter a newly published fdu release.
Review and allow the first-party `fdu` package in that policy, or wait for its cool-off
to expire.

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

## Directory Inventories and Formats

A default `Query()` keeps the existing directory tree.
Select a flat presentation when requesting a report, so an ordinary tree need not
materialize a complete inventory:

```python
from pathlib import Path

import fdu

index = fdu.open(Path("/path/to/tree"))
for name in (".venv", "venv", "node_modules", "target"):
    report = index.report(
        fdu.Query(
            format=fdu.Format.LONG,
            selection=fdu.Selection(
                kinds=(fdu.EntryKind.DIR,),
                include=(name,),
                modified_before="30d",
                sort=fdu.SortKey.MTIME,
                reverse=True,
            ),
        )
    )
    print(report.render())
    print(report.render(fdu.Format.JSON))
```

All four reads use one retained index without rescanning.
`View.LIST` is the metadata default.
`Format.TREE` explicitly requests the current tree; `PATHS` lists matching paths, `LONG`
adds size and signed modification age, and JSON/JSONL/YAML carry exact metrics.
`TEXT` selects automatic human presentation.
Legacy `View.FILES` keeps flat name ordering, and `View.TREE` preserves structured
directory output. Largest/recent remain regular-file presets, with optional Paths/Long
output.

Directory `FileRow` values carry subtree `bytes`, `allocated`, `files`, `dirs`,
`complete`, `mtime_ns`, and `age_ns`; non-directory counts are `None`. A directory whose
subtree was not listed in full has `complete=False`, lower-bound sizes, and
`age_ns=None`. The fixed `Report.age_reference_ns` explains age, including negative
future ages and pre-epoch mtime.
An unrepresentable reference yields `None` age.
Exclusions win throughout the subtree; nested roots may overlap, while grouped totals
count their union once.
Age describes modification, not access or last use.
Native entry/lookup projections retain inode attributes; these report rows deliberately
carry subtree metrics instead.

A Report owns its requested projection.
Re-render it to another serialization or between Paths and Long without querying again;
request another report to change between a bounded tree and complete flat inventory.
An incompatible conversion raises `InvalidArgumentError` rather than silently listing
only visible tree rows.
Tree limits remain per-directory, flat limits apply to the whole list, and machine List
output is complete unless explicitly limited.
Details and exact fields are in the
[usage guide](https://github.com/jlevy/fdu/blob/main/docs/usage.md) and
[machine-output reference](https://github.com/jlevy/fdu/blob/main/docs/machine-output.md).

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
[the repository README](https://github.com/jlevy/fdu#development).

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
