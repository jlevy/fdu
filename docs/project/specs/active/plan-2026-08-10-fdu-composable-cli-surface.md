# Feature: Composable CLI and Query Surface

**Date:** 2026-08-10

**Author:** fdu project

**Status:** Draft

## Overview

Reshape the fdu command line and the library query layer around a small set of
orthogonal, reusable concepts instead of accumulating per-feature flags.
One invocation composes five axes — scan scope, selection, views, format, and mode —
over the same engine, so one command grammar covers what du, dust, dut, diskus, fd, and
find each do separately, plus a `tail -f`-style change feed and timestamp-watermark
queries reliable enough to drive backup and sync tooling.
The same concepts appear as typed values in the Rust API and the Python API, so the CLI
is a thin composition of the library rather than a second implementation.

This plan supersedes the flag-by-flag surface produced by the
[CLI UX plan](plan-2026-08-09-fdu-cli-ux-and-agent-skill.md) and gives the remaining
Phase 1 CLI beads (`fdu-oqoy`, `fdu-jej9`) a coherent target rather than a list of
incremental additions.

## Goals and Design Principles

These are the accountability criteria for every iteration of this design.
A change that violates one of these needs this spec amended first, not a silent
exception. The governing aspiration: the design fits the contours of the true problem —
no more complexity, but no less either.
Simple things are simple (`fdu` alone is a good answer), and complex things are possible
(any axis composes with any other), as with all good developer tools.
The concrete test for “no more complexity”: before adding a view or flag, show it cannot
be expressed as a composition of the existing axes — `largest` and `recent` views were
removed from this design by exactly that test.

1. **Five axes, no one-off flags.** Every option belongs to exactly one axis: scan
   scope, selection, view, format, or mode.
   A proposed flag that does not fit an axis is a design smell; either it generalizes
   into an axis value or it does not ship.
2. **Intuitive by default, everything by composition.** `fdu` or `fdu <path>` with no
   flags gives a useful, fast report with sensible, obvious defaults.
   There are no subcommands: the grammar is always “report on a path,” customized by
   composable flags, so a path argument can never be shadowed.
   Help documents each axis, its options, and its defaults plainly enough that the
   design is legible from `--help` alone.
3. **One scan, many views.** Views are projections over the in-memory index.
   Requesting more views in one invocation never adds filesystem work, and two reports
   over the same tree come from the same consistent index state.
4. **Views are readers.** The Delta contract stands: `scan` and `watch` produce
   observations, the index consumes them, and views only read the index.
   No view or format may add a mutation path or reach into producer state.
5. **Speed may be traded for certainty, never for honesty.** Every value carries its
   provenance — where it came from, when it was observed, whether it is final — so a
   consumer rendering a thousand rows knows which ones to trust, not merely that the run
   was partial (Goal 7 of the research;
   [the provenance model](plan-2026-08-11-fdu-progressive-results.md)). Provenance is a
   property of the value in the library; every surface displays it rather than inventing
   it. `auto` may choose the cheapest sound path per tree, and that choice is legible in
   the report’s `source` — a hidden choice would violate this principle, a visible one
   is how it is implemented.
   The explicit policies (`refresh`, `read-only`, `only`, `off`) remain for callers who
   want a particular path rather than the cheapest one.
6. **Benefit from the OS; never depend on it.** Three tiers, each fast in its own right:
   a portable walk, a cache for the cases a cache can help, and platform APIs (bulk
   stat, `statx`, `io_uring`, fanotify, the FSEvents journal) probed at runtime as
   accelerators for the first two.
   A missing or degraded tier falls back to the one below, losing speed and never
   accuracy. The most valuable platform APIs do not compute answers; they **bound
   uncertainty** — naming in milliseconds which parts of a million-entry tree could have
   changed, so verification goes only there.
   That is what makes near-real-time views of huge trees possible, and why such an API
   may narrow what is checked but never substitute for checking it.
7. **Same concepts at every level; the CLI invents nothing.** `Query`, `View`, `Report`,
   and `CachePolicy` are typed values in the Rust library; the CLI parses flags into
   them and renders `Report`s; Python exposes the same types.
   Any logic beyond flag parsing, terminal handling, and exit codes belongs in the
   library, where Rust and Python callers get it too.
   A capability that exists in one surface and not the others is unfinished, and
   complexity that exists only at the CLI layer is misplaced.
8. **Subsume the neighbors.** The compositions below must each be expressible in one
   invocation; the mapping table in the Design section is a living checklist.
   Where a neighbor’s behavior is out of scope (e.g. ncdu’s TUI), that is recorded as a
   non-goal, not left ambiguous.
9. **Formats are serializations, not features.** Every view renders in every format
   (`text`, `json`, `jsonl`, `yaml`). Machine formats are schema-versioned; a schema
   change without a version bump fails a golden test.
10. **Watch is the same query, repeated.** A watch run evaluates the same selection and
    views as a one-shot run, re-applied as deltas arrive.
    There is no separate watch grammar to learn, and overflow or invalidation is
    reported explicitly in the stream, never dropped.
11. **Utilities are explicit flags, never side effects.** Cache inspection and clearing
    are lifecycle flags on the same grammar (`--cache-status`, `--cache-clear`) that run
    without scanning and without a report; a report run never deletes anything, and
    utility output composes with the format axis like any other.
