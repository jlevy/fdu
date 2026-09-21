# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Directory filters now measure eligible subtree bytes and modification activity,
  enabling stale `.venv`, `node_modules`, and Cargo `target` inventories.
  Exclusions win throughout the subtree; nested rows may overlap while aggregate totals
  count covered contents once.
  This changes existing requests: `--min-size`, `--modified-since`, and
  `--modified-before` without `--kind` now test a directory’s subtree rather than its
  inode, and a directory they match covers its contents in aggregate views, so
  `--modified-since 7d --view summary` counts every file under a directory with recent
  activity; add `--kind file` for the files alone.
  A directory whose subtree was not listed in full, at a `--scan-depth` boundary or in a
  scan that finished with errors, reports `complete: false`, lower-bound sizes, an
  unknown age, and matches no modification bound.
- Metadata defaults to the `list` view with the existing tree output unchanged.
  `--format tree|paths|long` and `--tree`/`--long` expose tree, flat paths, and size/age
  columns. Flat lists are complete by default.
  Legacy Files/Tree presets remain.
- Core and Python queries select presentation before projection.
  Flat rows include subtree counts, `complete`, and signed `age_ns`, with a fixed
  `age_reference_ns` in the report.
  The default report’s machine `view` label is now `list` rather than `tree`, and its
  default section is a flat `files` array; `full` and `--view tree` keep `view: tree`.
  Report schemas advance to `fdu.report/7` and `/8` for these changes; snapshots are
  unchanged. Rust `report_format::render` now returns a Result to reject incompatible
  conversion between a folded tree and flat inventory.
  Python maps this to InvalidArgumentError.
  See [usage](docs/usage.md) and the [machine schema](docs/machine-output.md).

## [0.1.0] - 2026-09-16

<!-- Release date: 2026-09-16 is a placeholder. Set it to the tag date if v0.1.0 is cut
on a later day.
-->

The first release. fdu walks a directory tree once and answers, for every directory at
once, how big it is, how many files it holds, what changed most recently, and what kinds
of files it contains.
It caches that index between runs and can keep it live.
The same engine ships three ways: the `fdu` command line, the `fdu` Python package, and
the `fdu-core` Rust crate, which the `fdu` crate re-exports.
The GitHub release text is
[docs/project/release-notes/0.1.0.md](docs/project/release-notes/0.1.0.md).

### Added

- **Request model.** The command line, the Python package, and the Rust library share
  one typed `Request` (`Basis`, `Query`, `now`), one `Delivery`, the axis grammars, and
  `Request::DEFAULTS`. Allocated size is the default everywhere: `SizeMetric::default`
  and an opened selection that names no metric answer in allocated bytes, as `--size`
  and Python `size` already did.
  A refused request is a usage error: exit status 2 on the command line,
  `InvalidArgumentError` (a `ValueError`) in Python.
  `Query.axes` is `&'static AxisNames`, so a refusal names flags or fields in the
  caller’s vocabulary.
