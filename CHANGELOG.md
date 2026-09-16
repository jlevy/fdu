# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial scaffold: the architecture expressed in working, tested code.
  - **Observation/commit contract** (`types.rs`): producers submit verified, optionally
    conditional `Upsert` / `Remove` / `InvalidateSubtree` observations; the index emits
    clocked `Commit` batches containing exact effective mutations and state transitions.
  - **In-memory index** (`index.rs`): parent-pointer arena with per-directory
    pre-computed roll-ups (counts, apparent and allocated bytes, newest mtime,
    per-extension tallies), O(depth) apply, generation-safe free-slot reuse, explicit
    freshness, revision-safe conditional arbitration (including structural and ABA
    races), an operation-bounded change feed, and correct pre-Unix-epoch timestamp
    reduction.
  - **Scan layer** (`scan.rs`): portable cold scan plus applying full/subtree/shared
    reconciliation with stale-observation arbitration, exact scope matching, retryable
    invalidations with missing/non-directory ancestor widening, root-only depth-zero
    semantics, bounded producer batches, and enforcement of depth, symlink, and
    filesystem subtree boundaries.
  - **Snapshot** (`snapshot.rs`): semantic-scope invalidation, bounded streaming load,
    payload integrity checks, complete-only concurrency-safe atomic replacement, and
    corrupt-equals-empty semantics.
    Unix snapshots are created owner-only (`0600`).
  - **Watch layer** (`watch.rs`, build feature `watch`): notify-backed, coalescing then
    verifying by stat, with `Flag::Rescan` escalated to `InvalidateSubtree` rather than
    dropped, plus an apply/reconcile driver that closes invalidations.
    The applying driver re-verifies queued samples at a clock-stable commit boundary,
    rejects a watcher for another root, and rejects depth- and filesystem-restricted
    scopes until events can be filtered against those boundaries.
  - **CLI** (the `fdu` crate): composable scope, selection, view, format, and mode axes;
    compact human tree output; schema-versioned text/JSON/JSONL/YAML reports; cache
    lifecycle controls; and a `tail -f`-style watch stream.
    Reports require an explicit path, while bare `fdu` prints help without scanning the
    current directory.
  - **Python package** (`fdu`): typed immutable options, reports, roll-ups, provenance,
    cache values, and change feeds over a private `fdu._native` extension, with bulk
    calls and GIL release during native work.
    The same abi3 wheel includes watch support, a `py.typed` marker, exact extension
    stubs, the native CLI console script, license text, and a CycloneDX SBOM.
  - **Release rehearsal**: exact `0.1.0` identity checks, crate/sdist/wheel content
    inspection, installed-wheel and installed-sdist consumer tests, strict downstream
    type checking, and portable Linux, macOS, and Windows wheel build definitions.
  - **Opt-in content metrics** (`content`): compiled stable file-type rules; bounded
    parallel one-pass UTF-8, NUL, line, blank, and raw-word analysis; conditional
    fingerprint-checked commits; sparse type/family roll-ups; and independently
    checksummed atomic sidecars that preserve metadata snapshot v2. The `code-sloc-v1`
    dialect partitions code, comment, and blank lines for 15 common languages; logical
    prose and reader-visible Markdown add normalized words, paragraphs, and
    aggregate-derived pages.
    Bounded shebang, modeline, ambiguous-header, format-signature, and origin probes run
    only after path-only classification cannot decide.
  - **Metric summaries** (`fdu.report/2`): stable `types`, `families`, `languages`, and
    `documents` views with exact byte-share fractions, coverage, analyzer provenance,
    logical page denominators, detection source and confidence, origin flags, and
    matching Rust, CLI, and Python surfaces.
    The original extension view is retained as `extensions`, while metadata-only
    `fdu.report/1` output remains unchanged.

### Changed

- Corrected the fixed `.gitignore` matcher’s recursive, negation, and character-class
  semantics and bounded adversarial pattern work.
  Its semantic fingerprint is now version 2, so snapshots written with the earlier
  matcher are intentionally rejected and rebuilt from the filesystem.
