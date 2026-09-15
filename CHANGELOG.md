# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - YYYY-MM-DD

<!-- Release date: set when v0.1.0 is tagged -->

The first release. fdu walks a directory tree once and answers, for every directory at
once, how big it is, how many files it holds, what changed most recently, and what kinds
of files it contains.
It caches that index between runs and can keep it live.
The same engine ships three ways: the `fdu` command line, the `fdu` Python package, and
the `fdu-core` Rust crate, which the `fdu` crate re-exports.
The GitHub release text is
[docs/project/release-notes/0.1.0.md](docs/project/release-notes/0.1.0.md).

### Added

- **Command line.** `fdu PATH` prints a size-sorted tree two levels deep with ten rows
  per directory; bare `fdu` prints help and scans nothing.
  - Every option belongs to one axis: scope (`PATH`, `--scan-depth`,
    `--one-filesystem`), content (`--analyze`), selection (`--include`, `--exclude`,
    `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--depth`,
    `--limit`, `--sort`, `--reverse`, `--size`), view, format, and mode (`--cache`,
    `--watch`, `--analysis-workers`). `--depth` and `--limit` bound only what is
    rendered; `--scan-depth` bounds what is scanned.
  - `--view` takes `summary`, `tree`, `families`, `types`, `extensions`, `languages`,
    `documents`, `largest`, `recent`, `files`, or `full`. Several views in one run share
    one walk. `largest` and `recent` are the 20 largest and most recently modified
    regular files; `files` lists every matching entry; `full` is every view but `files`.
  - `--format text|json|jsonl|yaml`, with color decided by `--color auto|always|never`,
    `NO_COLOR`, and `FORCE_COLOR`. Results go to stdout, warnings and errors to stderr,
    and one-shot text reports end with a gray performance line.
  - Exit status 0 means a complete result, 1 a failed command, and 2 a partial result or
    a usage error. `--allow-partial` accepts a partial result as success.
  - `fdu --docs` prints the usage guide and `fdu --skill` prints a portable agent skill,
    both without a `PATH` and without scanning.
- **Machine output.** Every report carries a versioned `schema`: a metadata-only report
  uses one report schema, a report that ran content analysis or includes a metric
  summary (`types`, `families`, `languages`, `documents`) uses the next, and a `--watch`
  stream uses `fdu.stream/1`.
  <!-- PR A / PR B: confirm the final names.
  On main they are fdu.report/4 and fdu.report/5; PR A's branch moves them to
  fdu.report/5 and fdu.report/6, and PR B may move them again.
  -->
  A field change bumps the schema version.
  Completeness (`complete`, `errors`) is separate from what a view chose not to render,
  and a path that is not valid Unicode keeps a lossless, platform-tagged raw identity.
- **Content analysis**, opt-in with `--analyze`, which takes `lines`, `code`, `words`, a
  comma-separated set of them, `none`, or `all`.
  - `lines` counts physical, blank, and nonblank lines and raw words; `code` adds
    `code-sloc-v1` code, comment, and blank lines for Rust, Python, JavaScript,
    TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, shell, and SQL; `words`
    adds normalized words, paragraphs, and derived pages, and reader-visible words for
    Markdown.
  - Binary files, invalid UTF-8, and code without a shipped SLOC analyzer are reported
    as coverage and do not make a run partial; an I/O failure or a file that changes
    while it is read does.
  - Classification stays path-only for exact filenames and known extensions.
    Only unresolved paths and `.h` files receive bounded probes for shebangs, modelines,
    binary signatures, and generated-file markers, and every metric row reports how its
    files were detected and with what confidence.
  - A content sidecar keyed by the analyzer set keeps results between runs, so an
    unchanged file is not reopened and a stored set answers any narrower request.
  - `--view` never turns on an analyzer, and a view that displays none of what was read
    says so.