- **Command line.** `fdu PATH` prints a size-sorted tree two levels deep with ten rows
  per directory; bare `fdu` prints help and scans nothing.
  Sizes are allocated bytes unless `--size apparent` asks for file lengths.
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
  uses `fdu.report/5`, a report that ran content analysis or includes a metric summary
  (`types`, `families`, `languages`, `documents`) uses `fdu.report/6`, a `--watch`
  stream uses `fdu.stream/1`, and `--cache-status` carries `fdu.cache/2`, its own
  document identity rather than a report schema, with the identity of every tier each
  cached file holds. A field change bumps the schema version.
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
    unchanged file is not reopened; it answers only the analyzer set that wrote it.
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
    Unix snapshots are created owner-only (`0600`).
  - Cache status reports every file in the cache directory with a `state`: `current`,
    `stale` for a snapshot another fdu version or format wrote or a header this build
    cannot read, `leftover` for one of fdu’s own files left by an interrupted write,
    `unrecognized` for a file that is not fdu’s, and `absent` for a root with no
    snapshot. A stale row names why (`older_format`, `newer_format`, `other_engine`,
    `unreadable`) and a leftover row what it is.
  - `--cache-clear` and `--cache-clear=all` remove stale snapshots as well as current
    ones, so an upgrade’s invalidated snapshots are reclaimed rather than deleted by
    hand, and `--cache-clear=all` also reclaims fdu’s own leftovers: a staging file a
    killed writer never renamed, once it is older than the age its reaper uses, and a
    content sidecar whose snapshot is gone.
    A file is removed only if its name and its magic both say it is fdu’s; a symbolic
    link or a directory is never followed or removed, and an unrecognized file is always
    left in place and reported.
  - An unfiltered `--no-gitignore --view summary` is the one composition that retains no
    index and writes no snapshot.
    Reading `.gitignore`, which is the default, needs the index to classify entries, so
    an unfiltered `--view summary` retains it and saves a snapshot like any other
    report.
  - Cache-only content restore avoids repeated roll-up, candidate, classification, and
    path work, streams records into the retained index, and repeated unfiltered metric
    views share one entry traversal.
    Individual development comparisons on exploratory, uncontrolled, warm-steady M1
    Pro/APFS runs measured roughly 4% to 13% lower wall time for cache-only restore
    changes and 10% lower peak RSS from streamed restore.
    A separate workload that included setup plus 100 four-view reports measured 19%
    lower wall time. These results are not additive and do not measure an ordinary
    one-shot command or Linux; the
    [performance evidence](docs/project/reports/report-2026-08-20-fdu-performance-evidence.md)
    preserves each workload and interval.
- **Watch.** `fdu --watch` repeats the same query as the tree changes.
  Aggregate views repaint at most every `--interval` (2 seconds by default; the age
  grammar, including `200ms`), and `--view files --format jsonl` emits one
  `fdu.stream/1` record per change.
  Events are verified by stat, and a backend overflow or rescan request becomes a
  reconcile of the affected subtree rather than a dropped event.
  An upsert carries `ignored`, and so does a removal a rule edit caused; an ordinary
  removal, an invalidation, and every record of a run that read no rules omit it.
  A watch is metadata-only on every surface: the command line refuses `--analyze` with
  `--watch`, and a Rust `Session` or Python `Index.watch()` refuses an index opened with
  content analysis, as `Error::UnsupportedScanConfig` or `InvalidArgumentError`. The
  request model also refuses a narrowed scan scope and cache-only, so a library or
  Python caller cannot ask for what the command line refuses.
