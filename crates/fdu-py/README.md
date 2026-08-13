# fdu (Python)

Python bindings for [fdu](https://github.com/jlevy/fdu), a fast, incremental file
roll-up engine.

```python
import fdu_py

index = fdu_py.open("/path/to/tree", analyze="full", max_file_size="16MiB")
print(index.complete)         # false when unreadable paths made the result partial
print(index.freshness)        # fresh, reconciling, stale, or partial
print(index.errors)           # details from the latest open/scan/refresh
print(index.total())          # {'files': ..., 'bytes': ..., 'by_extension': {...}}
print(index.children("src"))  # one call returns every child with its roll-up
print(index.report(views=["types", "families", "documents"]))

mark = index.clock
result = index.refresh()      # reuses the original max_depth and returns error details
print(index.since(mark))      # what changed, or truncated=True if you fell behind
```

Every method is bulk: it returns a whole structured result in one call rather than a
cursor Python iterates.
Open, scan, and the native reconciliation phase of refresh run with the GIL released, so
unrelated Python threads and independent indexes can progress.
One `Index` object still has `PyO3` runtime borrow exclusion: an overlapping call on
that same object is rejected rather than becoming an unsynchronized shared-index access.
The wheel uses the core scan/cache surface and does not compile the optional watch
dependency; no Python watcher API is implied yet.

`open()` and `scan()` accept the same `none`, `basic`, `code`, `documents`, and `full`
analysis profiles as the Rust CLI, plus bounded file-size and worker settings.
The report dictionary exposes stable type/family groups, exact share fractions, line and
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
uvx --from fdu==<version> fdu --help
```

That registry command is conditional until the first release is actually on PyPI.

**Status: pre-release**, not yet published to PyPI.

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