12. **No unbenchmarked performance claims.** Each new output surface becomes a named
    benchmark job per the
    [performance evidence research](../../research/research-2026-08-09-end-to-end-performance-evidence.md),
    and flags are part of benchmark identity: renaming one means updating the job
    manifests in the same change.

These principles graduate into a standalone design doc under `docs/project/guides/` once
implementation has validated them (Phase 4), so they outlive this spec.

## Non-Goals

- An interactive TUI (ncdu/gdu style).
  The composable one-shot and streaming surfaces come first; a TUI would be a consumer
  of the same `Query`/`Report` layer later.
- Content-tier metrics (word counts, code metrics) and the type-rule dialect; those
  remain owned by `fdu-v4lc` and the post-Phase 1 roadmap.
  The `types` view here rolls up by derived extension exactly as the index already does.
  This plan does fix the *shape* of the derived-data cache layer (see Cache Policy) so
  content metrics can arrive without a format break, but ships no analyzer.
- Regex name matching.
  Selection uses globs; fd-style regex can layer on later without changing the axis.
- Multiple root paths in one invocation.
  One root, one index, one cache remains the contract; see Open Questions.
- Configuration files, prompts, pagers, and progress animation (unchanged from the CLI
  UX plan).
- Publishing or performance claims; release gates are owned by `fdu-9cf0` and the
  performance plan.

## Background

The current CLI (see the CLI UX plan) is one flat command with two views and two formats
expressed as mutually exclusive booleans (`--by-type`, `--json`), hardcoded
size-descending sort, no filtering, and rendering welded to the flag struct as private
methods. Watch exists only as a library feature with zero CLI surface.
There is no cache status, list, path, or clear operation anywhere — cache files are
opaque path-hash names under the user cache directory, with no way to map a file back to
its root. `ScanConfig` already models scope correctly (`max_depth`, `one_filesystem`,
`follow_symlinks`, `threads`) and already excludes non-semantic knobs from the cache
key, but the CLI exposes only `--max-depth`.

flowmark-rs (checked out under `attic/flowmark-rs`) is the closest first-party prior art
for cache UX and is worth borrowing from deliberately:

- `--show-cache` prints a six-line status covering both the aggregate cache root and the
  current project’s manifest; `--clear-cache` is idempotent, non-interactive, and always
  echoes the cache directory before acting.
- Cache lifecycle operations run before file-argument validation, so they need no other
  arguments, and clearing works even when the cwd is unresolvable.
- Cache-root resolution is a pure, testable function returning a path plus a named
  source tier (OS cache dir, home fallback, temp fallback), with the CLI emitting a
  distinct warning per fallback tier.
- Comma-delimited list values are split, trimmed, checked per token, and rejected with
  the valid values named inline.

fdu’s snapshot layer is stronger than flowmark’s manifest (binary format, CRC, engine
fingerprint, atomic replace, fail-closed corrupt handling), so this plan adopts
flowmark’s *surface* patterns, not its storage.

## Design

### Concept Model: Five Axes

| Axis | Question it answers | Options | Engine binding |
| --- | --- | --- | --- |
| **Scope** | What does the engine observe and retain? | `PATH`, `--scan-depth <N>`, `--one-filesystem` | `ScanConfig` → `ScanScope`, the cache identity |
| **Selection** | Which retained entries does this query consider, and how are results shaped? | `--include <GLOB>`, `--exclude <GLOB>`, `--min-size <SIZE>`, `--modified-since <WHEN>`, `--modified-before <WHEN>`, `--kind <LIST>` of `file\|dir\|symlink`, `--depth <N>`, `-n/--limit <N>`, `--sort <size\|count\|mtime\|name>`, `--reverse`, `--size <allocated\|apparent>` | View-time filter over the index; never part of the cache key |
| **View** | Which roll-ups or listings are reported? | `--view <LIST>` — comma-delimited from `tree`, `types`, `files`, `summary`; default `tree` | Pure projections over `Index` |
| **Format** | How is the report serialized? | `--format <text\|json\|jsonl\|yaml>`, `--color <auto\|always\|never>` | Serializers over `Report`; schema-versioned |
| **Mode** | One answer or a live feed, and how is the cache used? | one-shot (default) vs `--watch [--interval <DUR>]`; `--cache <auto\|refresh\|only\|off>`; `--allow-partial` | `open()` path selection; `watch::Watcher` driving the same query |

Scope versus selection is the load-bearing distinction, and it is why filters are cheap:
scope determines what is scanned and cached (one cache serves every query), while
selection is evaluated at view time against the retained index.
This is the same reasoning as the research’s gitignore *tag, don’t prune* decision.
The rename from `--max-depth` to `--scan-depth` (alongside render `--depth`) makes the
distinction legible; the benchmark job manifests that reference the old spelling are
updated in the same change (Principle 12).

### CLI Surface

```text
fdu [PATH] [OPTIONS]                              # the one and only command
fdu [PATH] --cache-status | --cache-clear[=all]   # lifecycle flags; never scan
fdu --skill / --help / --version                  # unchanged discovery surfaces
```

Representative compositions, which double as the subsumption checklist (Principle 8):

