# fdu

**Fast, incremental file roll-up engine** — `fd` and `du`, read as “fast du”.

fdu answers, for *every* directory in a tree at once: how big is it, how many files does
it hold, what changed most recently, and what kinds of files live in it.
One walk, many metrics, cached between runs.

> **Typical macOS/APFS live performance:** FDU built a reusable exact index and ten-row
> tree over 901,963 entries in a **3.324-second median**, versus 5.657 seconds for pdu,
> 6.016 for dust, and 6.782 for Go gdu on an M1 Pro MacBook with a local SSD. See
> [the full comparison](#speed-and-the-cache).

> **Status: pre-release.** The observation/commit contract, bounded in-process change
> feed, cache lifecycle, applying reconciler, CLI, and Python wheel are tested end to
> end, and the measured-improvement loop described below is running.
> The portable walker has a bounded parallel pool; macOS additionally uses an audited
> `getattrlistbulk` backend.
> Local M1/APFS evidence is published below, while the controlled Linux and full release
> matrices remain open.
> See [the Phase 1 plan](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md).

## Five Common Reports

These commands cover the common cost and reporting choices:

| Report | Command | Regular-file content | Result |
| --- | --- | --- | --- |
| Language sizes | `fdu --view languages PATH` | Never read | One row per code type detected from exact filenames and extensions, with file counts, sizes, and exact byte shares |
| Languages and lines of code | `fdu --analyze code --view languages PATH` | Reads eligible files | The same language rows with standard code lines, comment lines, blank lines, code-line shares, and explicit unsupported coverage |
| All file types | `fdu --view types PATH` | Never read | Stable rows for code, prose, markup, data, binary, and unknown types classified from exact filenames and extensions |
| Folder sizes | `fdu PATH` | Never read | The default directory tree with rolled-up sizes and file counts; builds or refreshes the reusable metadata index |
| Fast totals only | `fdu --cache off --view summary PATH` | Never read | One aggregate row with total size, file count, and directory count; retains no index or cache |

`--view languages` selects the detected code-type roll-up.
By itself it classifies from exact filenames and extensions, reports byte shares, and
never reads file content.
Adding `--analyze code` authorizes streaming reads through the end of every eligible
file, adds standard lines of code, and uses code lines for the shares.
The view never turns on content analysis by itself.
Human text uses language names such as `CSS`, `C++`, `JavaScript`, and
`Protocol Buffers`; JSON, JSONL, and YAML retain stable lowercase IDs such as `css`,
`cpp`, `javascript`, and `protobuf` for scripts.

For metadata classification, `--view types` applies stable exact-name and extension
rules. The language roll-up uses those detected types and may refine unresolved or
ambiguous paths with bounded probes once analysis is enabled.
Use `--view extensions` when the raw filename extension is the desired grouping.
For folder sizes, `tree` is the default view, so `fdu PATH` is the complete command.
For the no-index totals path, both `--cache off` and the single unfiltered `summary`
view are required. Sizes use allocated bytes by default; add `--size apparent` for
logical file lengths.

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
The combination is unoccupied ground, and it is what a live file browser actually needs.

The full survey, with the techniques worth adapting and their sources, is in
[the file roll-up engine research](docs/project/research/research-2026-08-06-file-rollup-engine.md).

## Speed and the Cache

**Typical live scan:** on a self-contained 901,963-entry tree, a fresh FDU process with
its own cache disabled built a reusable exact index and ten-row tree in a **3.324-second
median**. Pdu and dust took 5.657 and 6.016 seconds, and Go gdu took 6.782 seconds.
FDU’s richer index-and-tree product was the fastest of every tree or index tool
measured. This was 12 adjacent paired trials per tool on an M1 Pro MacBook with a local
APFS SSD in a repeated-workload warm-steady filesystem-cache state.
One independent full-tree fingerprint and three warmups per tool preceded timing; this
does not claim that every metadata object remained resident.

The cache-off rich-summary plan took 3.125 seconds and beat diskus, dua, BSD du, and GNU
du. Dumac’s narrower allocated-byte total had a 2.980-second median, but its paired 2.2%
advantage was statistically unclear (95% interval -5.7% to +1.7%). FDU also returned
file and directory counts, apparent bytes, and newest file time while using 13.6 MiB
instead of dumac’s 44.4 MiB peak RSS. See the
[technical white paper](docs/project/reports/report-2026-08-12-fdu-performance-architecture.md),
[full comparison](docs/project/reports/report-2026-08-13-fdu-live-tool-comparison.md),
and
[reproduction manifest](docs/project/reports/fdu-live-tool-comparison-manifest-v2.json).

| Tool | Work returned | Typical median |
| --- | --- | ---: |
| **fdu** | reusable exact index and ten-row tree | **3.324 s** |
| **fdu** | five-tally exact summary | **3.125 s** |
| dumac | allocated-byte total only | 2.980 s (statistical tie) |
| pdu | rendered depth-one tree | 5.657 s |
| dust | rendered ten-row tree | 6.016 s |
| gdu | rendered ten-row tree | 6.782 s |
| diskus | scalar total only | 5.708 s |
| dua | scalar total only | 5.459 s |

fdu has two paths to an answer, and it labels which one you got.

**Without a usable cache, it is a fast walk and roll-up.** Every entry is enumerated and
statted once, and per-directory roll-ups accumulate as the walk proceeds.
That is the same job `du` does, plus the extra metrics, and it is bounded by syscall
count and storage latency.
For the existing `--cache off --view summary` composition, FDU now derives an exact
summary-only execution plan instead of retaining a reusable index.
On a frozen heterogeneous 978,339-entry run that improved paired wall time 14.56% and
cut peak RSS 95.28%, with identical stable report semantics (exp-040). Exact-binary
replications on uniform 720,805- and 901,963-entry trees reproduced the resource
mechanism but only 1.8–2.8% wall gains, showing that syscall-bound topology can hide
most of the saved user-space work.
Tree, filtered, multi-view, cached, and live requests still use the full index.

**With a usable cache, it can be much faster** — but only where the cache can supply
something the filesystem will not.
This is worth stating plainly, because it is where naive du-caches go wrong: change
information does not propagate up a directory tree.
An in-place file edit changes no directory’s mtime, not even its parent’s, so a
directory fingerprint proves that no entry was *added, removed, or renamed*, and nothing
about any child’s bytes.
The trustworthy floor for a warm run is therefore one stat per entry, and the cache pays
off decisively in the cases that beat that floor:

- **Environments where the OS metadata cache cannot hold the tree** — CI runners, cloud
  hosts, whole-drive scans.
  There the snapshot is not an optimization; it is the only warm state available.
- **Journal-assisted revalidation**, where the OS already recorded what changed.
  Replaying the macOS FSEvents journal would make a quiet tree’s warm start cost
  O(changes) rather than O(entries) — incremental on a warm laptop, decisive where no
  sweep can be fast. It is designed but not built, and cross-restart replay is
  Apple-documented yet unproven in shipping tools, so a spike validates it first; see
  [the FSEvents-scoped revalidation plan](docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
- **Expensive derived metrics** such as line counts, where an unchanged fingerprint
  skips re-reading the file entirely.

On a warm laptop against a mid-size tree, none of those apply, and a warm run is
currently *slower* than a cold one.
That inversion is measured rather than assumed, and closing it is the current work.

Speed changes here are decided by paired, interleaved measurement against a real tree,
with an independent oracle verifying that faster output is still identical output.
Every accepted and rejected experiment is recorded in
[the experiment ledger](docs/project/reports/report-2026-08-10-fdu-performance-experiments.md);
the key architectural conclusions are in
[the performance white paper](docs/project/reports/report-2026-08-12-fdu-performance-architecture.md),
and the protocol is [the performance loop](docs/project/guides/performance-loop.md).
The cost model, the platform levers that change constants by integer factors, and the
ranked backlog are in
[the performance frontier research](docs/project/research/research-2026-08-10-performance-frontier.md),
which draws on source review of bfs, dut,
[pdu](https://github.com/KSXGitHub/parallel-disk-usage),
[diskus](https://github.com/sharkdp/diskus), and
[jwalk](https://github.com/jessegrosjean/jwalk), plus
[dumac](https://healeycodes.com/maybe-the-fastest-disk-usage-program-on-macos)’s macOS
bulk-attribute design and
[follow-up scheduler and inode-sharding work](https://healeycodes.com/optimizing-my-disk-usage-program).

## Build Locally

```shell
cargo install --path crates/fdu
fdu --help
```

```shell
make python-smoke  # build the wheel; test its module, console script, and local uvx path
```

Publishing is Phase 1 work.
`cargo install fdu` and `uvx --from fdu==<version> fdu` are future commands; neither
package should be presented as available from crates.io or PyPI yet.

## Three Cost Layers

fdu separates **what it reads** from **how it reports the result**. `--analyze` is the
content-I/O switch; `--view` is a projection over the state that was requested.
A view never enables an analyzer implicitly.

| Layer | Representative command | Filesystem work | State retained |
| --- | --- | --- | --- |
| Exact summary | `fdu --cache off --view summary PATH` | Enumerate and stat every entry; never read file contents | Five aggregate tallies; no index or cache |
| Metadata index | `fdu PATH` | Enumerate and stat every entry; classify recognized paths without reading contents | Reusable parent-pointer index and, unless disabled, metadata snapshot v2 |
| Content index | `fdu --analyze PROFILE PATH` | Metadata work plus streaming reads through every eligible file missing from a compatible content sidecar | Metadata index plus sparse content roll-ups and a separate `.content` sidecar |

The summary-only plan applies only to one unfiltered `summary` view with `--cache off`.
Filters, multiple views, cache participation, watch mode, or content analysis fall
closed to the full index because they need paths, hierarchy, or reusable state.
The planner derives this internally; there is no separate “fast” flag whose semantics
can drift from the ordinary query.

With no `--analyze`, the default is strictly metadata-only:

- no regular file is opened for content
- no analyzer worker pool or sparse content index is created
- `tree`, `files`, `summary`, and `extensions` retain their metadata behavior
- `types`, `families`, and `languages` add path-only classification by exact filename
  and extension
- the content sidecar is not loaded, and analyzer settings do not alter metadata
  snapshot v2

Analysis profiles request nested bundles; each deeper bundle includes the basic pass.
Views select a report from the requested bundle:

| `--analyze` | Adds | Views that expose those metrics |
| --- | --- | --- |
| `none` | Nothing; the default metadata-only path | `languages` remains a byte and count view |
| `basic` | Physical, blank, and nonblank lines; raw prose words | `types`, `families`, `documents` |
| `code` | `basic` plus standard code, comment, and code-blank lines | `languages`; also the basic document report |
| `documents` | `basic` plus logical words, paragraphs, and reader-visible Markdown | `documents` |
| `full` | Every shipped analyzer | `languages,documents` together |

`languages` therefore works without analysis and uses byte shares; `code` or `full` adds
standard LOC and switches its shares to code lines.
`documents` requires any enabled profile.
The metadata views remain legal with every profile, so one command can compose byte,
type, code, and prose summaries over the same observed tree.
Content analysis is currently one-shot; `--watch` remains metadata-only and rejects an
enabled analysis profile.
Content analysis reads every eligible file through EOF; `--analysis-workers` bounds
concurrency, not coverage.
Known binary types are rejected before opening.
Invalid UTF-8, binary data, and code types without a shipped SLOC analyzer remain
explicit coverage outcomes: they still contribute file and byte totals and do not make
the run partial.
I/O failures, files that change during their read, and stale conditional
commits are operational failures; those make the result partial and produce a warning.
Selection flags such as `--include` shape the report, not the retained analysis scope.
An enabled profile analyzes eligible files in the chosen `PATH` and `--scan-depth` so
the resulting sidecar can serve later selections without rereading them.
Coverage records are profile-scoped today.
If a deeper analyzer such as standard LOC does not support a file, its byte metadata
remains visible but that profile does not retain a separate lower-level line result for
the same file.

## Reading the Performance Footer

Every one-shot text report ends with one compact operational summary:

```text
Performance: walked 12,345 files / 8.2 GiB; content read 1.4 GiB at 920 MiB/s; analysis 2,104 fresh at 1.3k files/s, 10,241 cached / 6.8 GiB; warm revalidation; total 2.08 s
```

“Walked” counts regular files successfully stated during this run and sums their
apparent lengths, independent of the report’s selected `--size` metric.
“Content read” counts bytes actually returned by fresh analyzer reads, so a known binary
file can be walked without being opened and an observed binary probe can read less than
the file’s full length.
Fresh file and byte rates use the content-analysis phase’s wall time.
Cached files and bytes are unchanged content records restored from the profile-scoped
sidecar. The final label distinguishes a cold metadata scan, warm revalidation, and a
cache-only answer; cache-only therefore reports zero files walked.

The line is gray when terminal color is active and contains no ANSI escapes when color
is disabled or redirected.
JSON, JSONL, YAML, skill output, lifecycle commands, and watch streams omit it.
The timing line is human telemetry rather than part of the versioned machine schema.

## Compose Other Queries

```shell
fdu --depth 3 ~/src                        # render three levels deep
fdu --view extensions ~/Downloads          # break down by raw file extension
fdu --analyze basic --view families .      # lines, blanks, words, and exact byte shares
fdu --analyze basic --view documents .     # text lines, words, paragraphs, and pages
fdu --format json .                        # stable, versioned machine output
fdu --view files --sort size -n 20 ~/src   # compose a largest-files query
fdu --skill                                # print the self-contained agent skill
```

A cache-oriented walkthrough on one tree is:

```shell
# Metadata only: compact one-shot totals, then reusable indexed views.
fdu --cache off --view summary PATH
fdu --cache refresh --view tree,extensions,types,families PATH
fdu --cache only --view tree,types PATH

# Opt into successively richer content bundles.
fdu --cache off --analyze basic --view types,families,documents PATH
fdu --cache off --analyze code --view languages PATH
fdu --cache off --analyze documents --view documents PATH
fdu --cache refresh --analyze full --view languages,documents PATH

# Reuse the exact same profile without touching source-file contents.
fdu --cache only --analyze full --view languages,documents PATH
```

```text
   156 KiB  ██████████   100%  . (12 files)
   140 KiB  █████████░    90%    fdu (10 files)
   136 KiB  █████████░    87%      src (8 files)
```

Reports never infer `.`: bare `fdu` prints the same help as `fdu --help` and performs no
scan, while `fdu .` opts into the current directory explicitly.
`--help` is the complete source of truth.
Human output uses restrained semantic color when its destination is a terminal;
`--color auto|always|never`, `NO_COLOR`, and `FORCE_COLOR` make the policy explicit.
Primary results go to stdout, while warnings and errors go to stderr.
Machine and skill output never contain ANSI styling.
Metadata-only machine reports retain the versioned `fdu.report/1` schema unless a
metric-summary view is requested.
Every explicit content request and the `types`, `families`, `languages`, and `documents`
metric summaries use `fdu.report/2`, adding exact share numerators and denominators,
analyzer coverage, and versioned rule, option, and analyzer identities.
An unavailable metric share is represented as `0/0` in machine output and `—` in human
output, never as a measured zero percent.
Every metric row also reports how its files were detected, the confidence of those
decisions, and generated, vendored, and documentation flags.
Scan completeness and each tree node’s rendered truncation are separate fields.
Invalid-Unicode paths retain their display string and add a lossless, platform-tagged
raw identity.
Exit status 2 means partial results; pass `--allow-partial` to accept those
as success. Exit status 1 means the command failed.

Content analysis is opt-in through `--analyze none|basic|code|documents|full`. The basic
profile streams each eligible file once, recognizes LF, CRLF, lone CR, and mixed line
endings, separates blank and nonblank lines, rejects NUL-containing and invalid UTF-8
files explicitly, and counts raw words only for prose and markup types.
Every eligible file is streamed through EOF. `--analysis-workers` bounds concurrent
readers, and `--words-per-page` controls only the report-time page denominator.
Content results use a separately versioned, profile-scoped sidecar, so an unchanged warm
run with the same profile and semantic settings does not reopen files.
Changing profiles can require content reanalysis, but it never invalidates the separate
metadata snapshot v2. The `code` profile adds the dependency-free `code-sloc-v1` state
machine for Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP,
Swift, Kotlin, shell, and SQL. It reports code, comment, and code-blank lines
separately, counts mixed lines as code, treats multiline strings and docstrings as code,
and uses code lines as the default language-percentage denominator.
Other code types remain visible as unsupported coverage rather than being mislabeled
from nonblank lines.
The `documents` profile adds FlexDoc-style normalized word counts, paragraph runs, and
pages derived after aggregation.
For Markdown it separately reports reader-visible words and excludes URLs, link
destinations, code, metadata, footnote markers, and hidden markup; `full` combines the
code and document analyzers.

When analysis is enabled, classification is also a cost ladder.
Exact filenames and recognized extensions stay path-only.
Only unresolved files and the ambiguous `.h` extension receive bounded probes for
shebangs, modelines, C++ literals, XML and manpage markers, binary signatures, and
generated-file markers.
For unresolved paths, NUL and named binary signatures take precedence over shebang and
modeline hints. A NUL found anywhere in any eligible read discards provisional text
metrics, and every deeper decision is explainable in `fdu.report/2` rather than silently
guessed.

This surface — composable views, selection filters, time-window and watermark queries,
cache policies, and a `tail -f`-style watch mode, all as orthogonal flags over one
grammar — is designed in
[the composable CLI and query surface plan](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md).
The principles it settled on, written as rules for extending it rather than as a record
of what was built, are in
[the design principles](docs/project/architecture/fdu-design-principles.md).
Why the cache can be a speed-up or a cost depending on platform and view is in
[the cache design](docs/project/guides/cache-design.md).

## As a Rust Library

```toml
[dependencies]
fdu = { path = "crates/fdu", default-features = false }
```

`default-features = false` skips the CLI’s dependency tree.
Add `features = ["watch"]` for the OS-native watch layer.

```rust
use fdu::{OpenConfig, open};
use std::path::Path;

let (index, report) = open(Path::new("."), &OpenConfig::default())?;
let total = index.total();
println!("{} files, {} bytes", total.files, total.bytes);

// Per-directory roll-ups are materialized from pre-computed state, with no tree walk.
if let Some(src) = index.rollup(Path::new("src")) {
    println!("src/: {} files, newest {}", src.files, src.newest_mtime_ns);
}
# Ok::<(), fdu::Error>(())
```

Opt into the complete code-and-document tier explicitly; metadata-only remains the
default:

```rust
use fdu::content::AnalysisProfile;
use fdu::{OpenConfig, open};
use std::path::Path;

let mut config = OpenConfig::default();
config.analysis.profile = AnalysisProfile::Full;
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
import fdu_py

index = fdu_py.open("/path/to/tree", analyze="full")
print(index.complete, index.freshness, index.errors)
print(index.total())          # {'files': ..., 'bytes': ..., 'by_extension': {...}}
print(index.children("src"))  # one call returns every child with its roll-up
print(index.report(views=["languages", "documents"]))

mark = index.clock
index.refresh()               # reconcile against the filesystem
print(index.since(mark))      # exactly what changed, or truncated=True if you fell behind
```

Every method is bulk: it returns a whole structured result in one call.
A million small zero-copy calls lose comfortably to one large call.
The same wheel also installs an `fdu` console script backed by the native Rust CLI. Once
a release is published, that makes an exact version directly runnable as
`uvx --from fdu==<version> fdu`; the local wheel and `uvx` path are already exercised by
`make python-smoke` without implying that a public release exists.

## How It Works

The metadata core and opt-in content layer retain separate state:

| Artifact | What it is |
| --- | --- |
| **Metadata index** | In-memory parent-pointer tree; every directory carries pre-computed size, count, recency, and extension roll-ups |
| **Metadata snapshot** | A complete metadata baseline, keyed by canonical root, semantic scan scope, format, and engine version |
| **Content index** | Optional sparse per-file analysis records and derived roll-ups, allocated only after `--analyze` opts in |
| **Content sidecar** | Separately versioned, profile-scoped persistence for unchanged content records; never loaded by metadata-only requests |
| **Observation** | Verified producer input, optionally conditional on the indexed path state |
| **AppliedDelta** | A clocked batch of effective committed changes for the bounded change feed |
| **Derived report** | Exact minimum state for a proven one-shot composition; otherwise the planner falls back to the index |

Metadata producers submit observations; the index alone removes no-ops, advances the
metadata clock, and mints `AppliedDelta`. Content workers submit fingerprint-checked
analysis observations to the optional derived tier without changing metadata truth or
snapshot compatibility.

Two invariants are non-negotiable, because a cache that lies is worse than no cache.
Content-reuse fingerprints are size, mtime, ctime, and inode, never mtime alone, because
mtime is user-settable and some applications roll it back after writing.
A corrupt or unrecognized snapshot is treated as absent, never as data.

Every value also carries its provenance: where it came from, when it was observed, and
whether it is final.
That is what lets a caller show a cached number immediately, label it honestly, and
clear the label as verification converges.

The serving model, the concurrency guards, and the full set of rules any change must
respect are in
[the design and principles doc](docs/project/architecture/fdu-design-principles.md).

## Development

```shell
npm ci             # install the exact development-only golden-test toolchain
make supply-chain  # verify release age, provenance, exact pins, and CI trust controls
make build         # debug build, all features
make test          # Rust tests plus the end-to-end CLI golden contract
make test-golden   # build and compare only the CLI sessions
make check         # tests, audits, docs, and installed-wheel smoke — the handoff gate
make fix           # apply formatting
```

The golden sessions are executable Markdown under `tests/golden/`, run by
[tryscript](https://github.com/jlevy/tryscript).
After an intentional CLI output change, run `make golden-update`; it regenerates
affected blocks and immediately reruns comparison.
Review the Markdown diff before committing.
The scenario design and the small set of permitted dynamic patterns are documented in
[the completed CLI golden-test plan](docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md).

Performance work has its own targets (`make perf-baseline`, `perf-profile`,
`perf-compare`, `perf-ledger`), deliberately outside `make check` — a timing gate on a
shared CI runner measures the runner.
Follow [the performance loop](docs/project/guides/performance-loop.md) before changing
anything for speed.

To set the project up from a fresh clone and prove the pieces work together by hand —
including that issue tracking survives a sync round trip with its comments intact —
follow [the integration runbook](docs/project/guides/integration-runbook.md).
It covers what `make check` cannot: the workflow around the code.

Read [the supply-chain policy](SUPPLY-CHAIN-SECURITY.md) before changing a dependency,
toolchain, CI action, or bootstrap download.
[The design and principles doc](docs/project/architecture/fdu-design-principles.md)
carries the rules worth not rediscovering, and [AGENTS.md](AGENTS.md) covers how to
operate on the repository.

## License

MIT. See [LICENSE](LICENSE).

Designs adapted from GPL-licensed tools ([dut](https://codeberg.org/201984/dut)’s
atomic-refcount roll-up, [fsearch](https://github.com/cboxdoerfer/fsearch)’s record
layout) are clean reimplementations written from the descriptions in the research doc,
not transliterated from their source.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
