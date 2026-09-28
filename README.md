# fdu

**Fastest du replacement and file tree analysis for 100+GB, million-file worktrees**

Use fdu to find what takes up space, locate old build directories, or summarize a tree
without writing a filesystem walker.

Key features:

- **Speed:** Native Rust and native filesystem APIs make fdu fast.
  On a one-million-file macOS benchmark, fdu ran at about 9× the speed of standard `du`,
  over 50% faster than [dust](https://github.com/bootandy/dust), 49% faster than
  [pdu](https://github.com/KSXGitHub/parallel-disk-usage), and about 9% faster than
  [dumac](https://github.com/healeycodes/dumac#readme), the next-fastest tool, which
  returns only a total.
  On Linux, fdu’s summary mode beats pdu and
  [diskus](https://github.com/sharkdp/diskus), the fastest peers there, but building its
  full index takes about 20% longer than they do, and on source trees with many
  `.gitignore` files its default command is still several times slower than pdu.
  See [Speed](#speed) for the measurements, the limits, and the work under way.
- **Text, file, and code analysis:** Rolls up content metrics, including lines, source
  code lines by language, and words, paragraphs, and pages for Markdown and text.
- **Cached statistics:** Content metrics require reading files, so fdu caches them
  between runs and reads again only the files that changed.
- **Watch and stream events:** Unlike `du` or dust, fdu can keep a result current and
  stream its changes, using each platform’s native file watching (FSEvents, inotify,
  `ReadDirectoryChangesW`).
- **Rust and Python APIs:** Every capability is available directly from Rust and Python:
  typed results, a retained index, a change feed, and a long-lived opened root, all
  backed by the same native engine as the command line.
- **Easy use as a skill or from the command line:** Nothing needs to be built from
  source. Prebuilt binaries install from PyPI on macOS, Linux, and Windows, and the Rust
  crates (`fdu` and `fdu-core`) are on crates.io.
  Run `uvx fdu@latest` anywhere [uv](https://docs.astral.sh/uv/) is available, or
  install a skill for coding agents as described below.

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

fdu is published as a Python wheel, so [uv](https://docs.astral.sh/uv/) is all you need.
No Rust toolchain is required.

**Try it without installing:**

```shell
uvx --no-build fdu@latest .
```

**Install it as a command:**

```shell
uv tool install --no-build fdu
```

Then run it on any directory:

```shell
fdu .
```

**Upgrade later:**

```shell
uv tool upgrade --no-build fdu
```

Wheels cover Linux, macOS, and Windows.
For other platforms, pinned versions, and building from source, see
[Other Ways to Install](#other-ways-to-install).

## Quick Start

A one-level summary of this repository’s files:

```console
$ fdu . --depth=1
██████████   100%      23 MiB  . 872 files (4.0 KiB gitignored)
█████░░░░░    55%      12 MiB    docs/ 315 files
██░░░░░░░░    21%     4.7 MiB    crates/ 122 files
██░░░░░░░░    17%     3.9 MiB    explorations/ 268 files
░░░░░░░░░░     4%     932 KiB    tests/ 100 files (4.0 KiB gitignored)
░░░░░░░░░░     1%     352 KiB    scripts/ 29 files
░░░░░░░░░░     2%     440 KiB    … and 38 more files
```

Add `--analyze` to read file contents, here for lines of code and words in documents:

```console
$ fdu . --analyze=code,words
CODE
Code lines   Share  Comments   Blank  Analyzed files  Language
    79,748   61.5%    12,801   6,720         101/101  Rust       (0 gitignored)
    42,714   32.9%     1,697   5,218         133/133  Python     (0 gitignored)
     3,988    3.1%       526     358           25/25  JavaScript (0 gitignored)
     2,518    1.9%       229     145           19/19  C          (0 gitignored)
       648    0.5%       124      64           18/18  Shell      (0 gitignored)
        13   <0.1%         4       1             2/2  Swift      (0 gitignored)
         6   <0.1%         4       1             2/2  C++        (0 gitignored)
         3   <0.1%         3       1             1/1  C#         (0 gitignored)
         3   <0.1%         4       0             1/1  Go         (0 gitignored)
         3   <0.1%         4       0             1/1  PHP        (0 gitignored)
         2   <0.1%         4       1             1/1  Java       (0 gitignored)
         2   <0.1%         4       1             1/1  Kotlin     (0 gitignored)
         2   <0.1%         4       1             1/1  Ruby       (0 gitignored)
         2   <0.1%         4       1             1/1  SQL        (0 gitignored)
         2   <0.1%         4       1             1/1  TypeScript (0 gitignored)
         —       —         —       —             0/2  Make
   129,654  100.0%    15,416  12,513         308/310  TOTAL      (0 gitignored)
15 analyzed languages (include population)
36 selected files with unclassified type
2 unsupported

DOCUMENTS
Percentage column: document words
   5.9 MiB   84.6%  markdown           317 files, 117,532 lines (104,000 nonblank, 13,532 blank), 507,111 words (2,028.4 pages), 4 generated, 288 documentation
   748 KiB   13.4%  text               58 files, 5,455 lines (5,339 nonblank, 116 blank), 80,121 words (320.4 pages), 2 documentation
   380 KiB    9.7%  html               1 file, 1,125 lines (1,110 nonblank, 15 blank), 58,222 words (232.8 pages), 1 documentation
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
| Complete recursive tree | `fdu . --view tree --full --format json` |
| Every directory with recursive usage | `fdu . --kind dir --full --sort name --format json` |
| Every regular file | `fdu . --view files --kind file --full --format json` |
| Find Rust files | `fdu . --kind file --include '*.rs' --full --format paths` |

`--view` chooses what is reported; several views share one walk.
`--analyze` is the only switch that reads file bodies.
A view never turns analysis on.
Exit status 0 is a complete result, 1 a failure, and 2 a partial result or a usage
error.

`fdu --docs` is the offline guide, `fdu --help` is every flag, and `fdu --install-skill`
writes a portable skill for coding agents where they look for it (`fdu --skill` prints
it); see [Set Up with Any Coding Agent](#set-up-with-any-coding-agent).
The full grammar is in the [usage guide](docs/usage.md).

`--full` is shorthand for `--depth=all --breadth=all --limit=all --min-share=0%`;
explicit bounds override it.
It expands the selected view without changing scan scope or analysis.
Use path output for find/fd-style searches, or JSON for the same selection with exact
usage fields. Directory rows contain recursive totals and can overlap; regular-file rows
contain each file’s own size.
See
[complete inventories and find/fd examples](docs/usage.md#find-files-and-export-complete-inventories).

## Understand a Codebase

```shell
fdu . --analyze=code --ignored=exclude --limit=5
```

For example, the implementation at repository revision `7a499493` produced this stdout:

```text
(5 of 16)
Code lines   Share  Comments   Blank  Analyzed files  Language
    79,330   65.1%    12,681   6,700         101/101  Rust
    37,861   31.1%     1,513   4,696           95/95  Python
     3,988    3.3%       526     358           25/25  JavaScript
       403    0.3%        65      33             3/3  C
       249    0.2%        57      45             7/7  Shell
   121,858  100.0%    14,881  11,840         242/244  TOTAL
15 analyzed languages (exclude population)
22 selected files with unclassified type
2 unsupported
```

The note and suggestion appear once on stderr, followed by the run’s `perf:` summary:

```text
note: code totals include languages hidden by display limits
tip: show more rows: --limit=all
```

The percentages and bold TOTAL row cover all measured code lines, including languages
outside the five displayed rows.
Analyzed-file counts show measured files over selected source files; unavailable SLOC
uses a dash, distinct from a measured zero.
Counts include tests and fixtures in the selected repository, and change as the checkout
changes. Coverage makes unsupported and unclassified files visible instead of treating
them as zero lines.

`--ignored=exclude` avoids traversing and reading ignored trees such as local builds and
environments. Omit it to analyze both populations and show their contributions.
`--limit=all` shows every language; `--format=json` gives structured counts and
coverage. See [content analysis](docs/usage.md#analyze-file-contents) for the counting
convention and supported languages.

## Tally Environments and Build Outputs

Find every `.venv`, `node_modules`, and Cargo `target` directory under a work directory,
largest first, then get their combined usage from the same cached scan:

```shell
fdu ~/work --kind dir --include .venv --include node_modules --include target \
  --full --long --cache on
fdu ~/work --kind dir --include .venv --include node_modules --include target \
  --view summary --stale-ok
```

The first command lists each matching directory’s allocated size, modification age, and
path. `--cache on` makes it leave a snapshot, which a one-shot report does not do by
default. The second answers from that snapshot without another walk; it describes that
recorded scan, not changes made afterward.
Ignored directories are included by default, which is useful for environments and build
outputs.

Nested matches appear individually in the list, so adding those rows can double-count
contents. Summary counts their covered paths once.
For example, a nested `node_modules` contributes to both its own row and its parent’s
row, but only once to Summary.
The names are conventions: `target` is Cargo’s default build directory, and custom build
locations require another include pattern.
Symlinks are not followed.

For detailed rows and the total in one structured report:

```shell
fdu ~/work --kind dir --include .venv --include node_modules --include target \
  --view files,summary --full --sort size --format json
```

These are per-path sizes, not estimates of space freed by deletion.
Hard links can share one file, and copy-on-write clones can share physical blocks while
retaining separate file identities.
Multiple uv environments may therefore have overlapping physical storage even when their
paths are distinct. See
[allocation and shared files](docs/usage.md#allocation-and-shared-files).

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
On macOS, the kernel reports writes to a file only when it is closed, so a file held
open for writing, such as a growing log or database, shows its size as of its last
close.

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

On a million-entry tree, fdu built a reusable index and rendered a ten-row tree in **6.4
seconds**, covering **137k files/s** and **0.47 GB/s**. Measured on an M1 Pro’s internal
APFS SSD with warm filesystem caches and fdu’s cache disabled (`--cache off`),
2026-09-28. These are approximate local results under heavy background load.
The default `fdu PATH` measured the same within 0.1%, because it no longer writes a
snapshot on a one-shot run.

| Tool | Work returned | Median wall-clock time | Wall time vs. fdu | Files/s | GB/s |
| --- | --- | ---: | ---: | ---: | ---: |
| **fdu** | reusable exact index and ten-row tree | **6.4 s** | baseline | **137k** | **0.47** |
| dumac | allocated-byte total only | 6.9 s | **+9%** | 127k | 0.43 |
| pdu | block total only (`--max-depth 1`)¹ | 9.2 s | +49% | 96k | 0.33 |
| diskus | scalar total only | 9.3 s | +42% | 94k | 0.32 |
| dua | scalar total only | 10.4 s | +61% | 84k | 0.29 |
| gdu | rendered tree | 10.5 s | +67% | 83k | 0.28 |
| dust | allocated-byte total only | 11.0 s | +57% | 79k | 0.27 |
| BSD `du` | one total, serial | 55.6 s | +801% | 16k | 0.054 |
| ncdu | reusable index | 67.3 s | +968% | 13k | 0.044 |
| GNU `du` | one total, serial | 68.0 s | +960% | 13k | 0.044 |

Positive percentages mean extra elapsed time: +60% means 1.6× as long as the adjacent
fdu run. Rates count regular files and their disk space, not file-content reads; `k`
means thousands and GB is decimal.
¹ pdu counts the root as depth 1, so `--max-depth 1` prints only the root’s total; these
runs were recorded as a rendered tree before that was noticed.
On a real source tree, pdu’s default depth took 6% longer than `--max-depth 1`. See the
[full comparison](docs/project/reports/report-2026-09-26-fdu-live-tool-comparison.md)
for methodology, memory use, confidence intervals, and exact results.

On Linux the ranking depends on the job.
The same million-entry tree on a 4-vCPU virtualized ext4 host, same harness, 2026-09-28:

| Tool | Work returned | Median wall-clock time | Wall time vs. fdu summary |
| --- | --- | ---: | ---: |
| **fdu `--no-gitignore --view summary`** | exact totals, no index | **0.94 s** | baseline |
| pdu | block total only (`--max-depth 1`)¹ | 1.02 s | +8% |
| diskus | scalar total only | 1.04 s | +12% |
| fdu | reusable exact index and ten-row tree | 1.25 s | — |
| dust | allocated-byte total only | 1.67 s | +76% |
| gdu, GNU `du`, ncdu, dua | tree or total | 2.6–3.9 s | +162% to +303% |

fdu’s summary mode is the fastest tool measured, but building the reusable index takes
about 23% longer than pdu and 21% longer than diskus there.
The gap is in user space, not the filesystem: all three issue the same system calls, and
fdu spends less kernel time per entry.
Under a preloaded jemalloc, fdu’s indexed tree ran level with pdu, so most of the gap is
allocator traffic between fdu’s walker threads and its index builder; the rest is
walkers waiting on a shared queue where pdu’s rayon workers steal work.
Returning the builder’s buffers to the walkers that allocated them
([#150](https://github.com/jlevy/fdu/pull/150)) cut this indexed tree by 10.6% and a
real directory-dense `node_modules` tree by 8.6%; the saving is per directory, so a
source tree with few, larger directories does not show it.
See the
[Linux comparison](docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md)
and
[the pdu brief](docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md).

This generated tree holds no `.gitignore` files, and that hid the larger cost on real
repositories. On the Linux v6.12 source tree (92,474 entries, 358 `.gitignore` files),
the default `fdu .` took 564 ms against 70 ms for pdu, while `fdu . --no-gitignore` took
76 ms (screens): classifying entries against ignore rules ran on one thread and
allocated 78 times per entry.
Two changes since 0.2.0 cut that job to 211 ms and the default summary from 505 to 167
ms (quiet paired probe runs,
[exp-173](docs/project/experiments/exp-173-linux-h162-allocation-free-gitignore-matching-halves-the-def.md)
and
[exp-174](docs/project/experiments/exp-174-linux-h163-per-listing-control-chains-cut-another-third-from.md)).
That is still three times pdu; the remainder is classification on one thread, and moving
it onto the walker threads is the next change.
Both tables measure fdu with its cache disabled, which is also what the default `fdu .`
now does for a one-shot report: it used to write a snapshot that no later `fdu .` reads,
about a fifth of a repeated run on this Linux tree and two fifths of a first one.
On macOS the saving is memory rather than time: about 100 MiB of peak RSS, with no wall
change the benchmark could resolve.
The default summary, which reads ignore rules, no longer falls back to the full index:
it classifies each entry as it counts it.
On this tree that made it 19% faster and cut its peak memory from 314 to 8.6 MiB, within
2% of `--no-gitignore`
([exp-187](docs/project/experiments/exp-187-linux-h161-ignore-aware-transient-summary-clears-wall-rss-ba.md));
the table keeps the `--no-gitignore` figure the harness measured.
The
[cache economics brief](docs/project/research/research-2026-09-27-cache-economics-and-default-plans.md)
covers when the cache and the index pay on each platform.
Windows builds and passes tests but has not been performance-benchmarked.

### Multi-View Reports

Report construction has a separate result.
In an exploratory, uncontrolled macOS benchmark on a 137,085-entry tree, a loop that
constructed the unfiltered Types, Families, Languages, and Documents views 100 times
from an already line-analyzed index took 12.0 seconds, down from 29.9 seconds—about
**2.5× faster**, or roughly 120 ms instead of 299 ms per report.
The code is retained provisionally; a quiet confirming run on the integrated stack
(2026-09-28) failed to qualify on a loaded host, so the inconclusive major-fault gate
still stands. These timings predate the integration of the new Code overview, population
controls, and tree accounting; the combined engine needs a fresh paired measurement.

This is not a scan or end-to-end full-analysis speedup.
It applies only to unfiltered requests with multiple metric views; the default
disk-usage command and single-view analysis are unchanged.
The implementation is platform-neutral Rust, but its Linux magnitude has not yet been
measured. See
[the experiment](docs/project/experiments/exp-159-share-content-metric-resolution-across-views.md)
for the paired interval, host regime, and resource qualification.

## Why

Of fifteen surveyed tools in this space ([du](https://www.gnu.org/software/coreutils/),
[ncdu](https://dev.yorhel.nl/ncdu), [dust](https://github.com/bootandy/dust),
[pdu](https://github.com/KSXGitHub/parallel-disk-usage),
[diskus](https://github.com/sharkdp/diskus),
[dumac](https://github.com/healeycodes/dumac#readme),
[dua](https://github.com/Byron/dua-cli), [gdu](https://github.com/dundee/gdu),
[dut](https://codeberg.org/201984/dut), [duc](https://github.com/zevv/duc),
[fsearch](https://github.com/cboxdoerfer/fsearch),
[bfs](https://github.com/tavianator/bfs), [fd](https://github.com/sharkdp/fd),
[scc](https://github.com/boyter/scc), [tokei](https://github.com/XAMPPRocky/tokei)),
exactly one persists anything, exactly one carries multiple metrics per pass, **none**
does per-directory type tallies, and **none** does mtime-based incremental revalidation.
None of them is a native library with a live change feed that a Rust or Python program
can hold. That combination is what a live file browser needs.

The survey is in
[the file roll-up engine research](docs/project/research/research-2026-08-06-file-rollup-engine.md);
[the pdu brief](docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md)
reads pdu, diskus, and dumac at source level and maps pdu’s options to fdu’s.

## Other Ways to Install

**Platforms:** Wheels cover GIL-enabled CPython 3.12 and newer on Linux glibc (x86-64
and arm64), macOS (x86-64 and arm64), and Windows x86-64. `--no-build` requires one of
those wheels rather than compiling from source.
You don’t need to pick a Python version: uv selects a matching interpreter.
If uv selects free-threaded CPython, such as `3.14t`, retry with `--python 3.14`; fdu
does not publish free-threaded wheels yet.

**Pinned versions:** For a repeatable run, replace `latest` with a release number, such
as `uvx --no-build fdu@0.2.0 .`.

**uv cool-off policies:** If uv is configured with an `exclude-newer` cool-off, a new
fdu release may be filtered.
Review and allow the first-party `fdu` package in that policy, or wait for the cool-off
to expire.
`--no-config` is a one-off override that skips all uv configuration, including
that policy.

**From crates.io (Rust 1.85 or newer):**

```shell
cargo install --locked fdu
```

`--locked` keeps the reviewed dependency set; see
[SUPPLY-CHAIN-SECURITY.md](SUPPLY-CHAIN-SECURITY.md).

**From a source checkout:**

```shell
git clone https://github.com/jlevy/fdu.git
cd fdu
cargo install --locked --path crates/fdu
```

## Documentation

- [Usage guide](docs/usage.md): views, analyzers, selection, cache, watch, machine
  output
- [Documentation index](docs/README.md): library, architecture, performance, release
- [Design principles](docs/project/architecture/fdu-design-principles.md)
- [0.2.0 release notes](docs/project/release-notes/0.2.0.md)
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
The [output design system](docs/project/architecture/fdu-output-design.md) governs
report layout, diagnostic categories, colors, and stdout/stderr separation; its
implementation rules are documented beside the shared renderer and diagnostic collector.
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