| Instead of | Run | Notes |
| --- | --- | --- |
| `dust` / `dut` | `fdu` | tree view, warm when a cache exists |
| `du -sh` / `diskus` | `fdu --view summary` | one-line totals |
| `du -a --max-depth 3` | `fdu --depth 3 -n all` | unlimited entries per directory |
| `fd -e rs` / `find -name` | `fdu --view files --include '*.rs'` | flat listing, one path per line in text |
| biggest files (`dust -f`, `find -size +10M`) | `fdu --view files --min-size 10M --sort size -n 100` | not a special view — files + sort + limit |
| recently modified (`find -mmin -60`) | `fdu --view files --modified-since 1h --sort mtime` | humane durations, not day counts |
| `du` by type | `fdu --view types` | current `--by-type` |
| two reports, one scan | `fdu --view types,tree` | one index, both roll-ups |
| `tail -f` for a tree | `fdu --watch --view files --format jsonl` | one record per applied change |
| live dashboard poll | `fdu --watch --view tree,types --interval 2s` | aggregate views re-render when dirty |
| forced cold measurement | `fdu --cache refresh` | ignores and rewrites the snapshot |
| answer from cache alone | `fdu --cache only` | never touches the tree; labeled stale |
| incremental backup sync | `fdu --view summary`, then later `fdu --view files --modified-since <scan_started_at>` | exact-timestamp watermark; see below |

Grammar conventions, applied uniformly so every flag behaves the way its neighbors do:

- **Closed identifier vocabularies are comma-delimited lists** (`--view`, `--kind`):
  split on commas, trim, reject empty tokens, and name the valid values in the error for
  an unknown token (flowmark’s list pattern).
  Views render in the order given; duplicates are an error.
- **Open pattern values are repeatable flags** (`--include`, `--exclude`), never comma
  lists, because glob brace syntax (`*.{rs,toml}`) contains commas.
- **Humane value grammars, shared with the API**: sizes accept `10M`/`1.5GiB`; times
  accept ages (`45s`, `2h`, `1h30m`), absolute timestamps, `@epoch`, or `now`, per the
  single `WHEN` grammar defined under Timestamps and Sync Watermarks.
  Bounded values accept `all` for unbounded (`-n all`, `--depth all`); `--depth 0` keeps
  du’s meaning of “root totals only.”
- **Lifecycle flags take optional values** (`--cache-status[=root|all]`), defaulting to
  the current root.

Further notes:
- Replaced flags from the current pre-release surface: `--by-type` → `--view types`,
  `--json` → `--format json`, `--apparent-size` → `--size apparent`, `--number` →
  `-n/--limit`, `--max-depth` → `--scan-depth`, `--no-cache` → `--cache off`. No aliases
  are retained; the interface is explicitly pre-release and the SKILL.md, `AFTER_HELP`,
  README, and benchmark manifests are updated in the same change.
- Cache lifecycle flags follow flowmark’s optional-value pattern:
  `--cache-status[=root|all]` and `--cache-clear[=root|all]`, both defaulting to `root`
  (the resolved `PATH`). They run before scan validation, so they need no readable tree,
  suppress the report, and may be combined in one invocation (clear runs first, then
  status).
- Exit codes are unchanged: 0 complete (or partial with `--allow-partial`, or broken
  pipe), 1 fatal, 2 partial or usage error.
  `--cache only` with no usable snapshot is fatal (exit 1) — there is no data to answer
  with, and guessing would violate Principle 5.

### Views

Each view is a pure function of the index and a `Selection`, returning one section of
the `Report`. There are exactly four, one per shape of answer, and each is a different
grouping of the same underlying entries:

| View | Grouping | Content | Default sort | Subsumes |
| --- | --- | --- | --- | --- |
| `tree` | by directory hierarchy | roll-ups per directory (files, dirs, bytes, allocated, newest mtime), bounded by `--depth`/`--limit` | size desc | du, dust, dut |
| `types` | by derived extension | flat roll-up per type | size desc | current `--by-type` |
| `files` | none (individual entries) | flat listing of matching entries | name asc | fd, find |
| `summary` | everything in one group | one aggregate row: files, dirs, bytes, allocated, newest mtime | — | du -s, diskus |

“Largest files” and “recently modified” are deliberately not views: they are
`files --sort size` and `files --sort mtime --modified-since …`, because every selection
and shaping knob applies to every view.
The same composability makes `tree --sort mtime` an activity map of a project with no
extra machinery. When the reducer registry (Goal 6) and type rules (`fdu-v4lc`) land,
they extend this table’s *columns and groupings* — new metrics per row, content-aware
`types` — rather than adding parallel flags, which is the extension contract that keeps
the axis count fixed.

In text format, `files` prints one path per line and nothing else, so it pipes straight
into xargs and diff-friendly file lists; per-entry fields (size, mtime, kind) are
carried in the machine formats.
Two performance tiers, stated so nobody is surprised later: an unfiltered `tree`,
`types`, or `summary` view reads the pre-computed `RollUp` state directly; any selection
filter (and the `files` view) traverses the retained index in memory.
Both are milliseconds warm; neither touches the filesystem.
`ExtTally` gains an `allocated` field so the `types` view honors `--size allocated`
instead of silently switching metrics.

### Timestamps and Sync Watermarks

Every report carries two instants in every format: `scan_started_at` (when the walk or
revalidation began) and `generated_at` (when the report was rendered), as RFC 3339 UTC
with nanosecond precision; the text header prints the same “as of” instant.
Reports are therefore self-describing evidence: a summary is not just “the disk usage”
but “the disk usage as observed starting at T.”

Time selection uses one shared, fully specified grammar (`WHEN`), covering ages and
absolute times:

```text
WHEN      = "now" | AGE | TIMESTAMP
AGE       = 1*( INT UNIT )                ; that long before now: 45s, 2h, 7d, 1h30m
UNIT      = "s"|"sec"|"secs"|"second"|"seconds"
          | "m"|"min"|"mins"|"minute"|"minutes"
          | "h"|"hr"|"hrs"|"hour"|"hours"
          | "d"|"day"|"days" | "w"|"week"|"weeks"
TIMESTAMP = RFC3339                       ; 2026-08-10T18:22:31.482919114Z — exact
          | date [ " " time ]            ; 2026-08-10 [12:30[:45]] — local time
          | "@" INT [ "." FRAC ]         ; seconds since the Unix epoch, UTC
```

The rules that keep it well defined:

- The surface grammar is deliberately fd’s `--changed-within`/`--changed-before` grammar
  (durations, RFC 3339, local date/datetime, `@epoch`) — the de facto modern standard
  from a tool this design subsumes — extended with compound ages (`1h30m`) and the `now`
  keyword, so existing muscle memory transfers.
- We borrow the grammar, not the implementation: `humantime`, the crate behind fd’s
  parser, is unmaintained (RUSTSEC-2025-0014, and fd has an open issue to replace it),
  so `parse_when` is a small first-party parser with table-driven tests.
  If scope ever outgrows it, the vetted fallback is `jiff`, not a revival of
  `humantime`; either way the *grammar above* is fdu’s contract, independent of the
  parser behind it.
- Ages subtract exactly from one `now` captured per invocation.
  Calendar units (months, years) are rejected with an error suggesting `30d`/`365d`:
  they require calendar arithmetic that approximates (humantime counts a month as 30.44
  days), and a grammar for file ages must not approximate.
  Fractions are rejected the same way (`1.5h` → “use 1h30m”); `@epoch` is the one place
  a fraction is meaningful and allowed, because that is its RFC precedent.
- Natural-language forms (`yesterday`, `2 weeks ago` à la GNU date and journalctl) are
  rejected: locale-dependent and unbounded; the closed grammar above is the whole
  language.
- RFC 3339 timestamps carry their own offset and round-trip a report’s `scan_started_at`
  exactly; the date/datetime shorthand is local time, matching what fd users expect at a
  prompt.

`--modified-since` is inclusive and `--modified-before` is exclusive, forming the
half-open window `[since, before)`.

Those two pieces compose into a reliable incremental-sync watermark with no new
machinery:

```text
fdu /data --view summary --format json   # record scan_started_at, back everything up
# ... later ...
fdu /data --view files --format jsonl \
    --modified-since 2026-08-10T18:22:31.482919114Z   # exactly what needs re-sync
```

The rules that make this trustworthy, stated so they hold under iteration:

- The watermark is the *previous scan’s start*, not its end: a file modified mid-scan
  may have been observed before the modification, and only the start bound is
  conservative.
- The boundary is inclusive, so a file whose mtime equals the watermark re-lists.
  For sync, duplicates are safe and omissions are not.
- The follow-up query answers from a revalidated index under the default `--cache auto`;
  under `--cache only` it answers from the snapshot alone and says so (Principle 5).
- mtime-window queries trust mtimes, exactly as find and rsync do; a backdated mtime is
  invisible to all of them.
  The live exact feed is `--watch` (clocked deltas, no mtime trust); a durable
  cross-restart journal that upgrades one-shot queries to exact deltas is future work
  owned by `fdu-3dtq`, and this schema leaves room for it rather than pre-building it.

### Cache Policy and Utilities

`--cache <POLICY>` replaces the single `--no-cache` boolean:

| Policy | Reads snapshot | Touches filesystem | Writes snapshot | Use |
| --- | --- | --- | --- | --- |
| `auto` (default) | when it is cheaper | the cheapest sound verification | on complete | fastest trustworthy answer |
| `refresh` | no | full scan | on complete | forced cold start; benchmark control |
| `read-only` | yes | revalidates | never | warm answer without touching the cache |
| `only` | yes | never | no | instant answer from data on hand, labeled `freshness: stale` |
| `off` | no | full scan | no | leave no trace |

Disabling the always-write behavior is therefore a policy value, not a separate flag:
`read-only` keeps the warm read path and suppresses the write, `off` suppresses both.

Every report carries `source` (`cold_scan`, `warm_revalidate`, `cache_only`),
`freshness`, and `complete`, in all formats, so no policy can silently lie.

**`auto` is a cost decision, not a habit.** Measurement settled this: on a 60k-entry
tree a parallel rescan costs 37 ms while loading the snapshot and verifying it costs 102
ms, so reading the cache is a *loss* at project scale — and for stat-tier queries the
full sweep is dominated by rescanning at every size, because it performs the same
enumeration and the same one-stat-per-entry and then adds a load.
At home-folder scale the reverse holds: the tree cannot fit the OS metadata cache
(`kern.maxvnodes` is ~263k on a 32 GiB Mac), every scan is effectively cold, and the
snapshot plus a journal resume is the only affordable answer.