- Runtime type-rule registries now fingerprint their validated semantic values instead
  of accepting an asserted identity.
  This intentionally invalidates earlier snapshots and content sidecars once; the next
  complete run rebuilds them under the verified registry identity.
- **Breaking:** the opened root’s journal budget is `journal_capacity_bytes`, on Rust
  `OpenOptions` and Python `OpenedOptions`, replacing `journal_capacity`, which counted
  retained items. It is measured in bytes as `Commit::retained_cost` estimates them: a
  fixed allowance per commit and per retained change, transition, or dirty path, plus
  each path’s bytes, so a budget means the same on every platform.
  The default, `DEFAULT_JOURNAL_CAPACITY_BYTES`, is 8 MiB. Opening a root refuses a
  budget below `MIN_JOURNAL_CAPACITY_BYTES`, 64 KiB, with an error naming the unit and
  the minimum (`InvalidArgumentError` in Python).
  The floor is the old item-count default, so any count passed as bytes is refused or
  works, and it holds about a hundred single-file commits.
- A name a directory listing returned that is gone by the time it is stat’d is recorded
  as deleted on every walk: cold scans, reconciliation, `revalidate`, watches, and
  opened-root discovery and refresh.
  A cold walk omits it and a reconciliation removes the retained entry, rather than
  reporting an I/O error that leaves the walk partial and, under a watch or an opened
  root, the entry permanently partial.
- Reconciling a retained `.gitignore` as the root of its own walk, as a watch event or
  an opened-root refresh naming the file does, re-reads its rules.
  An unreadable one keeps its previous rules and leaves the path partial.
- Opened-root lifecycle reporting:
  - A panicking worker wakes a blocked `changes()` poll, which returns
    `OpenedWorkerPanicked` after delivering the commits retained before the panic.
    A panic inside a commit poisons the index and leaves nothing to deliver; a poll
    parked on the journal names the panic, while a poll already reading at that moment
    can report the poisoned lock instead.
    `close()` reports the earliest failure, ranking a panic ahead of the poisoned lock
    it left behind.
  - A refresh or observation pass records each directory it listed as complete unless an
    error arose in that directory’s own listing, as discovery does, and a multi-path
    refresh closes each subtree on its own walk.
    One unreadable child therefore does not leave its sibling directories unknown below
    a complete root.
  - Published freshness stays `Reconciling` until the observation handoff reaches
    `Watching`.
  - A refresh that verifies the same facts as a concurrent producer applies as unchanged
    rather than as a lost race, so it does not send the observation handoff around again
    or fail the root. That includes a `.gitignore`’s rules as well as its entry.
  - A refresh on a `Failed` root keeps the issue that explains the failure.
- An index that did not observe `.gitignore` control state says so instead of calling
  every entry unignored.
  `ScanConfig::read_controls`, and `ScanOptions.read_controls` in Python, is on by
  default, and a request can turn it off for `open`, `open_with_pending_save`,
  `fdu.open`, `fdu.scan`, and a watch over their index.
  On an index that observes none, `Index::is_ignored`, `controls`, `partition_total`,
  `partition_rollup`, and `partition_rollup_summary` return
  `Error::ControlStateNotObserved`, `ChildSnapshot` carries no ignore bit or partitions,
  and `Index::apply` refuses control input with the same error.
  Breaking: those five accessors return `Result`, and `ChildSnapshot.ignored` is
  `Option<bool>`.
- **Breaking:** `.gitignore` handling is always compiled in, and the `gitignore` build
  feature is removed from `fdu-core` and `fdu`. `fdu`’s default build features are now
  `["watch"]`, and a dependent that asks for `features = ["gitignore"]` fails to
  resolve. `ScanConfig::read_controls` is the only switch for reading control files.
  A consumer that built without the `gitignore` build feature sees three changes:
  - A scope with `read_controls` on, which is the default, now observes control state.
    `Index::is_ignored`, `controls`, and the partition accessors answer instead of
    refusing, `ChildSnapshot` carries the ignore bit and partitions, the scan reads
    every `.gitignore` in the tree, and it can reach the control bounds.
  - Control input through `ControlTable::upsert` or `Index::apply` is applied, or
    refused with `Error::ControlStateNotObserved` on a scope that observes no control
    state, where it used to fail with `Error::UnsupportedScanConfig`.
  - A scope with `read_controls` on now has an ignore-rules fingerprint of its own
    rather than 0, so a snapshot written under it misses once, and the next scan of that
    root replaces it with a cold one.
