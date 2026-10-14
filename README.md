# fdu

[![Follow @ojoshe on X](https://img.shields.io/badge/follow_%40ojoshe-black?logo=x&logoColor=white)](https://x.com/ojoshe)

**Fastest du replacement and file tree analysis for 100+GB, million-file worktrees**

fdu is a ground-up rewrite of both du and more modern tools like dust.
It is pre-built and distributed via PyPI and `uv` so you can run it right now with `uvx fdu@latest`.

For every directory in a file tree, fdu reports its size, file count, most recent
change, file kinds, how much `.gitignore` files cover.
Optionally, it also analyzes lines of code by programming language or words, paragraphs,
and pages in documents for Markdown or text.

The same engine powers the `fdu` command, a Python package, and a Rust crate.
It is systematically optimized native code, with parallelized, OS-specific techniques
for macOS and Linux.
It combines directory sizes, content analysis, cached results, and live updates in a
cross-platform CLI. See [speed measurements](#speed) and the
[comparison to alternatives](#comparison-to-alternatives).

It is packaged for convenience as a CLI installable via PyPI and uv, so you can run it
on most platforms instantly via `uvx fdu@latest` or `uv tool install fdu@latest`.
It offers APIs and agent-friendly use as a skill.

![fdu on the Linux kernel source](docs/media/fdu-demo.gif)

*fdu v0.5.0 on the Linux 7.3-rc6 source (1.7 GiB, 95,938 files), recorded in real time
on an older Mac M1 Pro.*

<!-- The tagline is duplicated as the description in crates/fdu/Cargo.toml (the first
line of `fdu --help`) and crates/fdu-py/pyproject.toml, the heading of
crates/fdu/README.md and crates/fdu-py/README.md, the first line of the --docs guide in
crates/fdu/src/cli.rs, and the GitHub repository description. Speed wording also appears
in those READMEs, crates/fdu/src/skills/SKILL.md, and --docs, whose output
tests/golden/cli-surface.tryscript.md and tests/parity/deviations-python.diff record.
Update them together. -->

## Why

Over the past few months, I found myself managing numerous worktrees with hundreds of
thousands of files or 100GB+ folders.
Agents struggle to tally disk usage, file age, or know where to
begin to clean up large folders.
I wanted a tool for myself and for agents to have visibility and manage or clean up
large numbers of files efficiently.
I've also been building applications like file managers that need to have live
visibility over 100K-1M files.

Thanks to an agentic performance iteration loop, fdu is the fastest tool of its kind,
typically walking and tallying 5-10GB/s or 100-300K files/s on my old MacBook.
That is ~9x faster than du and 10-100% faster than other modern native/Rust/Go options.

Also, existing high-performance file analytics tools are not designed flexibly enough.
I wanted something that is designed in layers with both easy reports but also
OS-native file watching with streaming APIs.
It is designed from the start to be usable in pure Rust, from Python,
or as a CLI or agent skill.

More on how it was built [in my Twitter post](https://x.com/ojoshe/status/2110201362189594898).

## Key Features

- **Extensive metrics:** Different `--view` options show total size, file count, age of
  entire directories, tree overview, largest or or most recent files, and breakdowns by
  file type, family, extension, and programming language.
- **`.gitignore`-aware sizes:** Rows show how much of their size the tree’s own
  `.gitignore` files cover, and `--ignored=exclude` or `--ignored=only` reports one
  side. Rules apply per directory, as git applies them; `.git/info/exclude` and global
  ignore files are not read.
- **Code and documents:** `--view=code` counts code, comment, and blank lines for each
  of 15 languages, over the tree or any directory you name; the Rust API’s
  `content_rollup` also gives each directory’s totals across languages.
  `--view=documents` counts words, paragraphs, and pages in prose and markup.
  Results are cached, so a repeated run reads only the files that changed.
- **Find and inventory:** Select entries by kind, glob, size, and age, such as every
  `node_modules` untouched for 30 days, and print them as a tree, plain paths,
  size-and-age rows, or a complete JSON inventory.
- **Live updates:** Designed to support watching an entire folder for changes and
  streaming results, using each platform’s native file events (FSEvents, inotify,
  `ReadDirectoryChangesW`).
- **For scripts, agents, and programs:** Terminal, text, JSON, JSONL, and YAML outputs
  make scripting easy.
  `fdu --install-skill` gives coding agents a self-contained skill; and the same engine
  is a Python package and a Rust crate.



## Use Instantly with Any Coding Agent

Hand any coding agent this one instruction:

> Run `uvx fdu@latest --install-skill` from the project root to install fdu’s
> self-contained skill for current and future agent sessions.

The command writes `.agents/skills/fdu/SKILL.md` and `.claude/skills/fdu/SKILL.md` under
the project root. The skill needs no prior session context and uses `fdu` on `PATH` or a
wheel-only `uvx` fallback; no installed command or Rust toolchain is required.

Use `--agent-base DIR` to write `DIR/skills/fdu/SKILL.md` for one agent’s user scope,
such as `~/.claude`. Re-run the installer after upgrading to refresh the skill;
`fdu --skill` prints it, and deleting the generated skill directories removes it.
See the [skill usage guide](docs/usage.md#agent-skill).




## Install the Command Line

fdu is published as prebuilt wheels on PyPI, so [uv](https://docs.astral.sh/uv/) is all
you need; no Rust toolchain is required.
Run the latest release once, or install it as a command and upgrade it later:

```shell
uvx fdu@latest .
uv tool install fdu
uv tool upgrade fdu
```

`pip install fdu` installs the same command along with the
[Python package](#as-a-python-module), and `cargo install --locked fdu` builds it from
source with Rust 1.85 or newer.
Wheels cover Linux (glibc), macOS, and Windows x86-64; for other platforms, pinned
versions, and uv cool-off policies, see [Other Ways to Install](#other-ways-to-install).

## Quick Start

The samples on this page come from a fresh clone of this repository at revision
`55d66863`; your numbers will differ.
A one-level summary of the clone, its hidden `.git/` directory included:

```console
$ fdu . --depth=1
██████████   100%      63 MiB  1m  . 1,185 files (4.0 KiB gitignored)
████░░░░░░    41%      25 MiB  1m    .git/ 28 files
████░░░░░░    38%      24 MiB  1m    docs/ 492 files
█░░░░░░░░░     9%     5.5 MiB  1m    crates/ 126 files
█░░░░░░░░░     8%     4.8 MiB  1m    explorations/ 286 files
░░░░░░░░░░     2%     1.2 MiB  1m    tests/ 112 files (4.0 KiB gitignored)
░░░░░░░░░░     1%     700 KiB  1m    scripts/ 43 files
░░░░░░░░░░     2%     972 KiB        … and 98 more files
```

Beside each size is the row’s age: how long ago anything it counts last changed, a file
or a directory, here the clone itself a minute earlier.
An age reads in seconds, minutes, hours, days, months (`mo`), or years (`y`), and is
`unknown` where part of a subtree could not be listed.

The result goes to stdout.
On stderr, notes and a suggestion follow, then a `perf:` line with the run’s timing,
left out here:

```text
note: totals include gitignored sizes and descendants
note: display limits: below 1% of root, depth 1
tip: show more: --min-share=0% --depth=all
```

Sizes are allocated disk space, as `du` reports by default, and `--size=apparent` gives
file lengths. On Windows, allocated size falls back to file lengths; see
[allocation and shared files](docs/usage.md#allocation-and-shared-files).

`--view` chooses what is reported, and several views share one walk.
Only `code` and `documents` read file contents; every other view reads metadata and
`.gitignore` files. `--analyze` adds analysis to the others, such as code lines in
`--view=languages`.

| Question | Command |
| --- | --- |
| Which directories are large? | `fdu .` |
| Lines of code and words in documents | `fdu . --view=code,documents` |
| One total for the tree | `fdu . --view=summary` |
| Languages by space | `fdu . --view=languages` |
| File families, types, and extensions | `fdu . --view=families,types,extensions` |
| Twenty largest files | `fdu . --view=largest` |
| Ten most recently changed working files | `fdu . --view=recent --limit=10 --ignored=exclude --exclude='.git/**'` |
| Totals without gitignored entries | `fdu . --ignored=exclude --view=summary` |
| Stay on one filesystem, like `du -x` (macOS and Linux) | `fdu . --one-filesystem` |
| Several paths as one report, with a total | `fdu docs src` |
| Machine output | `fdu . --format=json` |
| Keep the tree live | `fdu . --watch` |

Content analysis, `--watch`, and the libraries’ `open` keep their state under
`~/.cache/fdu` (`%LOCALAPPDATA%\fdu` on Windows), and a plain `fdu PATH` writes nothing.
`fdu --cache-status=all` lists what is stored, and `fdu --cache-clear=all` removes it.

`fdu --help` lists every flag, `fdu --docs` prints the offline guide, and the
[usage guide](docs/usage.md) covers every view, format, and exit status.

## Understand a Codebase

Lines of code by language and words by document type, from one scan of the same clone:

```console
$ fdu . --view=code,documents
CODE
Code lines   Share  Comments   Blank  Analyzed files  Language
    91,431   55.5%    15,702   7,488         102/102  Rust       (0 gitignored)
    61,533   37.3%     2,512   7,291         163/163  Python     (0 gitignored)
     6,182    3.8%       747     516           33/33  JavaScript (0 gitignored)
     2,709    1.6%       251     155           20/20  C          (0 gitignored)
     1,942    1.2%       252     206           35/35  TypeScript (0 gitignored)
     1,009    0.6%       386     135           31/31  Shell      (0 gitignored)
        13   <0.1%         4       1             2/2  Swift      (0 gitignored)
         6   <0.1%         4       1             2/2  C++        (0 gitignored)
         3   <0.1%         3       1             1/1  C#         (0 gitignored)
         3   <0.1%         4       0             1/1  Go         (0 gitignored)
         3   <0.1%         4       0             1/1  PHP        (0 gitignored)
         2   <0.1%         4       1             1/1  Java       (0 gitignored)
         2   <0.1%         4       1             1/1  Kotlin     (0 gitignored)
         2   <0.1%         4       1             1/1  Ruby       (0 gitignored)
         2   <0.1%         4       1             1/1  SQL        (0 gitignored)
         —       —         —       —             0/2  Make
         —       —         —       —             0/1  Perl
   164,842  100.0%    19,885  15,798         395/398  TOTAL      (0 gitignored)

DOCUMENTS
   7.5 MiB   67.9%  markdown           381 files
                                         145,935 lines (129,677 nonblank, 16,258 blank)
                                         651,479 words (2,605.9 pages)
                                         8 generated
   1.6 MiB   21.8%  text               66 files
                                         31,754 lines (31,571 nonblank, 183 blank)
                                         209,075 words (836.3 pages)
   648 KiB   10.3%  html               2 files
                                         2,354 lines (2,319 nonblank, 35 blank)
                                         98,515 words (394.0 pages)
```

On stderr, before the `perf:` line:

```text
note: percentages are shares of code lines (CODE), document words (DOCUMENTS)
note: 15 languages analyzed
note: not analyzed: 3 unsupported
note: 49 files with unclassified type
```

Code lines leave out comments and blank lines, which have columns of their own.
Analyzed files counts the files measured out of the source files selected, and a
language fdu has no counter for shows a dash rather than zero, as Make and Perl do.
The notes count unsupported and unclassified files instead of treating them as zero
lines.
In DOCUMENTS, each format’s measures are listed under its file count: lines, words
and pages, at 250 words a page (`--words-per-page`), then any files detected as
generated or vendored.

Each language row shows its gitignored share.
`--ignored=exclude` skips gitignored trees such as local builds and environments,
without walking or reading them.
`--limit=5` keeps the five largest languages, while the TOTAL row and the notes still
account for the rest, and `--format=json` gives structured counts and coverage.
See [content analysis](docs/usage.md#analyze-file-contents) for the counting convention
and supported languages.

## Find Environments and Build Outputs

List every `.venv`, `node_modules`, and Cargo `target` directory under a work directory,
largest first, with its allocated size and the age of its newest change:

```shell
fdu ~/work --kind=dir --include=.venv --include=node_modules --include=target --full --long
```

Add `--modified-before=30d` for the ones untouched in a month, or
`--sort=mtime --reverse` for oldest first; replace `--long` with `--format=paths` for
paths alone. `--view=summary` gives their combined usage and counts nested matches once;
`--view=files,summary --format=json` gives exact rows and the total in one report.
Add `--cache=on` to keep the scan, and a later run with `--stale-ok` answers from it
without walking again.

A directory’s size counts the regular files below it, and its age is the newest
modification of the directory or anything in it, which measures activity, not last use.
Ignored directories are included, and symlinks are not followed.
Sizes are per path, not space freed by deletion: hard links and copy-on-write clones, as
uv environments use, can share storage.
See [allocation and shared files](docs/usage.md#allocation-and-shared-files).

## Export Inventories and Search Files

`--full` lifts every display bound.
It is shorthand for `--depth=all --breadth=all --limit=all --min-share=0%`, explicit
bounds override it, and it changes neither what is scanned nor what is analyzed.

```shell
fdu . --view=tree --full --format=json                    # the complete recursive tree
fdu . --kind=dir --full --sort=name --format=json         # every directory, recursive usage
fdu . --view=files --kind=file --full --format=json       # every regular file
fdu . --kind=file --include='*.rs' --full --format=paths  # Rust files, like find or fd
```

Paths output is a find/fd-style search, and JSON gives the same selection with exact
usage fields. Directory rows hold recursive totals and can overlap; regular-file rows
hold each file’s own size.
See
[complete inventories and find/fd examples](docs/usage.md#find-files-and-export-complete-inventories)
and the [machine-output reference](docs/machine-output.md).

## Live Updates

`--watch` is the same query, re-evaluated as the tree changes:

```shell
fdu . --watch
fdu . --watch --view=files --format=jsonl
```

Changes arrive as the platform’s native file events (FSEvents, inotify,
`ReadDirectoryChangesW`), so an idle tree is not polled, and each event is checked with
a fresh `stat` before it changes the result.
`--interval` throttles how often a text view repaints, not how changes are detected.
On macOS, the kernel reports writes to a file only when it is closed, so a file held
open for writing, such as a growing log or database, shows its size as of its last
close. Content analysis is one-shot and cannot be combined with `--watch`.

Library callers get the same feed as typed values: Rust `Session` (behind the `watch`
build feature) and Python `Index.watch()`. An interactive client, such as a file
browser, uses `OpenedIndex` in Rust or `fdu.opened` in Python: it answers while the
first walk is still running, reads large results a page at a time, and resumes its
change feed from a cursor; see
[long-lived roots](crates/fdu-py/README.md#long-lived-roots).


## Python and Rust APIs

Keep discovery, analysis, reconciliation, observation, querying, and engine shutdown in
Rust. Python should set application policy through coarse requests and receive selected
answers. The Python package offers three entry-point groups over that same engine:

| API | Choose it for |
| --- | --- |
| Report API: `fdu.report()` | One answer per job or invocation, from a summary to a detailed report |
| Index API: `fdu.open()` / `fdu.scan()` | Prepare once, reuse queries, and control refresh; supports analysis and cache policy on open |
| Progressive API: `fdu.opened.OpenedIndex.open()` | Read during discovery, consume bounded pages, or follow optional changes in offline jobs or interactive applications |

Native inventory is not automatically exported to Python.
Returned rows and reports allocate Python values and can copy data; there is no public
zero-copy data export.
Minimize both boundary calls and returned volume.
Detailed or complete results remain a deliberate option for custom Python logic, with
their conversion and retention cost.
See the [Python guide](crates/fdu-py/README.md#use-as-a-python-library) for
final-summary requests, lifecycle choices, coverage, paging, and current costs.

## As a Python Module

Add the package to a uv project, or install it in the current Python environment:

```shell
uv add fdu
pip install fdu
```

The same wheel installs the native `fdu` command; there is no Python reimplementation of
the command line.

```python
from pathlib import Path

import fdu

# One question: a one-shot report, as the command line runs it.
report = fdu.report(Path("."), fdu.Query(views=(fdu.View.CODE, fdu.View.DOCUMENTS)))
print(report.render())  # the same tables as `fdu . --view=code,documents`

# Many questions: open a retained index once and ask it repeatedly.
index = fdu.open(Path("/path/to/tree"))
print(index.total().files, index.status.complete)
report = index.report(fdu.Query(views=(fdu.View.LANGUAGES,)))
print(report.as_dict())  # the command line's JSON report, as a dict
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

Values are frozen dataclasses and enums, native work runs with the GIL released, and
`index.status.complete` and `report.provenance.freshness` say whether an answer is
complete and current.
The [Python package README](crates/fdu-py/README.md) covers every option, report
section, and the long-lived `fdu.opened` interface.

## As a Rust Library

```shell
cargo add fdu
```

`fdu` re-exports the engine.
The default `watch` build feature adds the OS-native watch layer;
`cargo add fdu --no-default-features` leaves it out.
An embedding that wants none of the command line’s dependencies depends on `fdu-core`
instead. The API reference is on [docs.rs/fdu](https://docs.rs/fdu) and
[docs.rs/fdu-core](https://docs.rs/fdu-core).
`open` retains an index for many questions; `prepare_report` answers one report without
retaining one. Content analysis is opt-in: `content: Default::default()` scans metadata
only.

```rust
use fdu::content::AnalysisSet;
use fdu::query::{Basis, Delivery, Scope};
use fdu::{CachePolicy, open};
use std::path::Path;

fn main() -> Result<(), fdu::Error> {
    let basis = Basis {
        root: Path::new(".").into(),
        scope: Scope::default(),
        content: AnalysisSet::ALL,
    };
    let delivery = Delivery::new(CachePolicy::Auto, None);
    let (index, _report) = open(&basis, &delivery)?;
    let total = index.total();
    println!("{} files, {} bytes", total.files, total.bytes);

    // Per-directory roll-ups are already computed; this is not another walk.
    let src = Path::new("src");
    if let Some(rollup) = index.rollup(src) {
        println!("src/: {} files, newest {}", rollup.files, rollup.newest_mtime_ns);
    }
    if let Some(content) = index.content_rollup(src) {
        println!("src/: {} code lines", content.total.code.metrics.code_lines);
    }
    Ok(())
}
```

Later questions reuse the index; `refresh` reconciles it against the tree, and with
`watch` enabled, `fdu::session::Session` answers the same request as events arrive.

## Speed

fdu walks directories in parallel and uses native directory readers: `getattrlistbulk`
on macOS, and `getdents64` with directory-relative `statx` on Linux.
One-shot metadata summaries and bounded trees can avoid keeping a full index while still
counting every selected entry.
`.gitignore` rules are indexed by name, extension, and suffix to reduce pattern
matching.

Metadata reports read filesystem metadata and `.gitignore` files.
Content analysis also reads file contents; the content cache reuses results for
unchanged files, so repeated analysis need not reread the whole tree.
A retained index serves repeated queries, and `--stale-ok` can answer from a saved scan
without walking again when an older answer is acceptable.

[Performance measurements](docs/performance-measurements.md) records the workloads,
platforms, exact builds, and uncertainty behind comparisons with other tools.
The [performance loop](docs/project/guides/performance-loop.md) evaluates changes with
paired runs on real trees, and the
[performance evidence report](docs/project/reports/report-2026-08-20-fdu-performance-evidence.md)
retains the experiments and their measured effects.

The figures below are historical measurements of the builds named beside them.
They are not measurements of the final 0.5.0 release.
In the 2026-10-10 Linux rerun, neither the 0.5.0 development build nor 0.4.0 showed a
resolved lead over `pdu --max-depth 2` or diskus on the two real trees; the generated
million-entry tree retained its leads.
See
[the current Linux evidence](docs/performance-measurements.md#the-050-development-build).

Time to report on the generated million-entry tree, 875,000 files and 2.99 GB of
allocated space, with warm filesystem caches, as a multiple of fdu’s time (lower is
faster):

| Tool | Linux | macOS, exploratory |
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

Linux is a 4-vCPU virtual machine on ext4. pdu, `pdu --max-depth 2`, and diskus ran
beside fdu 0.3.0 (0.95 s) on 2026-09-30. The other Linux tools ran a day earlier beside
the pre-release engine `ebc06c78` (1.09 s), so their multiples are of that engine’s time
and understate 0.3.0’s lead.
On the two real trees, every tool took 29% to 38% longer in the 0.3.0 session than
earlier that day, and about ten points of the 19% gap to `pdu --max-depth 2` on this
tree are probably that session rather than fdu.
macOS is one exploratory session on an M1 Pro’s APFS SSD, on 2026-09-28, with a
pre-0.2.0 build on a heavily loaded, uncontrolled host; pdu ran there as
`--max-depth 1`. It is exploration, not a claim the project’s
[performance loop](docs/project/guides/performance-loop.md#host-pressure-regimes) would
accept, and dumac, the closest tool there, was measured only on macOS. Windows has no
measurements. fdu’s default report covers about 920,000 files and 3.1 GB a second on
Linux (fdu 0.3.0), and 137,000 files and 0.47 GB a second on macOS (the pre-0.2.0
build); these are metadata rates, since sizing a file reads none of its contents.
[Performance Measurements](docs/performance-measurements.md) has each run’s intervals,
memory, and what each tool returns.

## Comparison to Alternatives

Many tools report disk usage, and this is how fdu compares with the ones people most
often reach for, and with the two leading source-line counters for its code analysis.
Each cell was checked against that tool’s source or documentation; the
[performance measurements](docs/performance-measurements.md) compare their run times.
✅ means the tool does what the row names, text alone means partial or different support,
❌ means none (any text says what the tool does instead), and — means the row does not
apply:

| Feature | fdu | du | ncdu | dust | dua | gdu | pdu | diskus | dumac | scc | tokei |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Plain total | ✅ `--view=summary` | ✅ `-s` | ❌ TUI or export | ✅ `-d 0` | ✅ total row | ✅ `-ns` | ✅ `-d 1` | ✅ | ✅ | —² | —² |
| Tree breakdown and pruning | ✅ depth, breadth, minimum share, row limit | ✅ depth, size floor | TUI browsing | ✅ depth, top N, size floor | depth; TUI browsing | depth, top N files; TUI browsing | ✅ depth, minimum share | ❌ | ❌ | ❌ per language or file | ❌ per language or file |
| `.gitignore` | ✅ classify; include, exclude, or only ignored | ❌ | ❌ | ❌ | partial: TUI dims ignored entries; `--ignore-from` patterns | ❌¹ | ❌ | ❌ | ❌ | ✅ exclude | ✅ exclude, inside a git repository |
| Source code analysis² | ✅ 15 languages: code, comment, and blank lines; per directory in the Rust API | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ 366 languages; complexity and cost estimates | ✅ 333 languages; embedded languages |
| Text analysis | ✅ lines, words, paragraphs, pages | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| APIs and machine output | ✅ Rust, Python; JSON, JSONL, YAML | tab-separated text; `-0` | JSON export | JSON (`-j`) | Rust library; snapshot files | JSON export; SQLite or Badger | ✅ Rust library; JSON | Rust library | ❌ | ✅ Go package; JSON, CSV, HTML, SQL | ✅ Rust library; JSON |
| Watch and stream | ✅ `--watch`, JSONL change stream | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Cached results | ✅ snapshot and content cache, revalidated | ❌ | export, not revalidated | ❌ | snapshot, not revalidated | database, not revalidated | JSON, not revalidated | ❌ | ❌ | ❌ | ❌ |
| Agent skill | ✅ `--install-skill` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | MCP server (`--mcp`) | ❌ |

**When to use each:** ncdu, dua, and gdu let you browse a tree and delete from it
interactively, which fdu does not; gdu can also serve a browser view, and `dua clean`
finds build products to remove.
`du` is already installed on every Unix-like system.
diskus and dumac answer one total from a small binary, and pdu draws a compact size
chart.
For line counts alone, use scc or tokei: they count hundreds of languages to fdu’s
15, tokei counts embedded code, and scc estimates complexity;
[source-line counting](docs/performance-measurements.md#source-line-counting) compares
their run times with fdu’s. Use fdu for a tree you can prune, a `.gitignore`-aware
answer, content metrics, versioned machine output, a live or cached view, or a Rust or
Python API; for code, it adds each language’s ignored share, and per-directory counts in
Rust.

¹ gdu’s unreleased main branch adds `--ignore-from-gitignore`, which reads patterns from
one file.

² [scc](https://github.com/boyter/scc) and [tokei](https://github.com/XAMPPRocky/tokei)
count source lines, not disk usage, so the disk-usage rows show “—”. They count far more
languages than fdu; tokei also counts code embedded in another language, such as
Markdown code fences, and scc estimates complexity and cost.
The [SLOC tools survey](docs/project/research/research-2026-09-29-sloc-tools-survey.md)
compares their counting conventions and supported languages.
[cloc](https://github.com/AlDanial/cloc) recognizes the most languages, 402, but runs as
a single Perl process by default.

Versions checked for the feature cells: GNU coreutils `du` 9.4, and its source after
9.12; ncdu 1.19 and 2.9.2; dust 1.2.5; dua 2.45.0; gdu 5.37.0, and its main branch at
`4b179b0`; pdu 0.24.0; diskus 0.9.0; dumac at `1ffbe3c`; scc 4.1.0; tokei 15.0.0. dumac
runs only on macOS and ncdu only on Unix-like systems; the others run on macOS, Linux,
and Windows, `du` through a Unix layer such as MSYS2.

**Beyond this table:** A wider survey covered fifteen tools: the ten above, and
[dut](https://codeberg.org/201984/dut), [duc](https://github.com/zevv/duc),
[fsearch](https://github.com/cboxdoerfer/fsearch),
[bfs](https://github.com/tavianator/bfs), and [fd](https://github.com/sharkdp/fd).
Several save a scan to reload later (ncdu, gdu, and pdu export one; gdu, duc, and
fsearch keep a database; dua writes snapshots), but none revalidates a saved scan by
modification time, tallies file types per directory, or caches content metrics between
runs, and none is a native library with a live change feed that a Rust or Python program
can hold.
[The file roll-up engine research](docs/project/research/research-2026-08-06-file-rollup-engine.md)
has that survey, and
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
as `uvx --no-build fdu@0.5.0 .`.

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

- [Usage guide](docs/usage.md): views, analyzers, selection, cache, watch, and exit
  status
- [Machine output](docs/machine-output.md): JSON, JSON Lines, and YAML fields and
  schemas
- [Python package](crates/fdu-py/README.md) and Rust API on
  [docs.rs](https://docs.rs/fdu)
- [Performance measurements](docs/performance-measurements.md)
- [Documentation index](docs/README.md): architecture, research, and release process
- [Changelog](CHANGELOG.md) and [security policy](SECURITY.md)

## Development

```shell
make check    # handoff gate: fmt, clippy, tests, docs, lib-only build
make test     # Rust tests plus the CLI golden contract
make fix      # formatting and machine-applicable lint fixes
```

[AGENTS.md](AGENTS.md) is how to operate on the repository: toolchain versions, the
output design, the supply-chain policy, and the performance loop, which is deliberately
outside `make check`. Some tests need a host that enforces file permissions and delivers
native file events; AGENTS.md says how to run on one that cannot.

## License

MIT. See [LICENSE](LICENSE).

Designs adapted from GPL-licensed tools ([dut](https://codeberg.org/201984/dut)’s
atomic-refcount roll-up, [fsearch](https://github.com/cboxdoerfer/fsearch)’s record
layout) are clean reimplementations written from the descriptions in
[the file roll-up engine research](docs/project/research/research-2026-08-06-file-rollup-engine.md),
not transliterated from their source.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
