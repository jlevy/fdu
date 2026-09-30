# fdu

**Fastest du replacement and file tree analysis for 100+GB, million-file worktrees**

Use fdu to find what takes up space, locate old build directories, or summarize a tree
without writing a filesystem walker.

## Key Features

- **Speed:** fdu walks a tree on several threads at once and reads each directory
  through the platform’s native interface: `getattrlistbulk` on macOS, which returns a
  directory’s names and sizes in one call, and `getdents64` with `statx` on Linux.
  On a million-file tree it runs about 9× as fast as `du` on macOS and 2.6× as fast on
  Linux, and ahead of every other tool measured, including
  [dumac](https://github.com/healeycodes/dumac#readme),
  [pdu](https://github.com/KSXGitHub/parallel-disk-usage),
  [diskus](https://github.com/sharkdp/diskus), and
  [dust](https://github.com/bootandy/dust).
  See [Speed](#speed).
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

Time to report on the same generated million-file tree, with warm filesystem caches, as
a multiple of fdu’s time (lower is faster):

| Tool | Linux | macOS |
| --- | ---: | ---: |
| **fdu** | **1.00×** (0.95 s) | **1.00×** (6.4 s) |
| [pdu](https://github.com/KSXGitHub/parallel-disk-usage) `--max-depth 2` | 1.19× | — |
| [diskus](https://github.com/sharkdp/diskus) | 1.24× | 1.42× |
| [pdu](https://github.com/KSXGitHub/parallel-disk-usage) | 1.25× | 1.49× |
| [dust](https://github.com/bootandy/dust) | 1.62× | 1.57× |
| [gdu](https://github.com/dundee/gdu) | 2.58× | 1.67× |
| GNU [du](https://www.gnu.org/software/coreutils/) | 2.60× | 10.6× |
| [ncdu](https://dev.yorhel.nl/ncdu) | 2.74× | 10.7× |
| [dua](https://github.com/Byron/dua-cli) | 3.34× | 1.61× |
| [dumac](https://github.com/healeycodes/dumac#readme) | — | 1.09× |
| BSD `du` | — | 9.0× |

Linux is a 4-vCPU virtual machine on ext4; macOS is an M1 Pro on APFS.
[Speed](docs/speed.md) gives each run’s date, engine, intervals, and memory, and the
results on real source trees.

## Comparison to Alternatives

Many tools report disk usage, and this is how fdu compares with the ones people most
often reach for, and with the two leading source-line counters for its code analysis.
Each cell was checked against that tool’s source or documentation; [Speed](#speed)
compares their run times.
✅ means the tool does what the row names, text alone means partial or different support,
❌ means none (any text says what the tool does instead), and — means the row does not
apply:

| Feature | fdu | du | ncdu | dust | dua | gdu | pdu | diskus | dumac | scc | tokei |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Plain total | ✅ `--view summary` | ✅ `-s` | ❌ TUI or export | ✅ `-d 0` | ✅ total row | ✅ `-ns` | ✅ `-d 1` | ✅ | ✅ | —² | —² |
| Tree breakdown and pruning | ✅ depth, breadth, share floor, row limit | ✅ depth, size floor | TUI browsing | ✅ depth, top N, size floor | depth; TUI browsing | depth, top N files; TUI browsing | ✅ depth, share floor | ❌ | ❌ | ❌ per language or file | ❌ per language or file |
| `.gitignore` | ✅ classify; include, exclude, or only ignored | ❌ | ❌ | ❌ | partial: TUI dims ignored entries; `--ignore-from` patterns | ❌¹ | ❌ | ❌ | ❌ | ✅ exclude | ✅ exclude, inside a git repository |
| Source code analysis² | ✅ 15 languages: code, comment, and blank lines, per directory | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ 366 languages; complexity and cost estimates | ✅ 333 languages; embedded languages |
| Code analysis speed, Linux source³ | 7.9 s; 0.55 s repeated | — | — | — | — | — | — | — | — | 1.2 s | 1.9 s |
| Text analysis | ✅ lines, words, paragraphs, pages | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| APIs and machine output | ✅ Rust, Python; JSON, JSONL, YAML | tab-separated text; `-0` | JSON export | JSON (`-j`) | Rust library; snapshot files | JSON export; SQLite or Badger | ✅ Rust library; JSON | Rust library | ❌ | ✅ Go package; JSON, CSV, HTML, SQL | ✅ Rust library; JSON |
| Watch and stream | ✅ `--watch`, JSONL change stream | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Cached results | ✅ snapshot and content cache, revalidated | ❌ | export, not revalidated | ❌ | snapshot, not revalidated | database, not revalidated | JSON, not revalidated | ❌ | ❌ | ❌ | ❌ |
| Agent skill | ✅ `--install-skill` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | MCP server (`--mcp`) | ❌ |

¹ gdu’s unreleased main branch adds `--ignore-from-gitignore`, which reads patterns from
one file.

² [scc](https://github.com/boyter/scc) and [tokei](https://github.com/XAMPPRocky/tokei)
count source lines, not disk usage, so the disk-usage rows show “—”. They count far more
languages than fdu; tokei also counts code embedded in another language, such as
Markdown code fences, and scc estimates complexity and cost.
fdu tallies code per directory, reports each language’s ignored share, keeps its counts
in a content cache so that a repeated run reads only changed files, and measures disk
usage in the same pass.
On the Linux kernel’s 59,953 C sources and headers, all three give the same code,
comment, and blank counts for 59,766 files; scc differs on 22, where it counts form-feed
lines as code, and tokei on 165: 107 through three defects in its C parsing, 55 through
a different convention for a macro’s line splice after a comment, and 3 not attributed.
See the
[SLOC tools survey](docs/project/research/research-2026-09-29-sloc-tools-survey.md).
[cloc](https://github.com/AlDanial/cloc) recognizes the most languages, 402, but runs as
a single Perl process by default.

³ Median wall time on a copy of the Linux v6.12 source without `.git` (86,618 files, 1.5
GB of file data), each tool with every ignore-file source off, hidden files counted, and
text output: 12 adjacent pairs on a quiet 4-vCPU Linux virtual machine, 2026-09-29.
fdu’s first run, with its cache off, took 6.4 times as long as scc and 4.2 times as long
as tokei; the three read about the same bytes, so the gap is fdu’s CPU per byte.
Run again under the default cache policy, fdu reopened no unchanged file and answered in
0.55 s. With each tool’s own `.gitignore` handling on a git clone, fdu took 9.2 s, scc
1.3 s, and tokei 2.0 s.

Versions checked for the feature cells: GNU coreutils `du` 9.4, and its source after
9.12; ncdu 1.19 and 2.9.2; dust 1.2.5; dua 2.45.0; gdu 5.37.0, and its main branch at
`4b179b0`; pdu 0.24.0; diskus 0.9.0; dumac at `1ffbe3c`; scc 4.1.0; tokei 15.0.0. dumac
runs only on macOS and ncdu only on Unix-like systems; the others run on macOS, Linux,
and Windows, `du` through a Unix layer such as MSYS2.

**When to use each.** ncdu, dua, and gdu let you browse a tree and delete from it
interactively, which fdu does not; gdu can also serve a browser view, and `dua clean`
finds build products to remove.
`du` is already installed on every Unix-like system.
diskus and dumac answer one total from a small binary, and pdu draws a compact size
chart; they are the closest to fdu in speed.
For line counts alone, use scc or tokei: they count hundreds of languages to fdu’s 15,
tokei counts embedded code, scc estimates complexity, and both count a large tree four
to six times as fast as fdu’s first run.
Use fdu for a tree you can prune, a `.gitignore`-aware answer, content metrics,
versioned machine output, a live or cached view, or a Rust or Python API; for code, it
adds counts per directory, each language’s ignored share, and a cache that makes a
repeated count faster than either counter’s.

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
several save a scan to reload later: ncdu, gdu, and pdu export one; gdu, duc, and
fsearch keep a database; and dua writes snapshots.
But **none** revalidates a saved scan by modification time, **none** does per-directory
type tallies, and none caches content metrics between runs.
None of them is a native library with a live change feed that a Rust or Python program
can hold. That combination is what a live file browser needs;
[Comparison to Alternatives](#comparison-to-alternatives) shows where each of the common
peers stands.

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
as `uvx --no-build fdu@0.3.0 .`.

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
- [0.3.0 release notes](docs/project/release-notes/0.3.0.md)
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