- **Cache.** By default a report or open saves a snapshot of its index under the user
  cache directory: `$XDG_CACHE_HOME/fdu` when that is set, otherwise
  `~/Library/Caches/fdu` on macOS, `~/.cache/fdu` on other Unix systems, and
  `%LOCALAPPDATA%\fdu` on Windows.
  - `--cache auto|refresh|read-only|only|off` chooses how a run uses it, and
    `--cache-status[=root|all]` and `--cache-clear[=root|all]` inspect and remove
    snapshots without scanning.
  - A snapshot serves only the scan scope that wrote it, and a corrupt, truncated, or
    unrecognized file is treated as absent.
    `--cache-clear` never deletes a file it does not recognize, and Unix snapshots are
    created owner-only (`0600`).
  - An unfiltered `--view summary` retains no index and writes no snapshot.
    <!-- PR B: confirm. With .gitignore observed by default, the summary tier falls
    closed to the full index (fdu-elnn Q7); say what it retains and saves.
    -->
- **Watch.** `fdu --watch` repeats the same query as the tree changes.
  Aggregate views repaint at most every `--interval` (2 seconds by default), and
  `--view files --format jsonl` emits one `fdu.stream/1` record per change.
  Events are verified by stat, and a backend overflow or rescan request becomes a
  reconcile of the affected subtree rather than a dropped event.
  A watch is metadata-only and refuses `--analyze`.
- **`.gitignore` roll-ups.** An index that observes `.gitignore` files keeps ignored and
  unignored roll-ups for every directory beside the totals.
  - Every `.gitignore` inside the scanned root governs its own directory and everything
    below it, and deeper files take precedence.
    Negation, anchored and directory-only patterns, `**`, and git’s bracket expressions
    follow git, pinned against verdicts recorded from `git check-ignore`. Matching is
    byte-exact and case-sensitive on every platform.
  - A watch or refresh that sees a `.gitignore` change re-reads its rules.
  - An index that did not observe control state says so: `Index::is_ignored`,
    `controls`, and the partition accessors return `Error::ControlStateNotObserved`
    rather than calling every entry unignored.
  - `ScanConfig::read_controls` in Rust and `ScanOptions.read_controls` in Python turn
    observation off for one request.
    <!-- PR A: draft from the fdu-1onj, fdu-okne, and fdu-szkg decisions and PR A's branch; confirm against the merged PR. -->
  - Identical `.gitignore` contents are stored and charged once.
  - Rules are retained up to a budget, 4 MiB by default.
    A `.gitignore` past the budget, or with a line longer than 16 KiB, is refused: its
    rules do not apply, every size stays exact, and the result stays complete with exit
    status 0. The report names the directories whose rules were refused, in its
    `ignore_rules` field and in a text note that names the knob.
  - `--gitignore-budget SIZE|all` on the command line, and `control_budget` on Rust
    `ScanConfig` and `OpenOptions` and Python `ScanOptions` and `OpenedOptions`, raise
    the budget or lift both bounds.
    The budget is part of the snapshot scope.
    <!-- /PR A -->
    <!-- PR B: draft from the fdu-elnn and fdu-5ryb decisions; confirm against the merged PR. -->
  - Every surface observes `.gitignore` by default: `fdu PATH`, `fdu --watch`,
    `fdu.report`, `fdu.open`, `fdu.scan`, `fdu_core::open`, and opened roots.
    `--no-gitignore` turns it off on the command line and changes the cache scope.
  - Text summary, tree, and extension rows say how much of each size is ignored, such as
    `(340 MiB ignored)`, and say nothing when none is.
    In machine formats those rows and file rows carry the split, as a zero when nothing
    is ignored and `null` when `.gitignore` was not read.
  - `--exclude-ignored` and `--only-ignored` select entries by ignore state, and sorting
    and `--min-size` use the size a row displays.
    Either filter with `--no-gitignore` is a usage error.
  - A `.gitignore` that cannot be read makes the result partial, exit status 2 unless
    `--allow-partial`.
    <!-- /PR B -->
