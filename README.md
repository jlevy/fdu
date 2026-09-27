# fdu

**Fast disk usage skill, `du` replacement, and file roll-up engine for Python and
Rust.**

On our million-entry macOS benchmark, fdu delivered **over 8× the throughput of standard
`du`**, **about 60% more than dust**, and **roughly 10% more than
[dumac](https://github.com/healeycodes/dumac#readme)** while building a reusable index
with counts, sizes, recency, and file-type tallies for every directory.
These paired results used warm filesystem caches under background load, and the tools
return different amounts of information.
See [Speed](#speed) for the measurements and limits.

Use fdu to find what takes up space, locate old build directories, or summarize a tree
without writing a filesystem walker.
The same engine serves coding agents through a self-contained skill and ships as:

- **Command line:** `fdu PATH` prints a size-sorted tree; `--watch` keeps it current
- **Python package:** typed, immutable values plus the native `fdu` command
- **Rust library:** `fdu` / `fdu-core` (a retained index, a change feed, and a
  long-lived opened root)

## Set Up with Any Coding Agent

Hand any coding agent this one instruction:

> Run `uvx --no-build fdu@latest --install-skill` from the project root to install fdu’s
> self-contained skill for current and future agent sessions.

The command writes `.agents/skills/fdu/SKILL.md` and `.claude/skills/fdu/SKILL.md` under
the project root. The skill needs no prior session context and uses `fdu` on `PATH` or a
wheel-only `uvx` fallback; no installed command or Rust toolchain is required.

Use `--agent-base DIR` to write `DIR/skills/fdu/SKILL.md` for one agent’s user scope,
such as `~/.claude`. Re-run the installer after upgrading to refresh the skill;
`fdu --skill` prints it, and deleting the generated skill directories removes it.
See the [skill usage guide](docs/usage.md#agent-skill).

## Install the Command Line

With [uv](https://docs.astral.sh/uv/), use the published wheel without a Rust toolchain:

```shell
uvx --no-build fdu@latest .
uv tool install --no-build fdu
fdu .
uv tool upgrade --no-build fdu
```

The first command runs fdu without keeping an installed command; the second keeps it on
`PATH`. `--no-build` requires a compatible wheel instead of compiling from source.
The wheels cover GIL-enabled CPython 3.12 and newer on Linux glibc (x86-64 and arm64),
macOS (x86-64 and arm64), and Windows x86-64. A Python version is not needed in normal
use: fdu declares Python 3.12 or newer and uv selects a matching interpreter.
If uv selects free-threaded CPython, such as `3.14t`, retry with `--python 3.14`; fdu
does not publish free-threaded wheels yet.
For a repeatable run, replace `latest` with a release number, such as `fdu@0.1.0`. If uv
is configured with an `exclude-newer` cool-off, a new fdu release may be filtered.
Review and allow the first-party `fdu` package in that policy, or wait for the cool-off
to expire.
`--no-config` is a one-off override that skips all uv configuration, including
that policy.

To install the command from source, use Rust 1.85 or newer:

```shell
cargo install --locked fdu
```

`--locked` keeps the reviewed dependency set; see
[SUPPLY-CHAIN-SECURITY.md](SUPPLY-CHAIN-SECURITY.md).

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
     2.6 MiB  ██████████   100%  . 144 files
     1.5 MiB  ██████░░░░    58%    crates 116 files
     827 KiB  ███░░░░░░░    31%    tests 18 files
```

The default `list` view in `tree` format shows allocated sizes, largest first, down to
depth 5, including file leaves and subtrees contributing at least 1% of the root.
Set `--depth`, `--min-share`, `--breadth`, and `--limit` to adjust independent display
bounds. It reads metadata and `.gitignore` files; it does not open regular files for
content. Hidden and ignored entries are included; ignored byte shares are annotated when
present.

| Question | Command |
| --- | --- |
| Which directories are large? | `fdu .` |
| Old build directories with size and age | `fdu . --kind dir --include node_modules --modified-before 30d --long` |
| Matching paths only | `fdu . --kind dir --include .venv --format paths` |
| Totals, excluding ignored entries | `fdu . --ignored=exclude --view=summary` |
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

`fdu --docs` is the offline guide, `fdu --help` is every flag, and `fdu --install-skill`
writes a portable skill for coding agents where they look for it (`fdu --skill` prints
it); see [Set Up with Any Coding Agent](#set-up-with-any-coding-agent).
The full grammar is in the [usage guide](docs/usage.md).

## Find Stale Build Directories

```shell
fdu ~/projects --kind dir --include .venv --modified-before 7d --long
fdu ~/projects --kind dir --include node_modules --modified-before 30d --long
fdu ~/projects --kind dir --include target --modified-before 30d --format paths
fdu ~/projects --kind dir --include .venv --include venv \
  --include node_modules --include target --modified-before 30d \
  --format long --sort mtime --reverse
```

Directory size includes eligible regular files below it; age is measured from the newest
modification of the directory or an eligible descendant.
Exclusions apply throughout the subtree.
This is modification activity, not last use.
Nested matches can overlap; aggregate views count their contents once.

`fdu PATH` prints the directory tree, and `--format tree` makes that explicit.
`--format paths` prints complete flat paths, `--long` adds size and actual age, and
JSON/JSONL/YAML provide exact metrics.
Flat lists default to size order and have no row cap; `--sort name` gives an alphabetic
inventory. Tree keeps its depth and per-directory bounds.
See [formats and directory selection](docs/usage.md#choose-a-format) and the
[machine schema](docs/machine-output.md).

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

## As a Python Module

Add the package to a uv project, or install it in the current Python environment:

```shell
uv add fdu
pip install fdu
```

The same wheel installs the native `fdu` command; there is no Python reimplementation of
the CLI.

```python
from pathlib import Path

import fdu

index = fdu.open(Path("/path/to/tree"))
print(index.status.complete)
print(index.total().files)
print(index.children("src"))

report = index.report(fdu.Query(views=(fdu.View.LANGUAGES,)))
print(report.provenance.freshness)
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
Whether a whole answer is complete and current comes from `index.status.complete` and
`report.provenance.freshness`, which the example prints.
`fdu.opened.OpenedIndex` is the typed long-lived root: coherent multi-projection reads,
continuations, and a resumable change journal.

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
use fdu::{CachePolicy, open};
use fdu::query::{Basis, Delivery, Scope};
use std::path::Path;

let basis = Basis { root: Path::new(".").into(), scope: Scope::default(), content: Default::default() };
let delivery = Delivery::new(CachePolicy::Auto, None);
let (index, report) = open(&basis, &delivery)?;
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
use fdu::{CachePolicy, open};
use fdu::query::{Basis, Delivery, Scope};
use std::path::Path;

let basis = Basis { root: Path::new(".").into(), scope: Scope::default(), content: AnalysisSet::ALL };
let delivery = Delivery::new(CachePolicy::Auto, None);
let (index, report) = open(&basis, &delivery)?;
let analyzed = index
    .content_rollup(Path::new(""))
    .map_or(0, |content| content.total.lines.analyzed_files);
println!("{} files analyzed for line metrics", analyzed);
assert!(report.analysis.is_some());
# Ok::<(), fdu::Error>(())
```

## Speed

On a million-entry tree, fdu built a reusable index and rendered a ten-row tree in **6.0
seconds**, covering **146k files/s** and **0.50 GB/s**. Measured on an M1 Pro’s internal
APFS SSD with warm filesystem caches and fdu’s cache disabled, 2026-09-26. These are
approximate local results under background load.

| Tool | Work returned | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s |
| --- | --- | ---: | ---: | ---: | ---: |
| **fdu** | reusable exact index and ten-row tree | **6.0 s** | baseline | **146k** | **0.50** |
| dumac | allocated-byte total only | 6.3 s | **+8%** | 138k | 0.47 |
| diskus | scalar total only | 8.7 s | +45% | 101k | 0.35 |
| pdu | rendered tree | 9.2 s | +53% | 95k | 0.32 |
| dust | allocated-byte total only | 9.6 s | +59% | 91k | 0.31 |
| dua | scalar total only | 9.7 s | +63% | 90k | 0.31 |
| gdu | rendered tree | 10.4 s | +64% | 84k | 0.29 |
| BSD `du` | one total, serial | 49.3 s | +717% | 18k | 0.061 |
| ncdu | reusable index | 60.6 s | +910% | 14k | 0.049 |
| GNU `du` | one total, serial | 62.1 s | +955% | 14k | 0.048 |

Positive percentages mean extra elapsed time: +60% means 1.6× as long as the adjacent
fdu run. Rates count regular files and their disk space, not file-content reads; `k`
means thousands and GB is decimal.
See the
[full comparison](docs/project/reports/report-2026-09-26-fdu-live-tool-comparison.md)
for methodology, memory use, confidence intervals, and exact results.

On Linux the ranking depends on the job.
The same million-entry tree on a 4-vCPU virtualized ext4 host, same harness, 2026-09-27:

| Tool | Work returned | Median wall-clock time | Wall time vs. fdu summary |
| --- | --- | ---: | ---: |
| **fdu `--no-gitignore --view summary`** | exact totals, no index | **0.97 s** | baseline |
| pdu | rendered tree | 1.09 s | +11% |
| diskus | scalar total only | 1.10 s | +16% |
| fdu | reusable exact index and ten-row tree | 1.38 s | — |
| dust | allocated-byte total only | 1.70 s | +77% |
| gdu, GNU `du`, ncdu, dua | tree or total | 2.6–4.0 s | +166% to +299% |

fdu’s summary mode is the fastest tool measured, but building the reusable index is
about 19% slower than pdu or diskus there.
That gap is glibc allocator contention between fdu’s walker threads and its index
builder, not filesystem work; see the
[Linux comparison](docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md)
for the evidence and the work under way.
Both tables measure fdu with its cache disabled, which is also what the default `fdu .`
now does for a one-shot report: it used to write a snapshot that no later `fdu .` reads,
about a fifth of the run on this Linux tree.
The summary still needs `--no-gitignore`, because reading ignore rules falls back to the
full index. The
[cache economics brief](docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md)
covers when the cache and the index pay on each platform.
Windows builds and passes tests but has not been performance-benchmarked.

### Multi-View Reports

Report construction has a separate result.
In an exploratory, uncontrolled macOS benchmark on a 137,085-entry tree, a loop that
constructed the unfiltered Types, Families, Languages, and Documents views 100 times
from an already line-analyzed index took 12.0 seconds, down from 29.9 seconds—about
**2.5× faster**, or roughly 120 ms instead of 299 ms per report.
The code is retained provisionally; a quiet run must still resolve the inconclusive
major-fault gate before the experiment is accepted.

This is not a scan or end-to-end full-analysis speedup.
It applies only to unfiltered requests with multiple metric views; the default
disk-usage command and single-view analysis are unchanged.
The implementation is platform-neutral Rust, but its Linux magnitude has not yet been
measured. See
[the experiment](docs/project/experiments/exp-159-share-content-metric-resolution-across-views.md)
for the paired interval, host regime, and resource qualification.

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

During 0.x, a minor release may change the command line or either API. See the
[release process](docs/project/guides/release-process.md).

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
