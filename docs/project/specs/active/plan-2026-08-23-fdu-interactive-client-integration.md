# Feature: fdu for Interactive Clients — the Metabrowser Integration Contract

**Date:** 2026-08-23

**Author:** fdu project

**Status:** Draft

## Overview

The README stakes a claim: persistent per-directory roll-ups with incremental
revalidation are “what a live file browser actually needs.”
Metabrowser is that browser — a local Starlette server with a live web front end,
already fast for a Python implementation, already independently converged on fdu’s
architecture, and already naming a native walker as the way past its measured floor.
As fdu approaches its first release, working this integration end to end is the best
instrument available for finding where the API is still shaped for the CLI rather than
for a program.

This spec records what that exercise found.
Much of the contract is already served: the typed Python API walks the reference corpus
roughly 29× faster than the walker it would replace, revalidates an unchanged tree in
about 100 ms, answers per-directory queries in microseconds, and delivers verified watch
batches at whatever throttle the caller asks for.
Five gaps are real, and each generalizes past metabrowser to any embedded consumer — a
TUI, an IDE panel, an agent serving repeated queries:

1. **Concurrent reads during a write.** Measured, not inferred: while `refresh()` runs,
   reader threads raise `FduError: Already mutably borrowed`. A server answering
   requests from a thread pool while a watch batch commits therefore fails those
   requests. The engine already models the right thing — `IndexHandle` serves readers
   during short writes — so this is a binding-layer defect, and it is the one item that
   breaks a naive drop-in outright.
2. **Partitioned tallies.** Metabrowser renders two numbers for every directory —
   everything, and everything not gitignored — and fdu can maintain only one.
   Selection-time filtering is measured two to three orders of magnitude too slow to
   serve it per request.
3. **A customizable roll-up taxonomy.** fdu’s type rules are compiled at build time and
   cannot be varied at runtime at all, and its one grouping axis answers an *analysis*
   question — which analyzer may open this file — rather than a *browsing* one.
   Every image, video, PDF, and archive is therefore a single `binary` row, which is not
   a roll-up a file browser can display.
4. **An embedder-grade watch contract.** The watch layer’s event-driven core is right,
   but a server that lives on an event loop needs an async story, a network-filesystem
   fallback, a way to ingest hints from a foreign watcher, and a per-batch statement of
   which roll-ups changed.
5. **The streaming session**, so a cold open serves requests while the walk runs.
   [The progressive-results plan](plan-2026-08-11-fdu-progressive-results.md) owns that
   design; this spec adds only the integration-facing shape it must land with.

The rest is polish that working the contract surfaced: classification identity in
listings, a machine-readable truncation remainder, walk telemetry for the client’s own
performance loop, and documentation of thread affinity.

## Goals

- A metabrowser build can serve its default UI — dual tallies, live updates, resumable
  change feed, bounded subtree roll-ups — with its walker, inventory aggregation, and
  watcher replaced by fdu, and with no per-request re-aggregation in Python.
- Every capability lands engine-first, then CLI and Python present it, preserving the
  parity contract: nothing here is reachable by flag but not by typed call, or the
  reverse. Classification, grouping, and aggregation in particular stay in Rust: a
  consumer supplies rules and reads typed rows, and never runs a per-entry loop of its
  own to get a roll-up fdu could have maintained.
- Defaults keep answering the question they answer today with no configuration, and
  every axis this spec opens is opt-in.
- The first-release Python API is cleaner for every embedded consumer, not shaped to one
  client: each addition names the question it answers, and metabrowser-specific policy
  stays out of the core.
- The claimed speedup is demonstrated end to end by a committed, testable example, not
  asserted.

## Non-Goals

- Metabrowser-side work.
  The adoption sequence, its feature flag, and its wire models belong to that
  repository; this spec covers the fdu side of the seam.