- **Python package.** `import fdu` gives typed, immutable options, reports, roll-ups,
  provenance, cache values, and change feeds.
  - `fdu.report` answers one query while retaining the least state it needs; `fdu.open`
    and `fdu.scan` return an `Index` with `total`, `rollup`, `children`, `provenance`,
    `report`, `refresh`, `since`, and `watch`.
  - `Report.as_dict()` returns the command line’s JSON. `cache_path`, `cache_status`,
    `list_caches`, `clear_cache`, and `clear_all_caches` manage snapshots.
  - Native calls are bulk, and open, scan, and refresh release the GIL. A failure that
    stops an operation raises an `FduError` subclass, and one that makes a scan partial
    is reported on `Status`.
  - The wheel installs an `fdu` console script that runs the native command line.
- **Opened roots for interactive clients.** `OpenedIndex::open` in Rust, and
  `fdu.opened.OpenedIndex.open` in Python, return while discovery continues.
  - `read()` answers several projections (`Lookup`, `RollUp`, `Tree`, `Flat`,
    `Aggregate`, `Report`, `Continue`, `Diagnostics`) from one engine version, with
    bounded pages and opaque continuations.
    One projection can be refused, as `ProjectionResult::Refused` or Python
    `RefusedResult`, while the others answer.
  - `changes()` resumes an exact change journal from a cursor.
    `journal_capacity_bytes` bounds it, 8 MiB by default and at least 64 KiB, and a
    consumer that falls further behind receives a reset outcome and re-reads.
  - `refresh()` verifies named paths, `prioritize()` steers discovery, observation keeps
    the root live, and `close()` joins every worker.
    A worker panic surfaces as `OpenedWorkerPanicked`.
  - Selection globs match the portable path each row carries, so a path taken from a
    page can be passed back as a filter.
  - Options cover hidden-name pruning with an allow list, special-file exclusion, a file
    budget, and a custom file-type registry (File Rollup registry schema 3 or 4).
- **Rust library.** `fdu-core` is the engine, and `fdu` re-exports it alongside the
  command line.
  - `open` returns an `Index` whose per-directory roll-ups come from pre-computed state,
    and `prepare_report` answers one report without retaining an index.
  - Producers submit verified observations, and the index commits exact, clocked change
    batches that `Index::since` reads.
  - `watch` is the only build feature: off by default in `fdu-core`, on by default in
    `fdu`. `.gitignore` handling is always compiled in.
  - The minimum supported Rust version is 1.85.
- **Packaging.** The `fdu-core` and `fdu` crates; the `fdu` Python source distribution;
  and one CPython 3.12+ `abi3` wheel for each of Linux x86-64 and arm64 (manylinux2014,
  glibc 2.17), macOS x86-64 and arm64 (macOS 11), and Windows x86-64. Wheels carry type
  information, license text, and a CycloneDX SBOM.

### Upgrading from a pre-release build

This applies only to anyone who ran fdu built from a development checkout.

- **Each cached tree scans cold once.** A snapshot written by a development build from
  before the release does not serve 0.1.0: the snapshot format changed, and so did the
  type-rule and ignore-rule fingerprints that scope a snapshot.
  The first run on each previously cached tree scans cold and replaces the snapshot when
  that run saves one.
  <!-- PR A: the snapshot format moves to version 4. Confirm on the release candidate
  that a snapshot written by a pre-release build is refused rather than served
  (fdu-apbl). -->
- **fdu does not remove old snapshots.** <!-- PR A, true once the format bump lands -->
  `--cache-clear` and `--cache-clear=all` delete only snapshots this build recognizes,
  so a snapshot in an earlier format stays until a run on the same root replaces it.
  `fdu --cache-status=all --format json` lists such files with `"recognized": false`;
  delete them from the cache directory by hand to reclaim the space.