- **Breaking:** a `.gitignore` past the control budget, or with a line over the line
  limit, is refused instead of ending the scan.
  Its rules do not apply, every size stays exact, and the result stays complete with
  exit status 0; the batch that carried it commits, an opened root completes the
  directory and keeps watching, and a watch keeps applying events.
  Identical `.gitignore` files are stored and charged once, so a tree of package
  checkouts uses a fraction of the budget it did.
  - `Error::ControlSourceLimit` and `Error::ControlPatternLimit` are removed.
    `ControlTable::upsert` returns a `ControlAdmission`, `Index::control_coverage`
    returns the limits, the applied and refused counts, and at most
    `MAX_RETAINED_ISSUES` refused files with an exact count and the limit that refused
    each, and `EffectiveChange::ControlRefusalUpdated` reports a refusal recorded or
    lifted. `ReadDiagnostics::controls` carries the same for an opened root, and Python
    mirrors all three.
  - Two independent limits, each a size or unbounded, and each liftable without moving
    the other. The budget bounds the control state the whole index retains, 4 MiB by
    default; unbounded, it also reads every `.gitignore` whole.
    The line limit bounds one pattern, 16 KiB by default.
    They are `ControlLimits { budget, line_limit }` on `ScanConfig::control_limits` and
    `OpenOptions::control_limits`, `control_budget` and `control_line_limit` on Python
    `ScanOptions` and `OpenedOptions`, and `--gitignore-budget SIZE|all` and
    `--gitignore-line-limit SIZE|all` on the command line.
    `MAX_CONTROL_TABLE_BYTES` and `MAX_CONTROL_PATTERN_BYTES` are renamed
    `DEFAULT_CONTROL_BUDGET` and `DEFAULT_CONTROL_LINE_LIMIT`. Both limits are part of
    the snapshot scope, so changing either scans cold once.
    `Index::new_with_config` builds an index whose table enforces the limits its scope
    claims; saving or loading an index whose table and scope disagree is refused with
    `Error::ControlLimitsOutsideScope`.
  - Reports carry `ignore_rules` in every machine format: `null` when no `.gitignore`
    was read, otherwise `limits` (`budget` and `line_limit`, each bytes or `null`), the
    `applied` and `refused` counts, and `refusals`, each with a `path` and the `reason`
    naming the limit that fired.
    A note names the refused files’ directories and the flag for each limit that fired.
    The report schema moves to `fdu.report/5`, and to `fdu.report/6` with content
    analysis; Python `Status.ignore_rules` carries the same value.
  - The snapshot format moves to version 4, carrying both limits and every refusal.
    Scanning a root again writes its snapshot afresh, in place; a root that is never
    scanned again keeps a file this build does not read.