- The session and provenance design, owned by
  [progressive results](plan-2026-08-11-fdu-progressive-results.md), and the FSEvents
  journal, owned by [its own plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
  This spec consumes both and re-states neither.
- Arbitrary user-defined *tag* vocabularies.
  Two rules — gitignore and visibility — have a consumer today; a tag registry is
  speculation and fails the axis test.
  The *type* registry is a different axis and is explicitly customizable here: it
  already has two divergent real registries, which is the evidence the tag axis lacks.
- Display metadata as engine policy.
  A registry may carry colours and labels through to consumers, but fdu neither
  interprets nor validates them; theming belongs to the client.
- Content-analysis changes.
  The content tier is untouched.

## Background

### The consumer, and what it measures

Metabrowser holds a process-wide inventory of the served tree: one `FsEntry` per path,
per-directory subtree aggregates maintained on every change, a BFS boot walker feeding
it, a `watchfiles`-based watcher keeping it live, and an SSE bus streaming batched
change events to the browser with a resumable cursor.
Every directory row carries four aggregate values — total and gitignore-filtered files
and bytes — plus recursive newest-mtime and per-extension tallies for roll-up views.

Its own load-time review (2026-08-22) identifies the floor three architectural facts
impose: the index is amnesiac, so every boot re-walks everything and the second open
costs what the first did; the walk is sequential interpreted Python behind
`asyncio.to_thread`; and reads are O(N) passes over a heap of dataclasses.
On a 241k-file tree that is ~2.2 s before the first row and ~50 s of attached scanning.
Its hypothesis register names the exit: a native parallel walker (H40, “expected: the
cold scan from 50 s to a few seconds”), a columnar index (H41), and pricing the
gitignore matcher separately from the walk (H39). The caps `max_files=500_000` and
`max_depth=20` — and the `truncated` status they force — exist because a Python walker
cannot go further, as
[the interactive-browser research](../../research/research-2026-08-11-interactive-browser-use-case.md)
already observed.

Since that research was written, metabrowser also shipped the shared file-type taxonomy
(its 2026-08-13 plan, in that repository): it hosts the reference
`recommended-file-types.toml` and serves a `file-type-breakdown-v1` envelope, and fdu
adopted its `[[kind]]` manifest *dialect*. The seam is therefore aligned in grammar but
not yet in vocabulary — metabrowser’s registry is deeper and larger than fdu’s, and only
one of the two has a browsing group level, which is what the taxonomy section below
addresses.

### Orientation measurements, this host

Measured while working this spec, on one 120,001-entry generated corpus (`balanced`
recipe: 105,000 files, 15,001 directories), virtualized Linux, warm page cache, no
`.gitignore` patterns present — a regime that flatters the Python side, since its
matcher had nothing to do.
Single trials through the public Python APIs of both projects: orientation numbers for
this integration, not ledger claims, and not comparable across regimes.

| Moment | metabrowser today | fdu Python API |
| --- | ---: | ---: |
| Cold walk + index, no cache | 5.764 s | 0.201 s |
| Revalidate unchanged tree | re-walks: same 5.764 s | 0.106 s |
| Load from snapshot, no filesystem | not possible | 0.090 s |
| Immediate children of root, with roll-ups | — | 0.32 ms |
| Subtree roll-up query | — | 0.06 ms |
| Bounded tree report | — | 1.03 ms |
| Watch delivery, `interval=0.05` | — | 51 ms steady state |
| Summary, selection-filtered | — | 122 ms |
| Resident memory, one retained index | 99.8 MB | 70.3 MB |
| 3,200 concurrent reads, 16 threads | — | 0.31 s, no errors |
| Reads concurrent with `refresh()` | — | **readers raise** |

Both engines returned identical byte totals for the tree.

Two rows carry design weight.
The filtered summary is the first: an unfiltered summary reads pre-computed roll-up
state in 0.29 ms, while any selection filter pays a full re-aggregating traversal —
about 1 µs per retained entry.
That tier is priced for occasional filtered reports, not for serving every directory
listing twice per navigation on a millions-of-entries tree.

The last row is the second, and it is a defect rather than a cost.
Sixteen threads issuing `children`, `rollup`, and `report` concurrently complete 3,200
calls with no errors, so shared reads are already fine.
But four reader threads running while the main thread calls `refresh()` raise
`FduError: Already mutably borrowed`. PyO3’s runtime borrow check is rejecting what
`IndexHandle` exists to allow, so the exclusion is in the binding rather than in the
engine. A live server commits on every watch batch, so this is not a rare race: it is a
failed request every time a change lands under a reader.

### What is already settled

- **Tag, don’t prune, and the matcher is cheap enough.** The gitignore spike (bead
  `fdu-p35d`, closed) measured the `ignore` crate’s compiled matcher at 0.39–0.51 µs per
  entry with 100 patterns and 1.50–1.76 µs with 1,000, and recorded the design decision:
  an optional compiled tagging mode, never hidden in the default metadata walk.
  This spec is that mode’s specification.
- **The type-rule *dialect* is shared; the registries are not.** Bead `fdu-v4lc` adopted
  metabrowser’s `[[kind]]` manifest shape, and the rules file says so in its first line.
  What that bead delivered was 64 types (now 68) in fdu’s own file, and an earlier draft
  of this spec overstated it as “the compiled registry is the one metabrowser hosts.”
  It is not: metabrowser’s reference registry carries 126 families under six browsing
  *groups* (`archives`, `code`, `data`, `docs`, `media`, `other`) plus display metadata,
  where fdu has 68 kinds, no group level, and five analysis families.
  Same grammar, different vocabulary and one fewer level.
- **Sessions, provenance, lazy open** are specified in progressive results, with the
  session (`fdu-e86o`), its Python mirror (`fdu-a0j0`), attention-following verification
  (`fdu-1mwt`), and lazy warm open (`fdu-hd96`) already tracked there.
- **The integration bead** (`fdu-p02b`) left one question deliberately open — does
  `fdu::watch` replace the client’s watcher outright, or does the client keep its
  watcher and push hints — and this spec resolves it: both, sequenced, on one delta
  contract (see the watch contract below).

## Design

### The seam

| Metabrowser keeps | fdu replaces |
| --- | --- |
| SSE bus, wire models, HTTP surface | boot walker and rewalk paths |
| Plugin discovery and renderers | inventory aggregates and their eviction machinery |
| Active tracker, git history views | navigation tally passes over the index |
| IgnoreMode policy choice (which plane a request reads) | gitignore evaluation and per-entry tagging |
| Watcher *only* on filesystems fdu’s backends cannot serve | watcher and event verification elsewhere |

The client’s `write_token`/generation arbitration disappears rather than being ported:
the index’s clocked delta contract is the same guard, held in one place.

### The contract, requirement by requirement

| Metabrowser needs | fdu today | Verdict |
| --- | --- | --- |
| Boot walk that serves requests ~500 ms in | `open()`/`scan()` block until complete | **Session** (progressive results; shape below) |
| Per-directory recursive tallies, live | `Index.rollup()`, `merge_upward` | Ready |
| Dual all/unignored values on every row | one plane only; selection re-aggregates at ~1 µs/entry | **Gap: partitioned tallies** |
| Hidden-file policy with an allowlist | no visibility concept | **Gap: second tag rule** |
| Children listing with per-child roll-ups | `Index.children()` | Ready |
| Bounded subtree tree with omission accounting | `TreeNode.truncated` is a bare bool | Polish: remainder aggregate |
| Per-extension tallies per directory | `RollUp.by_extension` | Ready |
| Recency queries (top-N by mtime) | `files` view, `sort=mtime` | Ready |
| Per-entry type identity (kind, family, logical ext) | classified internally, not exposed in listings | Polish: expose |
| Browsing groups (media, docs, archives) | all collapse to `family = "binary"` | **Gap: group level** |
| Its own 126-family registry, revised on its own cadence | 68 kinds compiled at build time | **Gap: runtime registry** |
| Bounded per-directory extension rows | `by_extension` is unbounded | **Gap: bound with remainder** |
| Serving reads while a change commits | reader raises `Already mutably borrowed` | **Gap: shared reads** |
| `file-type-breakdown-v1` envelope | same dialect, different vocabulary and depth | Adapter, once groups exist |
| Live change feed, verified, coalesced | `Index.watch()` → typed batches | Ready |
| Event-loop (asyncio) consumption | blocking iterator, thread-affine | **Gap: async adapter** |
| Resumable cursor (SSE `Last-Event-ID`) | `since(clock)`, `ChangeSet.truncated` | Ready — document the mapping |
| “Which roll-ups went stale” per batch | client re-derives from paths | **Gap: dirty set** |
| NFS/FUSE watching (polling fallback) | native backends only | **Gap: poll backend** |
| Foreign-watcher hints | `refresh()` is whole-root only in Python | **Gap: scoped refresh** |
| Second open instant | snapshot load + revalidation (0.09 s + 0.11 s at 120k) | Ready now; instant at millions via lazy open (progressive results) |
| Index status for progress UI | `Status` after the fact | Session exposes it mid-walk |
| Client’s own perf loop instrumentation | counters and `PerformanceSummary` are CLI-only | Polish: expose |

### Concurrent reads during a write

`IndexHandle` exists so readers can be served while a producer applies short writes, and
the engine honours that.
The Python binding does not: PyO3’s runtime borrow check treats `refresh()` as an
exclusive borrow of the whole `Index`, so a reader thread calling `rollup()` during it
raises `FduError: Already mutably borrowed` rather than waiting or reading the prior
state. Measured above, and it is not a narrow race — a live client commits a batch every
time the tree changes, so any request landing in that window fails.

The fix is at the boundary, not in the engine: the Python `Index` holds the handle the
engine already provides rather than an exclusively-borrowed value, so reads take a
shared borrow and mutation takes the handle’s own short write.
Two properties then need pinning by test, because both are what a server depends on: a
read concurrent with a write never raises, and it returns either the pre-write or the
post-write value — never a torn one.

This is the one item on this list that makes a naive drop-in fail rather than merely
cost something, so it leads the implementation plan.

### A customizable roll-up taxonomy

**The defaults are right and stay; what is missing is the ability to vary them.** Two
separate problems, and the second is why the first matters.

**The grouping axis answers the wrong question for a browser.** `ContentFamily` is a
fixed five-value enum — `code`, `prose`, `markup`, `data`, `binary` — and it exists to
decide *which analyzer may open a file*. That is the correct question for the content
tier and the wrong one for a listing: `png`, `jpg`, `mp4`, `mkv`, `pdf`, and every
archive all carry `family = "binary"`, so `--view families` over a photo directory is
one row reading `binary 100%`. Metabrowser’s registry answers the browsing question
instead, with `media`, `docs`, and `archives` as distinct groups.
These are two different taxonomies over the same files, and collapsing them into one
enum is what makes the roll-up useless to a browser.
So the display taxonomy becomes its own axis — a **group** level above families — rather
than a reinterpretation of the analysis one, and the analysis family keeps meaning
exactly what it means today.

**Nothing is variable at runtime.** The rules are compiled by `build.rs` into `OUT_DIR`
and the module says plainly that runtime never parses TOML. A consumer wanting a type
fdu does not know, or a grouping fdu does not ship, has exactly two options today:
rebuild the crate from a patched file, or reclassify in its own language — which for
metabrowser means keeping the Python classifier this integration exists to delete, and
re-deriving groups per request over hundreds of thousands of entries.
Neither is acceptable, and the second is the same per-request re-aggregation the
partitioned-tallies section rejects for the same reason.

The shape, then:

- **The compiled registry remains the default and the fast path.** Zero configuration,
  no file to find, no parse at startup, and the CLI’s behaviour is unchanged.
  A caller that names no registry gets exactly what it gets today.
- **A caller may supply its own registry** as a typed value — built in the engine from
  the same `[[kind]]` dialect, with the group level added — and every surface accepts
  it: `ScanOptions`/`AnalysisOptions` in Python, the corresponding scope axis on the
  command line, `OpenConfig` in Rust.
  Parsing, validation, indexing, and grouping all happen in Rust; Python hands over a
  path or a manifest and receives typed rows back, never a classification loop.
- **Grouping is maintained, not recomputed.** A registry’s groups are roll-up state on
  the same reducer path as extensions, so a per-directory group breakdown is a
  pre-computed read, not a traversal.
  This is the same argument as planes, and it is why the work belongs in the engine.
- **The registry versions the cache, and the plumbing already exists.**
  `type_rules_fingerprint` is already carried in the scan scope and the content model
  and already compared for validity; it reads a `const fn` today and must read the
  active registry’s fingerprint instead.
  A rule change then invalidates exactly what it should, by the mechanism the design
  principles already require for a bucketing change.
- **Bounded per-directory extension tallies.** `RollUp.by_extension` is an unbounded map
  per directory; metabrowser bounds its equivalents (`ext_top`, `filename_top`,
  `remaining_top`) because a browser shows a handful of rows and a wide tree has many.
  A bound with a stated remainder belongs here for the same reason it belongs on trees.

**Sequencing against PR #38.** That PR replaces the per-file linear scan of the rules
table with two `LazyLock<HashMap<&'static str, &'static GeneratedRule>>` statics, worth
about 5% on warm content jobs.
Process-global statics over `&'static` data are exactly what a per-instance registry
cannot be, so the two changes meet in one file — but they do not conflict in substance.
The indexed-lookup *shape* is what the win comes from, and it survives being owned by a
registry value instead of by a static; what changes is lifetime, not algorithm.
So #38 lands first and this work converts its statics into a field on the registry,
keeping the index and keeping its test.
That test — `indexed_rule_tiers_agree_with_the_scan_they_replaced`, which pins
`max_by_key`’s last-wins tie-break — generalizes rather than retires: it becomes a
property over *any* registry, which is worth more once registries are user-supplied and
a tie-break bug can no longer be caught by reading one committed file.

### Partitioned tallies

The browser’s default listing shows, for every directory, values *as filtered by
gitignore* next to values for everything.
Serving that from selection costs a re-aggregating traversal per request per plane —
measured at ~1 µs/entry, which is seconds per navigation at home-folder scale.
Metabrowser maintains both aggregates on every mutation instead, and fdu must offer the
same: this is pre-computed roll-up state, the thing the index exists to hold.

**Tags are observations; planes are maintained aggregates.**

- `ScanOptions` (and the corresponding CLI scope axis) gains an explicit, off-by-default
  tag configuration naming rules from a small fixed vocabulary: `gitignore` (the
  `ignore`-crate compiled matcher, correct negation semantics — which retires the whole
  class of the hand-rolled prefixing bug metabrowser’s review calls F3) and `hidden`
  (dot-prefix, with a configurable allowlist so a client policy like “`.logs` and
  `.state` stay visible” is configuration, not a fork).
  Each entry carries its tag bits; tagging happens during the walk at the spike’s
  measured per-entry cost, and the watch layer re-tags on change.
  An observed change to a governing `.gitignore` escalates to subtree invalidation —
  re-tagging a subtree is exactly what `InvalidateSubtree` already expresses.
- For each enabled tag, every directory’s roll-up maintains one additional **plane**:
  the same fields it keeps today (files, dirs, bytes, allocated, newest mtime,
  per-extension tallies) restricted to entries not carrying the tag.
  Planes ride the existing reducer path; they multiply its cost, which is why they are
  opt-in and explicit.
- Tag rules change what stored records mean, so the enabled rule set — allowlist
  included — versions the snapshot fingerprint, exactly as bucketing rules do.
  A snapshot recorded under different rules is absent, never reinterpreted.
- Selection gains one axis value: `plane` (Python `Selection(plane=...)`, CLI
  `--plane`), defaulting to `all`. Any view can then answer “as the browser shows it”
  from pre-computed state, and the two-tier cost rule is preserved: an unfiltered
  single-plane request stays a roll-up read; combining `plane` with other filters
  re-aggregates as any filter does today.
  `RollUp`, `Child`, and `TreeNode` carry per-plane values and per-entry tag bits, so
  one `children()` call serves the dual-value listing.
- The partition rule holds per plane: for every enabled tag, tagged plus untagged equals
  the whole, asserted by test, so no entry can fall out of a tally.

What this deliberately is not: a general predicate store.
A tag is a named, versioned, compiled rule the engine owns end to end — that is what
keeps the cache honest and the cost stated.

### The embedder watch contract

The watch layer’s semantics are already what a live client needs: event-driven
detection, stat-verified samples, coalescing, explicit escalation, and a logical clock.
Four additions make it embeddable rather than merely correct:

1. **An async adapter.** `Watch` stays a thread-affine blocking iterator — that is the
   honest native shape — but the package ships and documents the event-loop handoff (a
   worker thread feeding an `asyncio` queue, yielding the same typed batches), so every
   asyncio consumer does not reinvent it around the GIL. The thread-affinity contract
   gets documented either way.
2. **A per-batch dirty set.** The engine already knows which ancestors `merge_upward`
   touched; the batch says so — the set of paths whose roll-ups changed since the last
   batch. That is precisely the client’s “projection invalidate” signal, computed where
   the knowledge lives instead of re-derived from change paths in Python.
3. **A polling backend.** `WatchOptions` gains an explicit backend choice (native/poll,
   with the poll interval stated), because network and FUSE filesystems drop native
   events silently, and metabrowser already knows this and downgrades.
   Whether the *engine* should probe the mount table and choose automatically is an open
   question below; an explicit knob is the honest first step and is what the client’s
   existing detection can drive.
4. **Scoped refresh.** `Index.refresh(path=...)` exposes the subtree reconciliation the
   engine already performs internally.
   This is the hint-ingestion primitive: a client keeping its own watcher for an exotic
   mount pushes each hint as a scoped refresh, and every mutation still flows through
   the one delta contract.
   This resolves `fdu-p02b`’s open question — `fdu::watch` where its backends serve the
   filesystem, client hints through scoped refresh where they do not, and the acceptance
   test for dropping the Python watcher is the cross-engine fixture in the testing
   strategy.

The resumable-cursor mapping needs no new machinery — `since(clock)` with
`ChangeSet.truncated` already models SSE resume including the “gap too large, resync”
case — so it becomes documentation with a tested example rather than API.

### The session surface

Progressive results owns the design (`Session::start` / read-anytime / cancel;
breadth-first monotone lower bounds; per-path freshness; `prioritize`). Working the
integration adds three requirements to land with it, not after it:

- **Progress is part of the surface**: entries applied so far, roll-up clock, and
  completeness, readable mid-walk, because the client renders a crawl-status UI from
  exactly that today.
- **The async shape ships with the sync one**, same adapter policy as watch.
- **A session hands off to a watch** without a gap: the clock at which the walk
  completed is the clock the watch resumes from, so a client can open, stream the fill,
  then follow changes, with no window where a mutation is neither in the walk nor in the
  feed.

### Classification identity in listings

`children()` and `files`-view rows gain the compiled registry’s verdict — type id,
family, and logical extension — as plain metadata fields.
The registry already runs during classification for the type views; exposing its
per-entry identity costs display only, reads no content, and is what lets the client
drop its own classifier while keeping its wire models.
The registry’s identity (schema version, revision, fingerprint) becomes readable from
Python, since the client stamps exactly that into its envelopes today.

### Instrumentation for the client’s own loop

Metabrowser runs its own measured performance loop.
`Report` and session/watch surfaces expose the walk telemetry the CLI footer already
computes — files and bytes walked, cache tier, fresh versus cached analysis — as typed
values (the envelope keeps excluding them; they are execution telemetry, delivered
beside the report, not query data).
The `FDU_COUNTERS` mechanism stays an environment concern and is documented for Python
consumers rather than wrapped.

### Polish

- `TreeNode` bounds state a remainder: when children are omitted, the node carries the
  aggregate of what was dropped (dirs, files, bytes per plane), machine-readable —
  “truncate freely, never silently” applied to the one place it is still a bare bool.
  A treemap’s “other” cell is this value.
- Document that `children()` on a pathologically wide directory returns everything, and
  that the bounded alternative is the tree view; whether `children()` needs its own
  bound is an open question.
- `WatchOptions.interval` default (2.0 s) is tuned for terminal repaints; embedder
  documentation states that live UIs set it near their frame budget (measured: 51 ms
  end-to-end at `interval=0.05`).

## Implementation Plan

Tracked under epic `fdu-u7vo`; each item names its bead.

### Phase 0: Shared reads during a write

First because it is the only outright blocker, and it is small.

- [ ] Python `Index` reads take a shared borrow over the engine handle rather than an
  exclusive borrow of the whole object, so `refresh()` and watch commits no longer raise
  in reader threads; tests pin that a concurrent read never raises and never returns a
  torn value (`fdu-gav9`)

### Phase 1: Partitioned tallies

- [ ] Tag rules in the engine: compiled gitignore and hidden-with-allowlist matchers,
  entry tag bits, opt-in `ScanOptions` configuration, snapshot fingerprint coverage;
  planes through the reducer path — per-plane roll-up state, `merge_upward`, refresh and
  watch re-tagging, `.gitignore`-edit escalation; partition-sum property tests and
  fingerprint invalidation (`fdu-mvt3`)
- [ ] Surfaces: `--tags`/`--plane` on the CLI, `Selection.plane` and per-plane
  `RollUp`/`Child` values in Python, tagged-fixture goldens in every format, parity
  rows, and plane-equals-all equivalence when no entry is tagged (`fdu-7rwf`)

### Phase 1b: The customizable taxonomy

Lands after PR #38, converting its statics rather than competing with them.

- [ ] A `group` level in the rule dialect and in roll-up state, so a browsing taxonomy
  is its own axis rather than a reinterpretation of the analysis families; the compiled
  default registry gains groups and the `groups` view renders them (`fdu-b2vy`)
- [ ] A runtime-supplied registry: parse, validate, index, and fingerprint in Rust;
  accepted by `OpenConfig`, `ScanOptions`/`AnalysisOptions`, and the CLI scope axis;
  `type_rules_fingerprint` reads the active registry so a rule change invalidates the
  snapshot and sidecar through the path that already exists.
  PR #38’s `LazyLock` statics become a per-registry index, and its tie-break test
  generalizes to a property over any registry (`fdu-ctp5`)
- [ ] Bounded per-directory extension and filename rows with a stated remainder
  (`fdu-e2p7`)

### Phase 2: The embedder watch contract

- [ ] Per-batch dirty roll-up set, engine through Python (`fdu-mz1a`)
- [ ] `Index.refresh(path=...)` scoped reconciliation in the Python surface (`fdu-fh0k`)
- [ ] Polling backend selection in `WatchOptions`, with its interval stated (`fdu-rhu3`)
- [ ] The asyncio adapter and the thread-affinity documentation, with a tested
  SSE-resume example mapping `since`/`truncated` to `Last-Event-ID`/resync (`fdu-97pb`)

### Phase 3: Session integration shape

(after the progressive-results session beads land the core; `fdu-4o0m`, blocked by
`fdu-e86o` and `fdu-a0j0`)

- [ ] Mid-walk progress surface: entries applied, clock, completeness
- [ ] Async session adapter, same policy as watch
- [ ] Session-to-watch clock handoff, tested for the no-gap property

### Phase 4: Adoption proof

- [ ] Classification identity in `children()` and files rows; registry identity readable
  from Python (`fdu-16l7`)
- [ ] Walk telemetry as typed values beside report/session/watch results (`fdu-tib6`)
- [ ] `TreeNode` remainder aggregates (`fdu-knyw`)
- [ ] Reference embedder example under `crates/fdu-py/examples/` — boot, serve dual
  tallies, stream changes with dirty sets, resume from a cursor — plus the cross-engine
  agreement fixture comparing fdu against the metabrowser walker’s semantics (symlinks
  as leaves, hidden allowlist, gitignore negations), differences documented or
  eliminated (`fdu-vfyw`)

## Testing Strategy

Two properties are load-bearing and both are cheap to state.
Partition sums: for every enabled tag, plane plus complement equals the untagged totals,
by property test across scan, refresh, and watch mutations; and for a registry, every
group’s families and every family’s types sum to what the same selection reports, so a
custom registry cannot lose a file the way the extension view once did.
Concurrency gets the pair named in its section — a read during a write never raises and
never tears — driven by the existing thread test, which found the defect.
Registry validation is tested for what it rejects as much as what it accepts: duplicate
ids, an unknown group, a tie-break ambiguity, and a manifest whose fingerprint collides
with the compiled default.
Golden sessions add a tagged fixture exercising `--tags`/`--plane` in every format, and
the parity harness replays them against Python as it does every axis.
Watch additions get the same treatment the layer already has — dirty sets asserted
against independently computed ancestor sets; scoped refresh asserted equivalent to full
refresh on the touched subtree; the async adapter driven by the existing watch tests
through the event loop.
The reference embedder is executed in CI like the other examples, and the cross-engine
fixture is the acceptance test `fdu-p02b` asked for.
Performance claims follow the loop: browser-moment jobs (cold with tags, warm unchanged,
plane query, watch steady state) join the benchmark manifests, and any constant tuned
here records its regime.

## Open Questions

- Are two maintained planes (all, unignored) the shipped set, or does the client’s
  show-all mode justify a third (visible)?
  Per-plane cost is real; measure the reducer overhead before enabling more than one
  extra plane by default in the tagged mode.
- Should the engine probe mount tables to choose the watch backend, or stay explicit and
  let clients own detection?
  Explicit ships first; probing is additive.
- Does `children()` need its own bound, or is the tree view’s remainder enough for wide
  directories?
- Should the two projects converge on one registry file, or stay two registries over one
  dialect? Convergence is attractive and couples release cadences: metabrowser could not
  then add a file type without an fdu release.
  A runtime registry makes the question answerable later rather than now, which is part
  of why it comes first.
- Does a runtime registry cost measurable throughput against the compiled one?
  The index shape is the same, but its strings are owned rather than `&'static`, so the
  comparison is a loop job on the same subject as PR #38’s.
- Free-threaded CPython (3.14t) wheels: metabrowser’s H48 expects real parallelism in
  Python; fdu’s abi3 wheel does not cover the t-ABI today.
  Out of scope here, tracked as a release-matrix question.

## References

- [Design principles](../architecture/fdu-design-principles.md) — the axis rules, cache
  honesty, and truncation contracts this spec extends
- [Surface architecture](../architecture/fdu-surface-architecture.md) — the parity
  harness every addition here must clear
- [Interactive browser research](../../research/research-2026-08-11-interactive-browser-use-case.md)
  — the three deadlines and the four-consumer comparison
- [Progressive results plan](plan-2026-08-11-fdu-progressive-results.md) — sessions,
  provenance, lazy open
- [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
  — the convergence half for warm opens
- PR #38 (`perf(content)`: roll-up ancestor walk and indexed type-rule tiers) — the
  indexed classification tiers this spec’s registry work converts from statics into
  per-registry state, and the tie-break test it generalizes
- Beads: `fdu-p02b` (the integration), `fdu-p35d` (the gitignore spike verdict),
  `fdu-v4lc` (the shared type-rule dialect), `fdu-e86o`/`fdu-a0j0`/`fdu-1mwt` (the
  session), `fdu-hd96` (lazy open priority)
- Metabrowser: its load-time performance review (2026-08-22) and hypothesis register
  (H39–H41, H47–H48), and its shared file-type taxonomy plan (2026-08-13), in that
  repository

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