- **`.gitignore` roll-ups**, read by default on every surface.
  An index keeps ignored and unignored roll-ups for every directory beside the totals,
  and every report says how much of each size the tree’s own rules ignore.
  - Every `.gitignore` inside the scanned root governs its own directory and everything
    below it, and deeper files take precedence.
    Negation, anchored and directory-only patterns, `**`, and git’s bracket expressions
    follow git, pinned against verdicts recorded from `git check-ignore`. Matching is
    byte-exact and case-sensitive on every platform.
  - A watch or refresh that sees a `.gitignore` change re-reads its rules.
  - `fdu PATH`, `fdu --watch`, `fdu.report`, `fdu.open`, `fdu.scan`, `fdu_core::open`,
    `prepare_report`, and opened roots all observe control state.
    `--no-gitignore` on the command line, and `read_controls` off on Rust `ScanConfig`
    or Python `ScanOptions`, reads no rules for one request and is a snapshot scope of
    its own.
  - An index that did not observe control state says so rather than guessing:
    `Index::is_ignored`, `controls`, and the partition accessors return
    `Error::ControlStateNotObserved` instead of calling every entry unignored, and a row
    from such a run carries `null` rather than “nothing ignored”.
  - Text summary, tree, and extension rows end with the ignored part of their size, as
    `(128 B ignored)`, and leave it off a row with nothing ignored.
    The performance line counts the rule files read, as `ignore rules 1 file`, or says
    `no ignore rules`.
  - In machine formats summary and tree rows carry an `ignored` object (`files`, `dirs`,
    `bytes`, `allocated`), extension rows one without `dirs` (`files`, `bytes`,
    `allocated`), and file rows an `ignored` flag; each is `null` where no rule was
    read.
  - `--exclude-ignored` and `--only-ignored`, `Selection::ignored` in Rust and
    `Selection(ignored=...)` in Python, report one side; sizes, ordering, and
    `--min-size` follow the entries shown.
    Either filter over a run that read no rules is refused, not answered with an empty
    report: a usage error on the command line, `InvalidArgumentError` in Python, and
    `Error::ControlStateNotObserved` from `prepare_report` and a watch session.
  - A `--watch` stream maintains the entry set its selection names, so a rule edit that
    moves an entry into `--exclude-ignored` or `--only-ignored` streams the upsert that
    draws it and one that moves it out streams the removal, even though nothing about
    the file changed on disk.
  - A `.gitignore` that cannot be read makes the result partial, exit status 2 unless
    `--allow-partial`.
  - **Two independent limits bound the rules an index retains**, each a size or
    unbounded, and each liftable without moving the other.
    `--gitignore-budget SIZE|all` bounds the retained control state, 4 MiB by default;
    `--gitignore-line-limit SIZE|all` bounds one pattern, 16 KiB by default.
    They are `ControlLimits { budget, line_limit }` on `ScanConfig::control_limits` and
    `OpenOptions::control_limits`, and `control_budget` and `control_line_limit` on
    Python `ScanOptions` and `OpenedOptions`. Both are part of the snapshot scope.
  - A `.gitignore` past either limit is refused rather than ending the scan: its rules
    do not apply, every size stays exact, and the result stays complete with exit status
    0\. An opened root completes the directory and keeps watching, and a watch keeps
    applying events. Identical `.gitignore` contents are stored and charged once, so a
    tree of package checkouts spends a fraction of the budget it otherwise would.
  - A refusal is reported, not silent.
    Reports carry `ignore_rules` in every machine format: `null` when the run read no
    rules at all, otherwise `limits` (`budget` and `line_limit`, each bytes or `null`),
    the `applied` and `refused` counts, and `refusals`, each with a `path` and a
    `reason` naming the limit that fired.
    A text note names the refused files’ directories and the flag that lifts each limit
    that fired. `Index::control_coverage` and `ReadDiagnostics::controls` carry the same,
    Python mirrors both, and `EffectiveChange::ControlRefusalUpdated` reports a refusal
    recorded or lifted.
- **Python package.** `import fdu` gives typed, immutable options, reports, roll-ups,
  provenance, cache values, and change feeds.
  - `fdu.report` answers one query while retaining the least state it needs; `fdu.open`
    and `fdu.scan` return an `Index` with `total`, `rollup`, `children`, `provenance`,
    `report`, `refresh`, `since`, and `watch`.
  - `Report.as_dict()` returns the command line’s JSON. A row’s `ignored` is an
    `IgnoredTally` on `SummaryRow` and `TreeNode`, an `ExtensionTally` on
    `ExtensionRow`, and a `bool` on `FileRow`, each `None` where no rule was read.
    `Status.ignore_rules` reports the limits, what was applied, and what was refused.
  - `cache_path`, `cache_status`, `list_caches`, `clear_cache`, and `clear_all_caches`
    manage snapshots. A `CacheStatus` carries `state`, with `stale_reason`,
    `format_version`, and `leftover_kind` where they apply.
    `clear_cache` returns whether it removed a snapshot, and `clear_all_caches` returns
    a `ClearSummary` counting the snapshots it removed and the leftovers it reclaimed.
  - Native calls are bulk, and open, scan, and refresh release the GIL. A failure that
    stops an operation raises an `FduError` subclass, and one that makes a scan partial
    is reported on `Status`.
  - The wheel installs an `fdu` console script that runs the native command line.
    The script restores `SIGINT` to the default disposition before entering the native
    CLI, so Ctrl-C interrupts `--watch` the way it does a `cargo install` binary.
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
    A worker panic surfaces as `Error::OpenedWorkerPanicked` in Rust and as
    `OpenedIndexError` in Python.
  - Selection inside a read matches the portable path, in which `%` is written `%25` and
    a byte that is not UTF-8 is percent-escaped, so `100%.txt` is matched as
    `100%25.txt`. The `portable_path` a `Lookup`, `Tree`, or `Flat` row carries passes
    back as a filter unchanged.
    A `Report` projection’s rows carry only the native path, so a name containing `%` or
    a byte that is not UTF-8, taken from one, does not.
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
  - Published `fdu-core` requirements are caret ranges of the reviewed minimum.
    `Cargo.lock` still pins exact versions for this workspace and for
    `cargo install --locked`.
