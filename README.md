# fdu

**Fast, incremental file roll-up engine:** `fd` and `du`, read as “fast du”.

One walk over a directory tree answers, for every directory at once, how big it is, how
many files it holds, what changed most recently, and what kinds of files it contains.
The index is cached between runs and can be kept live as the tree changes.

The same engine ships three ways:

- **Command line:** `fdu PATH` prints a size-sorted tree; `--watch` keeps it current
- **Rust library:** `fdu` / `fdu-core` (a retained index, a change feed, and a
  long-lived opened root)
- **Python package:** typed, immutable values plus the native `fdu` command

On a 2026-09-16 macOS calibration, fdu built a reusable exact index and a ten-row tree
over 1,000,001 generated entries in a **5.206-second median**. The same paired run:
dumac **+11.3%**, diskus **+34.7%**, dust **+60.6%**, dua **+63.1%**, BSD `du`
**+898%**. The host was loaded; pairing is what makes those comparisons fair.
See [Speed](#speed).

**0.x:** A minor release may change the command line or either API;
[the release process](docs/project/guides/release-process.md) states the rules.

## Install

Once `0.1.0` is on the registries:

```shell
cargo install --locked fdu          # command line; Rust 1.85 or newer
uv tool install fdu                 # same command, prebuilt wheel
uvx fdu@0.1.0 --help                # run that release without installing
uv add fdu                          # Python library in a uv project
pip install fdu==0.1.0              # Python library in the current environment
cargo add fdu                       # Rust library (re-exports the engine)
cargo add fdu-core --features watch # engine only, with the watch layer
```

`--locked` keeps the reviewed dependency set; see
[SUPPLY-CHAIN-SECURITY.md](SUPPLY-CHAIN-SECURITY.md).

Wheels are `abi3` for GIL-enabled CPython 3.12 and newer.
Free-threaded CPython cannot load them; pass a standard interpreter (`--python 3.12` or
`--python 3.14`).

From a source checkout:

```shell
git clone https://github.com/jlevy/fdu.git
cd fdu
cargo install --locked --path crates/fdu
```

## Command Line

fdu requires a path.
Use `.` for the current directory:

```console
$ fdu .
     2.6 MiB  ██████████   100%  . (144 files)
     1.5 MiB  ██████░░░░    58%    crates (116 files)
     827 KiB  ███░░░░░░░    31%    tests (18 files)
```

That is the `tree` view: allocated sizes, largest first, two directory levels, up to ten
children per directory.
It reads metadata and `.gitignore` files; it does not open regular files for content.
Hidden and ignored entries are included; ignored byte shares are annotated when present.

| Question | Command |
| --- | --- |
| Which directories are large? | `fdu .` |
| Totals, excluding ignored entries | `fdu . --exclude-ignored --view=summary` |
| Languages by space | `fdu . --view=languages` |
| Ten files that changed most recently | `fdu . --view=recent --limit=10` |
| Standard lines of code | `fdu . --analyze=code` |
| Keep the tree live | `fdu . --watch` |
| Machine output | `fdu . --format=json` |

`--view` chooses what is reported; several views share one walk.
`--analyze` is the only switch that reads file bodies.
A view never turns analysis on.
Exit status 0 is a complete result, 1 a failure, and 2 a partial result or a usage
error.

`fdu --docs` is the offline guide, `fdu --help` is every flag, and `fdu --skill` prints
a portable skill for coding agents.
The full grammar is in the [usage guide](docs/usage.md).

## Live Updates

`--watch` is the same query, re-evaluated as the tree changes.
Detection uses the platform’s native event backend (`FSEvents`, inotify,
`ReadDirectoryChangesW`); an idle tree is not polled.
Each hint is verified with a fresh stat before it becomes a delta.

```shell
fdu . --watch
fdu . --watch --view=files --format=jsonl
```

`--interval` throttles how often a text view repaints, not how changes are detected.
Content analysis is one-shot and cannot be combined with `--watch`.

Library callers get the same feed without parsing the command: Rust `Session` (behind
the `watch` build feature) and Python `Index.watch()`. Long-lived interactive clients
use `OpenedIndex` / `fdu.opened` for progressive discovery, paged reads, and a resumable
journal.

## As a Rust Library

```shell
cargo add fdu
```

`fdu` re-exports the engine.
The default `watch` build feature adds the OS-native watch layer;
`cargo add fdu --no-default-features` leaves it out.
An embedding that wants none of the command line’s dependencies depends on `fdu-core`
instead.

```rust
use fdu::{OpenConfig, open};
use std::path::Path;

let (index, report) = open(Path::new("."), &OpenConfig::default())?;
let total = index.total();
println!("{} files, {} bytes", total.files, total.bytes);

// Per-directory roll-ups are already computed; this is not another walk.
if let Some(src) = index.rollup(Path::new("src")) {
    println!("src/: {} files, newest {}", src.files, src.newest_mtime_ns);
}
# Ok::<(), fdu::Error>(())
```

`open` returns a retained index.
Later questions reuse it; `refresh` reconciles against the tree; with `watch` enabled,
`fdu::session::Session` answers the same request as events arrive.
`OpenedIndex` is the long-lived owner for progressive discovery.

Opt into content analysis explicitly; metadata-only is the default:

```rust
use fdu::content::AnalysisSet;
use fdu::{OpenConfig, open};
use std::path::Path;

let mut config = OpenConfig::default();
config.analysis.profile = AnalysisSet::ALL;
let (index, report) = open(Path::new("."), &config)?;
let analyzed = index
    .content_rollup(Path::new(""))
    .map_or(0, |content| content.total.analyzed_files);
println!("{} analyzed files", analyzed);
assert!(report.analysis.is_some());
# Ok::<(), fdu::Error>(())
```

## As a Python Module

```python
from pathlib import Path

import fdu

index = fdu.open(Path("/path/to/tree"))
print(index.status.complete, index.status.freshness)
print(index.total().files)
print(index.children("src"))

report = index.report(fdu.Query(views=(fdu.View.LANGUAGES,)))
print(report.as_dict())  # same JSON the command line emits

mark = index.clock
index.refresh()
print(index.since(mark).changes)
```

The watch feed is a live iterator; it does not return:

```python
from pathlib import Path

import fdu

index = fdu.open(Path("/path/to/tree"))
with index.watch() as stream:
    for batch in stream:
        for change in batch:
            print(change.kind, change.path)
```

Values are frozen dataclasses and enums.
Every method is bulk: it returns a whole structured result in one call.
Open, scan, and the native reconciliation phase of refresh run with the GIL released;
building the Python dicts and lists holds it.
Provenance on a roll-up is the entry’s own source, not its subtree: a revalidated
directory can hold cached descendants.
Whether a whole answer is complete and current comes from the scan’s `complete` and
`freshness`, which the example prints.
`fdu.opened.OpenedIndex` is the typed long-lived root: coherent multi-projection reads,
continuations, and a resumable change journal.

The wheel also installs the native `fdu` command.
There is no Python reimplementation of the CLI.

## Speed

On an older MacBook, it can tally file sizes at roughly 200K files/sec and analyze lines
of code at roughly 4M lines/sec after caching.
[The 2026-09-18 installed-CLI QA](docs/project/reports/report-2026-09-18-cli-installed-qa.md)
has the log.

**Exploratory macOS calibration, 2026-09-16, 0.1.0 release candidate.** A fresh process
with its cache disabled built a reusable exact index and ten-row tree over a generated
1,000,001-entry corpus in a **5.206-second median**. Twelve adjacent paired trials per
tool on an M1 Pro with a local APFS SSD, warm filesystem cache, one independent
full-tree fingerprint.
The host was busy (load 7.7–9.9 on ten cores).
The absolute seconds are a loaded-host number; the paired percentages are the
comparison.

| Tool | Work returned | Median | Versus paired fdu |
| --- | --- | ---: | ---: |
| **fdu** | reusable exact index and ten-row tree | **5.206 s** | baseline |
| dumac | allocated-byte total only | 5.637 s | **+11.3%** |
| diskus | scalar total only | 6.972 s | +34.7% |
| dust | allocated-byte total only | 8.292 s | +60.6% |
| dua | scalar total only | 8.744 s | +63.1% |
| BSD `du` | one total, serial | 51.226 s | +898% |
| GNU `du` | one total, serial | 65.775 s | +1177% |

Each competitor was reduced to one number.
fdu returned counts, apparent and allocated bytes, newest file time, per-directory and
per-extension roll-ups, and kept the index that answers the next question without
another walk. dumac’s 95% interval was +5.8% to +13.5%.

fdu’s peak RSS here was 285.4 MiB against dumac’s 29.4 MiB, because fdu retained a
million-entry index and dumac retained one integer.
`fdu --no-gitignore --view summary` keeps the aggregate-only tier, which returns the
same tallies without retaining that index: **15.0 MiB** against 285.4 MiB for the tree
view in a separate fdu-only round-robin.
That tier buys memory, not time.

Linux evidence is real and improving, from virtualized hosts.
The most recent campaign on a 450k-entry tree, measured against its own starting point:
warm snapshot load **−31.4%**, warm revalidate **−25.3%**, cold indexed scan **−9.1%**.
A warm open now runs about 23% faster than a cold scan, where that campaign began with
it 69% *slower*. Windows builds and passes tests; no performance claim is made there.

A second run on an unchanged tree is a different job.
Metadata-only one-shots still revalidate; `--analyze` reuses unchanged file-body results
from a content sidecar.
The trustworthy floor for a warm metadata run is still one stat per entry: directory
mtimes do not record in-place edits.

[The full comparison](docs/project/reports/report-2026-09-16-fdu-live-tool-comparison.md)
has the method and the limits.
[The performance campaign status](docs/project/reports/report-2026-08-14-performance-campaign-status.md)
is the place to start on the evidence as a whole.

## Why

Of a dozen surveyed tools in this space ([du](https://www.gnu.org/software/coreutils/),
[ncdu](https://dev.yorhel.nl/ncdu), [dust](https://github.com/bootandy/dust),
[dua](https://github.com/Byron/dua-cli), [gdu](https://github.com/dundee/gdu),
[dut](https://codeberg.org/201984/dut), [duc](https://github.com/zevv/duc),
[fsearch](https://github.com/cboxdoerfer/fsearch),
[bfs](https://github.com/tavianator/bfs), [fd](https://github.com/sharkdp/fd),
[scc](https://github.com/boyter/scc), [tokei](https://github.com/XAMPPRocky/tokei)),
exactly one persists anything, exactly one carries multiple metrics per pass, **none**
does per-directory type tallies, and **none** does mtime-based incremental revalidation.
None of them is a native library with a live change feed that a Rust or Python program
can hold. That combination is what a live file browser actually needs.

The survey is in
[the file roll-up engine research](docs/project/research/research-2026-08-06-file-rollup-engine.md).

## Documentation

- [Usage guide](docs/usage.md): views, analyzers, selection, cache, watch, machine
  output
- [Documentation index](docs/README.md): library, architecture, performance, release
- [Design principles](docs/project/architecture/fdu-design-principles.md)
- [0.1.0 release notes](docs/project/release-notes/0.1.0.md)
- [Changelog](CHANGELOG.md)

## Development

```shell
make check    # handoff gate: fmt, clippy, tests, docs, lib-only build
make test     # Rust tests plus the CLI golden contract
make fix      # formatting and machine-applicable lint fixes
```

Permission and native watch tests fail when the host cannot establish their operating
system preconditions.
A deliberately unsupported local host may set `FDU_TEST_ALLOW_NO_PERMISSION_BITS=1` or
`FDU_TEST_ALLOW_NO_NATIVE_WATCH=1` for the affected test selection.
CI must leave both variables unset so a passing test proves its assertions ran.

[AGENTS.md](AGENTS.md) is how to operate on the repository.
[The supply-chain policy](SUPPLY-CHAIN-SECURITY.md) applies before any dependency
change. Performance work follows
[the performance loop](docs/project/guides/performance-loop.md) and is deliberately
outside `make check`.

## License

MIT. See [LICENSE](LICENSE).

Designs adapted from GPL-licensed tools ([dut](https://codeberg.org/201984/dut)’s
atomic-refcount roll-up, [fsearch](https://github.com/cboxdoerfer/fsearch)’s record
layout) are clean reimplementations written from the descriptions in the research doc,
not transliterated from their source.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