So `auto` estimates before it acts, from the snapshot header alone: entry count and the
µs/entry that tree’s own last scan achieved, against the platform’s metadata-cache
capacity and the reducer tier the requested views need.
Small tree, stat-tier query: rescan and refresh the snapshot.
Large tree with a usable journal: load, replay, verify only what changed.
Content-tier query at any size: load and sweep, because the sweep’s stats are what avoid
re-reading unchanged files.
The decision function, its self-calibrating cost model, and the derived replay budget
are specified in the
[FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
(bead `fdu-6ld9`); `refresh`, `read-only`, `only`, and `off` remain explicit overrides
for anyone who wants a specific path rather than the cheapest one.

**When the cache is written.** The policy axis decides *whether* a run may write; these
rules decide *what and when*, and they are rules, not heuristics (Principle 5):

- The core snapshot is written by `auto` and `refresh` only when the scan is complete
  and the index is `Fresh` (the existing invariant), and **on a background thread
  overlapped with rendering** — once producers finish, the index is read-only, so
  serialization and rendering are two concurrent readers.
  The save never delays first output; the process joins the save thread before exit so a
  write is never abandoned, and the save still completes when rendering ends early
  (broken pipe must not discard a finished scan’s work).
  A failed save (read-only cache dir, quota) is a stderr warning, never a changed exit
  code.
- It is written on every platform, for every tier of query, whenever complete —
  including pure stat roll-ups.
  The tempting refinement “stat-only runs skip the write” is rejected on the
  [performance frontier research](../../research/research-2026-08-10-performance-frontier.md):
  the write is tens of milliseconds off the hot path, while the stat-tier snapshot is
  exactly what the two decisive warm paths consume — on cloud runners the snapshot is
  the *only* possible warm state (the OS metadata cache does not fit in RAM), and on
  macOS the planned FSEvents journal resume (O(changes) warm opens) anchors on
  snapshot-carried resume tokens.
  What makes the cache feel unnecessary on a warm laptop today is the measured
  warm-costs-more-than-cold defect (the loop’s H9 family), which is owned by the
  performance loop, not by write policy.
- The snapshot format reserves the journal-resume fields (event ID, volume UUID,
  platform tag) now, per that research, so the macOS rung can land without a format
  break.

**Two cache layers.** Content-derived metrics (line counts and future analyzers — where
the user-visible payback is largest, minutes cold to seconds warm) live in a
**derived-data layer, not the core snapshot**: separate per-analyzer files keyed by
`(fingerprint, analyzer id, analyzer version)`, written whenever an analyzer ran under a
writing policy, loaded lazily, invalidated per analyzer without touching tree truth,
size-bounded, and purgeable via `--cache-clear`. The core snapshot stays small and fast
to open; runs that request richer roll-ups enrich the derived layer additively.
No analyzer ships under this plan, but the layer’s shape is fixed here so the content
tier arrives without a format break.

**Verification cost follows the query.** Under `auto`, “revalidates” means the cheapest
*sound* verification for the reducers the requested views actually use, per the frontier
research’s tier rule: name-tier questions (counts, tree shape) verify with one stat per
directory; any stat-tier metric (sizes, mtimes — every current default view) requires
one stat per entry, because in-place edits are invisible to directory fingerprints;
content-tier questions add re-reads of changed files only.
This is exact and needs no staleness label — view selection changes verification cost by
integer factors while staying trustworthy.
Reducers declare their tier when the reducer registry (`fdu-a6dz`) lands; until then all
shipped views are stat-tier and verification is the N-stat sweep.
A future *labeled* stale-sizes mode (the research’s H44) is possible but never a default
and not part of this plan.

The lifecycle flags are backed by new library functions rather than CLI-side directory
walking (Principle 7 — the CLI invents nothing):

- `cache_status(root) -> CacheStatus` — snapshot path, presence, size, entry count,
  scope, saved-at time, and whether the engine fingerprint still matches.
- `list_caches() -> Vec<CacheStatus>` — enumerates the cache directory and reads each
  snapshot’s bounded header to recover the root path, fixing today’s “opaque hash files
  with no reverse mapping” problem.
  This backs `--cache-status=all`; unrecognized files are listed as unrecognized.
- `clear_cache(root)` / `clear_all_caches()` — idempotent; `--cache-clear` echoes the
  cache directory before acting and reports `Cache cleared.` or `Cache already empty.`
  (flowmark’s pattern), with no prompt and no `--force`. `--cache-clear=all` removes
  only files that parse as fdu snapshot headers, never unrecognized files.

`--cache-status` renders through the same format axis (`--format json` works), so agents
get cache observability without a second schema style.

### Watch Mode

Change detection is event-driven, never polling: the watcher binds the native OS backend
through `notify` (FSEvents on macOS, inotify on Linux, ReadDirectoryChangesW on
Windows), coalesces kernel-pushed events, and verifies each coalesced path with one
fresh stat — idle cost is zero filesystem work.
`--interval` below throttles only how often *aggregate views re-render*; it plays no
part in detection. Polling exists solely as the deliberate fallback for filesystems
without native events (NFS/FUSE/SMB), selected per-filesystem by the watch-hardening
work (`fdu-lka2`).

`--watch` runs the same query continuously (Principle 10):

1. Open the index per the cache policy and emit the initial report exactly as a one-shot
   run would.
2. Drive `watch::Watcher::apply_next` on the consuming thread.
   Each applied batch is filtered through the same `Selection`.
3. The `files` view streams per batch: one line (text) or one record (jsonl) per
   effective applied op — path, op (`upsert`/`remove`), kind, size, mtime, and the index
   clock. This is the `tail -f` surface, and the same selection window applies:
   `--modified-since 1h --watch` bounds the initial report, then streams everything
   after it, and `--modified-since now --watch` is a pure tail with an empty initial
   listing — no dedicated suppress-initial flag needed.
4. Aggregate views (`tree`, `types`, `summary`) re-render at most once per `--interval`
   (default 2s), only when dirty, separated in text by a timestamped header.
5. Overflow or subtree invalidation from the watcher appears as an explicit `invalidate`
   record with its reason, followed by the post-reconciliation report; it is never
   dropped (Principle 5).
6. SIGINT/SIGTERM exit 0 after a final snapshot save when the index is `Fresh` and
   policy allows writes; watch errors exit 1.

Streaming machine output uses a new `fdu.stream/1` JSONL schema with tagged record types
(`report`, `change`, `invalidate`, `status`); one-shot machine output uses
`fdu.report/1` (below).
Constraint carried from the engine: watch requires full scope, so `--watch` with
`--scan-depth` or `--one-filesystem` is a usage error (exit 2) until
`validate_for_watch_scope` learns otherwise.

### Rust API

The new `query` module (feature-independent, usable by CLI, Python, and external
consumers alike):

```rust
pub struct Query {
    pub selection: Selection,      // include/exclude globs, min_size, kinds,
                                   // modified window (since/before), depth, limit,
                                   // sort, size metric
    pub views: Vec<ViewSpec>,      // Tree, Types, Files, Summary
}

pub fn report(index: &Index, query: &Query) -> Report;   // pure; never scans

pub enum CachePolicy { Auto, Refresh, ReadOnly, Only, Off }   // consumed by open()

pub struct Report { /* scan_started_at, generated_at, source, freshness,
                       complete, scope, one section per view */ }

pub fn parse_when(s: &str, now: SystemTime) -> Result<SystemTime>;  // "2h" | RFC 3339
pub fn parse_size(s: &str) -> Result<u64>;                          // "10M" | "1.5GiB"
```

The value grammars (`parse_when`, `parse_size`) are small first-party parsers in the
library, not CLI helpers and not new dependencies, so the CLI, Rust callers, and Python
all accept identical strings (Principle 7); `parse_when` takes `now` as an argument so
callers and tests control the reference instant.

`Report` and its sections derive `serde::Serialize`; `text` rendering and the
JSON/JSONL/YAML serializers live in the CLI feature.
`cli.rs` shrinks to parsing flags into `(ScanConfig, CachePolicy, Query, Format)` and
routing streams — the current private rendering methods on `Cli` move behind
`query`/`format` types with their own unit tests.
Watch composes the same pieces: a `Session` owning `IndexHandle` + `Watcher` yields
batches already filtered through the `Selection`, and the CLI loop is a thin consumer.

The parity test for Principle 7 is mechanical: the CLI’s five axes map one-to-one onto
these library types, so any capability reachable by flags is reachable as one typed
call, with the same defaults.
If implementing a flag ever requires logic that does not fit `Query`, `CachePolicy`, or
a `Report` serializer, the library types are wrong and get fixed first; each phase ends
with an explicit review of what, if anything, lives only in `cli.rs`.

Supply-chain note: `serde` (derive) enters the core crate; YAML requires care because
`serde_yaml` is unmaintained.
The YAML emitter is either a small first-party writer over the already-structured
`Report` or a vetted maintained crate passing the cool-off policy — decided in
implementation, recorded in `deny.toml` either way.

### Python API

Mirror, not wrapper-of-CLI:

```python
idx, report = fdu_py.open(root, cache="auto", scan_depth=None)
r = idx.report(views=["types", "tree"], include=["*.rs"], min_size="10M")
changed = idx.report(views=["files"], modified_since="2h")      # or a datetime
resync = idx.report(views=["files"], modified_since=r.scan_started_at)
for batch in idx.watch(views=["files"], interval=2.0):   # iterator of batches
    ...
```

String values accept exactly the CLI grammars (`"2h"`, `"10M"`); native types
(`datetime`, `int`) are accepted wherever a string is.

`open`, `scan`, and `Index` keep their existing contracts; `report` and `watch` are
additive.
The wheel’s console `fdu` automatically gains the whole CLI surface through the
shared process boundary, as today.

### Schemas and Compatibility

- `fdu.report/1` supersedes `fdu.tree/2`: top level carries `schema`, `generator`,
  `root`/`root_raw`, `scan_started_at`, `generated_at`, `source`, `cache`, `complete`,
  `freshness`, `scope`, `selection`, and `reports` (one entry per requested view, in
  request order). This plan explicitly authorizes the schema replacement the CLI UX plan
  forbade; the golden fixture and schema-bump test move with it.
- The interface remains pre-release; no aliases for replaced flags.
- Library compatibility: existing `Index`, `scan`, `snapshot`, and `watch` contracts are
  preserved; `query` is additive, `ExtTally` gains a field (semver-minor while
  unpublished), and this plan makes no snapshot format changes — the v2 → v3 cursor
  section is owned by the
  [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
- Benchmark identity: `cli-human` and `cli-json` job definitions are re-pointed at the
  new argument vectors in the same change, and `cli-summary`, `cli-files`, and
  `watch-stream` become named jobs when their surfaces land (Principle 12).
- The help text, SKILL.md, and README currently drift as three hand-maintained copies of
  the contract; each phase updates all three, and consolidating them into one generated
  source is tracked as follow-up work, not assumed.

## Implementation Plan

### Phase 1: Query and Report Core

- [x] Add `query` module: `Selection`, `ViewSpec` (tree/types/files/summary), `Query`,
  `Report`, pure `report()` with unit tests per view × selection, including the roll-up
  fast path vs traversal tier
- [x] Implement the shared value grammars (`parse_when`, `parse_size`) and the
  `--modified-since`/`--modified-before` half-open window; stamp `scan_started_at` and
  `generated_at` on every report. Local date-times are rejected pending a time-zone
  decision (`fdu-f6dn`); the watermark round-trip test is still owed (`fdu-3vgt`)
- [x] Add `allocated` to `ExtTally` and thread the size metric through all views
- [x] Serde-derive `Report`; implement `text`, `json`, `jsonl`, `yaml` formatters and
  the `fdu.report/1` golden + schema-bump tests; resolve the YAML dependency per the
  supply-chain note. `fdu.stream/1` has no equivalent schema-bump test yet (`fdu-rti1`)
- [x] Rework CLI parsing to the five axes (view list parsing, replaced flags, exit
  contract), update SKILL.md, `AFTER_HELP`, README, tryscript goldens, and the benchmark
  job manifests together
- [x] Python `Index.report(...)` with the same defaults and names

### Phase 2: Cache Policy and Utilities

- [x] `CachePolicy` in `open()` covering auto/refresh/only/off, with `only` failing
  closed when no usable snapshot exists
- [x] Snapshot write ordering and failure semantics: save on a background thread
  overlapped with rendering, only when complete and `Fresh`, joined before exit,
  completing even on broken-pipe rendering; a failed save warns on stderr without
  changing the exit code; `read-only` policy suppresses the write entirely.
  The journal-resume fields (event ID, volume UUID, platform tag) are reserved by the
  [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
  as snapshot format v3 (bead `fdu-2cdv`), not duplicated here
- [x] Document the two-layer cache design and the tier-derived verification contract in
  help, SKILL.md, and the schema docs (implementation of tiered verification lands with
  the reducer registry, cross-plan)
- [x] Library `cache_status`, `list_caches`, `clear_cache`, `clear_all_caches` with
  bounded header reads and never-delete-unrecognized semantics
- [x] `--cache-status[=root|all]` and `--cache-clear[=root|all]` lifecycle flags
  rendering through the format axis, running before scan validation; tryscript coverage
  per flowmark’s cache-behavior suite
- [x] Python `cache` accessors mirroring the library functions

### Phase 3: Watch Mode

- [x] `Session` API composing `IndexHandle`, `Watcher`, and `Query`; batch filtering
  through `Selection`
- [x] `--watch`/`--interval` CLI loop: initial report, streamed `files` records,
  dirty-gated aggregate re-render, explicit invalidation records, `fdu.stream/1` schema,
  and a persisting save. Delivered as a save after each dirty batch rather than a signal
  handler: std has no portable one, and a watch session ends by signal far more often
  than it ends politely, so an exit-time save would be the one that never runs.
  Pinned by `crates/fdu/tests/watch_persistence.rs`, which SIGKILLs the real binary
- [ ] **Not delivered:** goldens for the streamed records — needs a bounded capture
  command first (`fdu-t9nv`, see [Remaining work](#remaining-work))
- [x] Scope validation errors for `--watch` + `--scan-depth`/`--one-filesystem`
- [x] Python `Index.watch(...)` iterator with deterministic shutdown tests
- [x] `watch-stream` benchmark job registration — the job vocabulary only, which is
  what this item asked for; the runner is `fdu-g8ks`

### Phase 4: Design Principles Documentation

- [x] Distill the Goals and Design Principles of this spec — as actually implemented,
  with any amendments iteration forced — into a durable design doc at
  `docs/project/guides/fdu-design-principles.md`, following common-doc-guidelines: the
  five axes, the delta contract, cache honesty, the CLI-invents-nothing parity rule, and
  the subsumption checklist
- [x] Run the end-of-plan parity review (what, if anything, lives only in `cli.rs`) and
  record its outcome in the design doc
- [x] Point AGENTS.md, README, and the architecture references at the design doc; move
  this spec to done and reconcile the subsumed beads (Open Question 4)

## Testing Strategy

- Unit tests per view over a fixed synthetic index, crossed with selection filters, both
  size metrics, and both performance tiers; property test that adding a view never
  changes another view’s section.
- Golden tryscript sessions per axis: view lists, each format, cache policies (using a
  scratch `XDG_CACHE_HOME`), cache utilities, and watch streaming with injected changes;
  goldens are byte-stable (integer formatting, no floats in text output).
  Everything here except watch streaming is delivered (68 blocks).
  Watch streaming needs a bounded capture command before it can be goldened at all — see
  [Remaining work](#remaining-work).
- Schema tests: `fdu.report/1` and `fdu.stream/1` fixtures that fail on unversioned
  change.
- Time-window tests: table-driven `parse_when`/`parse_size` grammar units with injected
  `now`, covering every accepted form (`now`, compound ages, RFC 3339, local
  date/datetime under a pinned `TZ`, `@epoch` with fraction) and every rejection with
  its suggestion (months/years → days, fractional ages → compounds, natural language);
  boundary inclusivity at exact-equal mtimes; a watermark round-trip proving a report’s
  `scan_started_at` fed back as `--modified-since` lists exactly the files touched after
  scan start, including one touched mid-scan; timestamp fields are normalized in
  goldens.
- The existing partial-result, non-UTF-8 identity, broken-pipe, and stack-depth process
  tests are retargeted, not weakened; deep-tree rendering stays iterative.
- `make check` remains the handoff gate, including `--no-default-features` (the `query`
  module must build without the CLI feature).

## Rollout Plan

One PR per phase, each leaving the CLI fully working and documented.
Phase 1 is the breaking rename PR and lands before any new capability so churn on
SKILL.md, goldens, and benchmark manifests happens once.
Phase 4 is small but not optional: the principles must land in `docs/project/guides/` so
they govern future work, not just this plan.
No publishing; `fdu-9cf0` gates remain.

## Remaining work

The four phases are implemented and every leaf bead is closed.
What an end-to-end audit on 2026-08-11 found still outstanding, each mapped to a bead so
it cannot be lost by being described only here:

| Gap | Bead |
| --- | --- |
| Watch-streaming goldens with injected changes | `fdu-t9nv` |
| `fdu.stream/1` schema-bump test (only `fdu.report/1` is pinned) | `fdu-rti1` |
| Watermark round trip: `scan_started_at` → `--modified-since`, incl. a mid-scan write | `fdu-3vgt` |
| `watch-stream` benchmark **runner** (only the job vocabulary is registered) | `fdu-g8ks` |
| Local date-times in `parse_when`, pending a time-zone decision | `fdu-f6dn` |
| Cache retention: nothing prunes snapshots or bounds total size (open question 5) | `fdu-558j` |
| Open questions 1, 2, and 4 | `fdu-khu8` |
| Automate the runbook's bead-sync check as a periodic guard | `fdu-qut8` |
| Direct unit tests for the watch persistence state machine | `fdu-w8af` |

`fdu-w8af` is worth its own line rather than being folded into general test debt.
Two of the three defects review found on this branch were in the watch loop's save
throttle and pending flag, and the second was a regression introduced by the fix for the
first. Every test drives that logic end to end through the spawned binary, which can
observe only whether a file changed on disk — it cannot enumerate the transitions. The
logic is decisions, not I/O, and belongs under a table test.

The watch goldens are the one entry whose absence is a *shape* problem rather than
unwritten work. tryscript compares one command's completed output, and a watch process
never exits, so there is nothing to compare until watching can be expressed as a command
that terminates. The design recorded on `fdu-t9nv` is a Node helper — tryscript already
requires Node, so this adds no dependency — that spawns `fdu --watch`, applies a scripted
sequence of changes, collects the `fdu.stream/1` records, terminates the child, and
prints the normalized stream. Golden discipline then applies unchanged.

Until it lands, the watch surface is covered by `crates/fdu/tests/watch_session.rs` for
event semantics and selection filtering, `crates/fdu/tests/watch_persistence.rs` for
save-surviving-SIGKILL, goldens for the bounded parts (the `--help` contract, the
scope-validation rejections), and section 6 of
[the integration runbook](../../guides/integration-runbook.md) for the one property no
automated test asserts well: that an idle tree costs 0% CPU.

## Open Questions

1. Short flag for `--view` (`-v` collides with the verbose convention; no short flag is
   proposed initially).
2. Multiple roots per invocation (fd/find allow several): one index per root is easy to
   compose in the library; the CLI ergonomics and cache story are not designed here.
3. Disposition of `fdu-oqoy` (adaptive terminal width, gitignore display) and `fdu-jej9`
   (JSONL, schema docs): this plan subsumes their JSONL/sorting/summary scope; the
   remainder (adaptive width, gitignore tagging) likely re-homes under this epic — needs
   maintainer sign-off before closing or re-parenting either bead.
4. Whether a general `--group-by` ever surfaces once the reducer registry lands
   (generalizing `types`), or named views remain the entire vocabulary and new groupings
   arrive only as new views.
5. Cache retention: nothing yet prunes snapshots for roots that are never queried again
   or bounds the derived-data layer’s total size.
   Age-based GC, size caps, or manual-only (`--cache-clear`) needs a decision before the
   derived layer ships.

Resolved by composition rather than by new surface, recorded so they stay resolved:
suppressing watch’s initial report is `--modified-since now --watch`, and top-N
largest/recent listings are `files` plus `--sort`/`--limit`.

## References

- [Phase 1 plan](plan-2026-08-08-fdu-phase-1.md)
- [CLI UX and agent skill plan](plan-2026-08-09-fdu-cli-ux-and-agent-skill.md)
- [Rollup engine research](../../research/research-2026-08-06-file-rollup-engine.md)
  (Goals 1–7; delta contract; tag-don’t-prune)
- [End-to-end performance evidence research](../../research/research-2026-08-09-end-to-end-performance-evidence.md)
  (benchmark job identity; time-to-first-output vs time-to-complete)
- [Performance frontier research](../../research/research-2026-08-10-performance-frontier.md)
  (verification tiers by reducer; two cache layers; snapshot write economics; journal
  resume fields; the composability rule that scope/view/format never select engine
  variants)
- flowmark-rs cache surface: `attic/flowmark-rs/src/incremental_cache.rs`,
  `src/settings.rs`, `docs/cache.md`, `tests/tryscript/cache-behavior.tryscript.md`
- WHEN grammar prior art: [fd man page](https://www.mankier.com/1/fd)
  (`--changed-within`/`--changed-before` formats),
  [RUSTSEC-2025-0014](https://rustsec.org/advisories/) (`humantime` unmaintained),
  [fd issue #1689](https://github.com/sharkdp/fd/issues/1689) (fd replacing `humantime`)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