- **Packaging.** The `fdu-core` and `fdu` crates; the `fdu` Python source distribution;
  and one CPython 3.12+ `abi3` wheel for each of Linux x86-64 and arm64 (manylinux2014,
  glibc 2.17), macOS x86-64 and arm64 (macOS 11), and Windows x86-64. Wheels carry type
  information, license text, and a CycloneDX SBOM.

### Upgrading from a pre-release build

This applies only to anyone who ran fdu built from a development checkout.

- **A cached tree can scan cold once.** A snapshot is keyed on an engine fingerprint
  that mixes the crate version with the snapshot format, now version 4, and on the
  type-rule and ignore-rule fingerprints that scope it.
  Development builds already carried version `0.1.0`, so the format and the scope
  decide: a snapshot written before format 4, or under different type rules or
  `.gitignore` settings, does not serve 0.1.0, and the first run on that tree scans cold
  and replaces it when the run saves one.
  A snapshot a development build wrote in format 4 under the same settings can be
  served, and that tree’s first run is warm.
  The crate version moves at every release, so each later upgrade costs one cold run per
  cached tree.
- **`fdu --cache-clear=all` reclaims the rest.** A root that is never scanned again
  keeps a file this build cannot read, and clearing now takes it: `--cache-clear` and
  `--cache-clear=all` remove stale snapshots as well as current ones.
  `fdu --cache-status=all` lists each one as `stale` with the reason first, so you can
  see what a clear would take before running it.
  Files that are not fdu’s are never removed, and are reported as left in place.
- **The `gitignore` build feature is gone.** A dependency declaring
  `features = ["gitignore"]` fails to resolve; remove it.
  `fdu`’s default build features are `["watch"]`.
- **Update names a development build used.** Flags, report schema versions, and Rust and
  Python interfaces were renamed before this release, and `### Added` gives each under
  its released name. A consumer pinned to a development build’s `fdu.report` version
  moves to `fdu.report/5` or `fdu.report/6`.

### Compatibility

0.1.x may add fields and variants to public Rust types such as `ReadProjection`,
`ProjectionResult`, `ProjectionRefusal`, `LimitedProjection`, `IssueKind`,
`ImpactDomain`, `Error`, `ReportRequest`, `TreePage`, `ReadResponse`, `RollUp`,
`Provenance`, `StateTransition`, `ReportSource`, `Attrs`, and `Query`. Those additions
are breaking under Cargo’s semver rules for exhaustive types; they land in 0.2 rather
than behind `#[non_exhaustive]` on 0.1.0.

### Known limitations

- **Memory.** fdu builds an exact index that later questions reuse, and every report but
  an unfiltered `--no-gitignore --view summary` retains it, so peak memory grows with
  the entries retained.
  [The 2026-09-16 tool comparison](docs/project/reports/report-2026-09-16-fdu-live-tool-comparison.md),
  run on the release candidate over a generated 1,000,001-entry tree, measured fdu’s
  depth-one tree report, with its cache off, near 285 MiB of peak RSS, against 29 MiB
  for dumac, 21 MiB for dua, and 641 MiB for dust; each tool was invoked under a
  comparison contract that returned only a total.
  `--no-gitignore --view summary` retains no index: on that tree it took 4.876 s at 15.0
  MiB, against 4.942 s at 285.7 MiB for the default `--view summary`, with identical
  totals. The default summary retains the index because classifying entries against
  `.gitignore` needs it.
  On a 328k-file checkout with many `.gitignore` files, four runs of the same command
  peaked at 68, 68, 101 and 128 MiB, against 12 to 14 MiB without the index.
  The cost is the index’s, so it follows the entries retained and how the allocator grew
  on that run, and is a range rather than a fixed multiple.