- **The `gitignore` build feature is gone.** A dependency declaring
  `features = ["gitignore"]` fails to resolve; remove it.
  `fdu`’s default build features are `["watch"]`.
- **Renamed and removed interfaces:**
  - `--view all` is `--view full`.
  - The report schema versions moved.
    <!-- PR A / PR B: name the final versions -->
  - `journal_capacity` on Rust `OpenOptions` and Python `OpenedOptions` is
    `journal_capacity_bytes`, a byte budget; a value below 64 KiB is refused.
  - `Index::is_ignored`, `controls`, `partition_total`, `partition_rollup`, and
    `partition_rollup_summary` return `Result`, and `ChildSnapshot.ignored` is
    `Option<bool>`.
  - `Error::NotADirectory` and `Error::ContinuationRecordLimit` are removed; those cases
    refuse one projection instead, and a Python `Tree` of a non-directory returns a
    `RefusedResult` instead of raising `InvalidArgumentError`.
  - `EntrySelection` refuses a terminal suffix or ancestor name that can never match,
    with `Error::InvalidValue`. Opened-root selection globs match portable paths, so a
    native spelling such as `100%.txt` for `100%25.txt` matches nothing there.
  - <!-- PR A --> `Error::ControlSourceLimit` and `Error::ControlPatternLimit` are
    removed, `MAX_CONTROL_TABLE_BYTES` and `MAX_CONTROL_PATTERN_BYTES` are
    `DEFAULT_CONTROL_BUDGET` and `CONTROL_LINE_GUARD_BYTES`, and `ControlTable::upsert`
    returns a `ControlAdmission`.
  - <!-- PR B --> The command line and `fdu.report` read `.gitignore` by default; pass
    `--no-gitignore` or `read_controls=False` to scan without it.

### Known limitations

- **Memory.** Views other than an unfiltered summary retain the whole index, so peak
  memory on a large tree can exceed that of a tool that builds a throwaway tree, such as
  dust.
- **Cache retention.** Nothing prunes snapshots of roots that are never scanned again,
  or bounds the cache directory’s size.
- **Cache scope.** <!-- PR B, confirm --> `fdu PATH` and `fdu --no-gitignore PATH` keep
  snapshots of different scope at one path, so alternating them scans cold each time.
- **Content analysis** is one-shot: `--watch` is metadata-only, and a refresh reanalyzes
  after reconciling. SLOC covers 15 languages, with no embedded-language or syntax-tree
  metrics. Sidecars and coverage are scoped to the analyzer set, so a request for
  analyzers the stored set lacks reads the files again.
- **`.gitignore` fidelity.** `.git/info/exclude`, `core.excludesFile`, and `.gitignore`
  files above the scanned root are not read, and a nested repository is not a boundary.
  A `.gitignore` inside an ignored directory is still read, which git skips.
  Three unusual patterns match differently from git: `a/\/b`, `a//b`, and `***` between
  separators.
- **Ignored shares** <!-- PR B, confirm --> appear in summary, tree, and extension rows
  only; `types`, `families`, `languages`, and `documents` rows do not show them, though
  the ignore filters apply there.
- **Watch backends.** A watch uses the platform’s native event backend on every
  filesystem, with no polling fallback for network filesystems that do not deliver
  events.
- **Opened roots.** A `Tree` page can exceed its `max_work` by the width of one
  directory level.
- **Roll-up metrics** are a fixed set; there is no interface for custom per-directory
  reducers.
- **Performance evidence** comes mainly from an M1 Pro MacBook with a local APFS SSD.
  Linux measurements are from virtualized hosts, Windows has none, and CI checks
  behavior rather than timing.
- **Platform coverage.** Linux arm64 wheels are cross-built and inspected, not executed,
  before release. There is no wheel for free-threaded CPython, musl Linux, or Windows
  arm64; those systems build from the source distribution with Rust 1.85 or newer.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