- **Breaking:** every surface reads `.gitignore` by default and reports how much of each
  size its rules ignore.
  - `fdu PATH` and `fdu --watch PATH` observe control state, as `prepare_report`,
    `fdu.report`, `open`, and `fdu.open` now all do: the one-shot planner no longer
    turns it off. `--no-gitignore` on the command line, and `read_controls` off in the
    library and Python, reads no rules.
  - Text summary, tree, and extension rows end with `(N ignored)` when they hold an
    ignored file, and the performance line counts the rule files read or says
    `no ignore rules`.
  - Machine summary, tree, and extension rows carry an `ignored` object and file rows an
    `ignored` flag, `null` when no rule was read.
    The fields join the `fdu.report/5` and `fdu.report/6` schemas this release already
    introduced. Rust `SummaryRow`, `TreeNode`, `TypeRow`, and `FileRow` gain `ignored`,
    `Report` gains `ignored_entries`, `Candidate` gains `ignored`, and
    `EntrySelection::admits` reads the bit from the candidate instead of a second
    argument; Python rows gain `ignored` and `IgnoredTally`.
  - `--exclude-ignored` and `--only-ignored`, `Selection::ignored` in Rust, and
    `Selection(ignored=IgnoredEntries...)` in Python report one side; sizes, sorting,
    and `--min-size` follow the entries shown.
    Over a scan that read no rules the selection is refused: a usage error on the
    command line, `InvalidArgumentError` in Python, and `Error::ControlStateNotObserved`
    from `prepare_report` and a watch session.
    `fdu_core::query::report` returns `Result<Report>` and refuses the same way, so no
    entry point answers an unanswerable selection with an empty report.
  - A `--watch` stream maintains the entry set its selection names.
    A `.gitignore` edit that moves an entry into `--exclude-ignored` or `--only-ignored`
    streams the upsert that draws it, and one that moves it out streams the removal,
    even though nothing about the file changed on disk.
    Every change record carries `ignored`, absent when the run read no rules, joining
    `fdu.stream/1`; Rust `Change` and Python `Change` gain the field.
  - An unfiltered `--view summary` that reads `.gitignore` retains the index to classify
    entries, so it uses more memory than the aggregate-only plan, which
    `--no-gitignore --view summary` still takes, and it saves a snapshot like any other
    report.
  - An unreadable `.gitignore` is an unreadable path: the result is partial and the
    command exits 2 unless `--allow-partial`.
  - Default snapshots now carry control state, so the first default run after upgrading
    scans cold once, and `fdu --no-gitignore` keeps a snapshot scope of its own.
- One projection of an opened-root read can refuse while the rest of the read answers.
  `ProjectionResult::Refused`, `RefusedResult` in Python, names why: a `Tree` or roll-up
  of a path that is not a directory, a page whose continuation record would exceed its
  bound, or a `Continue` for a continuation this root consumed or evicted.
  Breaking: `Error::NotADirectory` and `Error::ContinuationRecordLimit` are removed, a
  Python `Tree` of a non-directory returns a `RefusedResult` instead of raising
  `InvalidArgumentError`, and `Error::ContinuationUnavailable` fails a whole read only
  for a token from another root or one this root never issued.
- Every selection axis in an opened-root read matches the portable path a page row
  carries, including a report projection’s `Selection` globs, so a path taken from a
  page can be passed back as a filter.
  The native spelling of an escaped name, such as `100%.txt` for `100%25.txt`, matches
  nothing there; one-shot command-line globs keep native paths.
  Breaking: `EntrySelection` refuses terminal suffixes and ancestor names that could
  never match, in Rust and in Python, and a read carrying such a selection fails with
  `Error::InvalidValue`.
- The File Rollup registry reader accepts `schema_version` 4 as well as 3. An `icon` on
  a group or family must be a string and, like `hue`, stays out of the type-rule
  fingerprint, so a schema-3 registry and its schema-4 form share one fingerprint.
  An `icon` under schema 3, and any other schema version, is refused with an error that
  names the supported versions.

### Known limitations

- Content performance evidence is currently local M1/APFS data rather than a controlled
  cross-platform release matrix; CI checks semantics and benchmark contracts, not timing
  thresholds.
- Content sidecars are profile-scoped.
  Repeating one profile reuses unchanged files, but switching profiles can reread
  content whose lower-level analyzer results were already computed under another
  profile.
- Content coverage is also profile-scoped rather than per analyzer.
  An unsupported deeper analyzer leaves the file uncovered for that profile instead of
  retaining a separate lower-level metric record.
- Content analysis is one-shot; watch mode remains metadata-only.
- Standard LOC covers 15 common languages rather than SCC or Tokei’s long tail.
  Unsupported code stays explicit coverage, and embedded-language and AST metrics are
  deferred rather than approximated.
- The snapshot format is a bounded flat bootstrap format; compressed lazy blocks remain
  Phase 1 work.
- Entry records are not yet packed to the ~25–32 bytes per file memory target.
- Roll-up metrics are a fixed set rather than a reducer registry.
- Watcher queue bounds and permanent backend-failure marking remain explicit Phase 1
  hardening work.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