- **Links.** A symbolic link is listed as an entry but never followed, and adds nothing
  to a directory’s totals; no command-line option follows links, and a `ScanConfig` or
  `OpenOptions` with `follow_symlinks` set is refused.
  A file with several hard links is counted once for each path, where `du` counts it
  once.
- **Cache retention.** Nothing prunes snapshots of roots that are never scanned again,
  or bounds the cache directory’s size.
  `--cache-clear` takes a root or the whole directory, so there is no way to clear only
  the stale snapshots an upgrade left; and because a snapshot in a *newer* format is
  stale to an older build in exactly the way an older one is, running an older build’s
  `--cache-clear=all` removes a newer build’s snapshots.
  Status names every file and its state before anything is removed.
- **Cache scope.** `fdu PATH` and `fdu --no-gitignore PATH` keep snapshots of different
  scope at one path, so alternating them scans cold each time.
  Changing either `.gitignore` limit does the same.
- **Content analysis** is one-shot on every surface: the command line refuses
  `--analyze` with `--watch`, a Rust `Session` and Python `Index.watch()` refuse an
  index opened with analysis, and a refresh reanalyzes after reconciling.
  SLOC covers 15 languages, with no embedded-language or syntax-tree metrics.
  Sidecars and coverage are scoped to the analyzer set, so a request for analyzers the
  stored set lacks reads the files again.
- **Analysis memory.** `--analyze code` holds a whole file in memory while it analyzes a
  file of unknown type, and `--analyze words` does the same for Markdown and unknown
  types, so a very large such file raises peak memory by its size.
- **`.gitignore` fidelity.** `.git/info/exclude`, `core.excludesFile`, and `.gitignore`
  files above the scanned root are not read, and a nested repository is not a boundary.
  Three unusual patterns match differently from git: `a/\/b`, `a//b`, and `***` between
  separators.
- **`.gitignore` budget.** A `.gitignore` inside a directory an ancestor’s rules already
  ignore is still read and charged against the budget, which git never does.
  Classification stays right, because an ignored parent settles its descendants, but a
  tree whose ignored directories hold most of its rule files can reach the 4 MiB default
  and see refusals for rules that could not have changed a verdict.
  `--gitignore-budget` lifts it.
- **Ignored shares** appear in summary, tree, extension, and file rows only; `types`,
  `families`, `languages`, and `documents` rows do not carry one yet, though
  `--exclude-ignored` and `--only-ignored` do filter those views.
- **Watch backends.** A watch uses the platform’s native event backend on every
  filesystem, with no polling fallback for network filesystems that do not deliver
  events.
- **A watched row’s ignored bit.** Under `--exclude-ignored` or `--only-ignored` a
  stream keeps its entry set exact across rule edits.
  Under the default selection it keeps membership live but does not restate a row’s
  `ignored` bit after a rule edit, so re-read a listing when the bit itself matters.
- **Opened roots.** A `Tree` page can exceed its `max_work` by the width of one
  directory level. The opened-root types (`OpenedIndex` and its `ReadProjection`
  projections, `TreePage`, `ReadResponse`, and the `fdu.opened` types that mirror them)
  are expected to change in 0.2, as the `0.x` rule allows.
- **Roll-up metrics** are a fixed set; there is no interface for custom per-directory
  reducers.
- **JSON integers.** Fingerprints, option hashes, and nanosecond timestamps are JSON
  numbers. Values above 2^53 lose precision in JavaScript `JSON.parse` and any other IEEE
  754 binary64 consumer.
  Read them as strings, or use a parser that preserves integers, if exact identity
  matters.
- **Performance evidence** comes mainly from an M1 Pro MacBook with a local APFS SSD.
  Linux measurements are from virtualized hosts, Windows has none, and CI checks
  behavior rather than timing.
- **Platform coverage.** Linux arm64 wheels are cross-built and inspected, not executed,
  before release. There is no wheel for musl Linux or Windows arm64; those systems build
  from the source distribution with Rust 1.85 or newer.
- **Free-threaded CPython** (such as `3.14t`) is not supported.
  It cannot install the `abi3` wheels, and an installer there falls back to building the
  source distribution; if uv selects a free-threaded interpreter, pass `--python 3.14`
  or `--python 3.12`.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
