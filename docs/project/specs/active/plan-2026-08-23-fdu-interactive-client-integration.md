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
   fdu already streams *changes* — the watch feed is real, verified, and resumable — but
   it does not stream the *walk*: `scan()` and `open()` return only when complete, and a
   watch over an idle tree yields empty batches, because it answers “what changed”, not
   “what is here”. That missing half is also what makes fdu’s breadth-first default inert
   today, since no consumer can observe the order it pays for.
   [The progressive-results plan](plan-2026-08-11-fdu-progressive-results.md) owns that
   design; this spec adds only the integration-facing shape it must land with.

The rest is polish that working the contract surfaced: classification identity in
listings, a machine-readable truncation remainder, walk telemetry for the client’s own
performance loop, and documentation of thread affinity.

Reading this against
[the metabrowser plan](https://github.com/jlevy/metabrowser/blob/954b6ed/docs/project/specs/active/plan-2026-08-23-pluggable-inventory-engine.md)
added two gaps this exercise had graded as served, and both are recorded in
[the reconciliation](../../research/research-2026-08-23-interactive-contract-reconciliation.md).
A **coherent read surface**: `children()` copied every child’s whole extension map, and
nothing tied several Python calls into one version, so a composed response could
straddle a commit. And a **second extension level**: fdu derived the canonical extension
the File Rollup Format wants — `release.v2.zip` already classified as `archive` and
bucketed as `.zip` — but not the raw two-component value beside it, and swapping the
derivation rather than adding the level would have turned that same archive into
`unknown:.v2.zip`. Both are now built; the phase lists below carry which beads closed.

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
  One rule — gitignore — has a consumer today; a tag registry is speculation and fails
  the axis test. Visibility looked like a second rule and turned out to be scope: hidden
  paths are pruned rather than tagged, for the reasons in the partitioned-tallies
  section. The *type* registry is a different axis and is explicitly customizable here:
  it already has two divergent real registries, which is the evidence the tag axis
  lacks.
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
| Entries streamed as the first walk discovers them | watch streams changes only; an idle watch yields empty batches | **Session**; the two senses of streaming are distinguished below |
| A directory row before its total is known | no placeholder; a roll-up exists or does not | **Session** (two-phase yield) |
| Breadth-first, stated rather than assumed | BFS is the default but neither surface can name it | **Gap: expose `ScanOrder`** |
| Per-directory recursive tallies, live | `Index.rollup()`, `merge_upward` | Ready |
| Dual all/unignored values on every row | one plane only; selection re-aggregates at ~1 µs/entry | **Gap: partitioned tallies** |
| Hidden-file policy with an allowlist | no visibility concept | **Gap: scope admission rule** |
| Children listing with per-child totals | `Index.children()`, paged, scalar rows | Ready |
| Bounded subtree tree with omission accounting | `TreeNode.truncated` is a bare bool | Polish: remainder aggregate |
| Per-extension tallies per directory | `RollUp.by_extension` | Ready |
| Recency queries (top-N by mtime) | `files` view, `sort=mtime` | Ready |
| Per-entry type identity (kind, family, logical ext) | classified internally, not exposed in listings; the raw logical extension is not derived at all | Polish: expose, plus **Gap: raw extension level** |
| Browsing groups (media, docs, archives) | all collapse to `family = "binary"` | **Gap: group level** |
| Its own 126-family registry, revised on its own cadence | 68 kinds compiled at build time | **Gap: runtime registry** |
| Bounded per-directory extension rows | `by_extension` is unbounded | **Gap: bound with remainder** |
| Serving reads while a change commits | reader raises `Already mutably borrowed` | **Gap: shared reads** |
| `file-type-breakdown-v1` envelope | same dialect, different vocabulary and depth | Adapter, once groups exist |
| Live change feed, verified, coalesced | `Index.watch()` → typed batches | Ready |
| Event-loop (asyncio) consumption | blocking iterator, thread-affine | **Gap: async adapter** |
| Resumable cursor (SSE `Last-Event-ID`) | `since(clock)`, `ChangeSet.truncated`, but trust transitions never reach it | **Gap: trust on the clock**, then document the mapping |
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

#### The extension has two levels, and fdu has only the second

File Rollup Format derives a **raw** logical extension of up to two eligible trailing
components, then **suffix-matches** it to a canonical extension that drives rule lookup
and roll-up bucketing.
`release.v2.zip` derives `.v2.zip` and matches canonical `.zip`; `bundle.umd.min.js`
derives `.min.js` and matches canonical `.js`.

fdu produces the canonical answer already.
Run on a fixture, `release.v2.zip` classifies as `archive` and buckets as `.zip`, and
`bundle.umd.min.js` classifies as `javascript` and buckets as `.js` — which is what the
format wants. What fdu lacks is the raw level, and the naive fix breaks the working one:
`classify_path_with_prefix` looks rules up by exact key in `RULES_BY_EXTENSION` with no
suffix fallback, so returning `.v2.zip` from `derive_ext` would miss every rule and
yield `unknown:.v2.zip`, while `ext_bucket` — the same function — would split the `.zip`
bucket at the same time.
One edit, two regressions, in exactly the names the change was meant to serve.

So this lands as a pair, not a replacement:

- a **raw logical extension** following the format’s eligibility rule, carried on
  entries and exposed in the projections that want it — navigation tallies, literal
  filters, recent and catalog rows, and unknown `remaining_types` keys;
- **canonical suffix matching**, so a raw extension matching no rule falls back to its
  trailing component for both rule lookup and the roll-up bucket.

The property to pin: adopting the raw level moves no existing bucket and no existing
type row. Eligibility belongs in the rule dialect rather than a hand-maintained list,
which is what `derive_ext`’s own comment already asks for — now as one of two levels.
The conformance packet gains direct basename-to-logical-extension cases before it can
serve as fdu’s oracle, since today it tests matching rather than derivation.

**What this inherits from PR #38, now merged.** That work replaced the per-file linear
scan of the rules table with two
`LazyLock<HashMap<&'static str, &'static GeneratedRule>>` statics, worth about 5% on
warm content jobs on top of a larger roll-up fix.
Process-global statics over `&'static` data are exactly what a per-instance registry
cannot be, so the two changes meet in one file — but they do not conflict in substance.
The indexed-lookup *shape* is where the win comes from, and it survives being owned by a
registry value instead of by a static; what changes is lifetime, not algorithm.
So this work converts those statics into a field on the registry, keeps the index, and
keeps its test.
That test — `indexed_rule_tiers_agree_with_the_scan_they_replaced`, which
pins `max_by_key`’s last-wins tie-break — generalizes rather than retires: it becomes a
property over *any* registry, which is worth more once registries are user-supplied and
a tie-break bug can no longer be caught by reading one committed file.
`type_rule_fingerprint()` is a `const fn` over a compiled constant for the same reason
and becomes a value read from the active registry.

That PR also merged
[the evidence-scope plan](plan-2026-08-23-experiment-evidence-scope.md), whose finding —
that a number measured on one subject travels as a general claim unless the record stops
it — is why this spec’s comparison table states its corpus, host, cache state, and
single-trial status inline rather than in a footnote.

### Partitioned tallies

The browser’s default listing shows, for every directory, values *as filtered by
gitignore* next to values for everything.
Serving that from selection costs a re-aggregating traversal per request per plane —
measured at ~1 µs/entry, which is seconds per navigation at home-folder scale.
Metabrowser maintains both aggregates on every mutation instead, and fdu must offer the
same: this is pre-computed roll-up state, the thing the index exists to hold.

**Tags are observations; planes are maintained aggregates.**

- `ScanOptions` (and the corresponding CLI scope axis) gains an explicit, off-by-default
  tag configuration naming one rule: `gitignore` (the `ignore`-crate compiled matcher,
  correct negation semantics — which retires the whole class of the hand-rolled
  prefixing bug metabrowser’s review calls F3). Each entry carries its tag bits; tagging
  happens during the walk at the spike’s measured per-entry cost, and the watch layer
  re-tags on change. An observed change to a governing `.gitignore` escalates to subtree
  invalidation — re-tagging a subtree is exactly what `InvalidateSubtree` already
  expresses.
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

**Hidden paths are scope, not a tag.** An earlier revision made visibility a second tag
rule with its own plane.
It is instead an admission rule: hidden components are outside the scan scope except for
a configured exact-name allowlist, and the excluded subtree is never walked.
A plane would still have to walk `.git`, caches, and virtualenvs — routinely the largest
part of a working tree — in order to exclude them, and the toggle that would have
justified paying for that does not exist in any consumer.
Governing control files stay readable without being retained, so `.gitignore` can be
parsed and a repository root detected inside a pruned subtree.
The admission rule and its allowlist are semantic scope, so they are fingerprinted into
snapshot identity like any other change to the retained set.
fdu’s own CLI default is untouched: a du replacement counts everything, and hidden
exclusion is opt-in scope configuration presented under the usual parity rules.

### Where the cost of both lands

Planes and groups are the same kind of change measured from opposite ends: each adds
maintained per-directory state, so together they multiply the ancestor-merge path rather
than adding to it once.
That is precisely the path exp-064’s H94 just made cheap — its `merge_ancestors` went
from 43.73% of profile to 14.07% — and precisely the path campaign 2 plans to delete
rather than tune, in the `fdu-cq7t` follow-on it names the content-tier instance of H86:
key roll-ups by `EntryId`, defer to one bottom-up pass.

Two consequences, both following the floor rule that a tier close to its floor is a
result rather than a place to keep digging.
Adding state to a path scheduled for structural replacement should be measured against
that replacement’s shape, not against today’s — a per-plane, per-group cost that looks
acceptable on the current ancestor walk may be the wrong measurement entirely.
And these features supply what that structural work has so far lacked: a consumer whose
requirements make the multiplication real, on a dense subject rather than a generated
one. The loop job (`fdu-n4gn`) belongs in both specs, and campaign 2 has since built the
instrument it needs: `make perf-subjects` nominates a host’s real trees by size and
density, and a subject may decide an accept when it is dense and at least 50,000
entries. So the subject is chosen by that rule rather than by hand, for the reason
exp-065 established — a sparse generated corpus flatters exactly this class of change.

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

**One commit carries rows and state alike.** A fifth addition turned out to be
load-bearing rather than cosmetic.
Coverage, trust, the run envelope, and the tag rules all decide what a projection
*means*, and each used to move by direct mutation: no clock, no journal entry, nothing a
consumer could resume against.
One cursor could therefore name the answer before a transition and the answer after it,
with nothing in either saying which had been read.
So a committed delta now carries `state` beside `ops`, a state-only delta is ordinary,
and the transitions ride the clock that already orders the rows they describe.
Three consequences worth stating, because each was a defect before it was a rule:

- A reconciliation sweep that finds no difference at all still advances the version.
  It promoted the provenance of everything it stat’d, and the sweep — not the entry — is
  the unit, which is what keeps a sweep over millions of entries from emitting millions
  of provenance-only changes.
- A re-tag commits before the batch that caused it reports a position, so the cursor
  follows the transition rather than preceding it.
- A batch that stepped over another producer’s commit is a `reset`. An index has one
  writer at a time but not one producer — a caller can refresh or ingest hints against
  the same handle while a watch runs — and a batch naming a position past a commit it
  did not deliver drops that commit for good.

### Two things called streaming, and the one that is missing

fdu streams, and it is worth being exact about what: the watch session is a real change
feed, verified, coalesced, clocked, and resumable through `since()`. What it is not is a
feed of the tree. `Session::new` in `watch_session.rs` says so in its own first line —
*“Start watching an already-opened index”* — and a watch over an idle tree yields
nothing at all: measured here, forty consecutive batches, every one empty.
So `watch()` answers “what changed since I started”, never “what is here”.

Metabrowser streams the other one, which is the half fdu lacks.
`walk_tree` is an async generator over a strict `popleft()` BFS queue, and it yields
twice per directory on purpose: a placeholder as soon as the directory is discovered,
carrying `total_* = None`, and its finalized form later as the sweep completes beneath
it, deepest first with the root last.
The inventory batches those yields at 256 into one SSE `FsChange` and pushes them, so
the browser paints rows the moment they exist and fills their totals in as they settle.
That two-phase yield *is* the skeleton-then-converge UI, produced by the walker rather
than simulated above it.

Setting the two side by side names the gap precisely:

|  | initial walk | subsequent changes |
| --- | --- | --- |
| metabrowser | streamed, batched at 256, two-phase per directory | streamed |
| fdu | **blocking; no read until complete** | streamed |

### Traversal order is already right, and currently inert

fdu walks breadth-first by default and has since the progressive-results work landed:
`ScanOrder` is a public engine type, a field on `ScanConfig`, region-scheduled across
top-level subtrees, and measured faster than depth-first on the large heterogeneous tree
(exp-037). Metabrowser derived the same answer independently, queueing BFS to a
first-render depth.

Two problems follow, and the second is the interesting one.

**Neither surface exposes it** (`fdu-4vkz`). There is no `--order` flag and no
`ScanOptions` field; `ScanOptions` carries `max_depth` and `one_filesystem` and nothing
else. A Rust caller may choose the order, and a command-line or Python caller may not —
the mirror image of the rule that the command line invents nothing, and the same defect,
since a capability reachable from one surface and not the others is unfinished either
way. The progressive-results plan recorded `--order` as landed “on the probe”, and the
probe is not a public surface.

**More importantly, the order cannot currently pay for itself.** Breadth-first exists so
that a consumer reading mid-walk compares partial values against each other rather than
against zeros. No Python or command-line consumer can read mid-walk, so today fdu pays
breadth-first’s costs and collects none of its benefit; the research already said as
much — a one-shot run “sees nothing, so it should take whichever order is cheapest”.
That is the sharpest argument for the session in this document.
It is not a new feature so much as the thing that makes an already-shipped property
observable, and it is why metabrowser is snappy in interpreted Python while fdu, far
faster per entry, would still show an empty pane until the walk ends.
Its measured wins are all of that shape: first row from 1,604 ms to 242 ms by rendering
inline, and server scanning time from 650 ms to 2 ms by letting rows stop waiting on
tallies. Neither is a faster walk.

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

#### How the stream crosses the boundary, and why it is not a callback

This changes no structure, because both halves already exist and are already the right
shape. It is worth writing down, since “add streaming” sounds like it implies callbacks
and here it must not.

**In Rust, the callback is already there and stays.** `scan` takes
`sink: &mut dyn FnMut(Observation)` and `reconcile` takes
`sink: &mut dyn FnMut(&AppliedDelta)`; the cold walk has always streamed to a sink.
`scan_into_index` is the convenience wrapper that swallows it and hands back a finished
index, and that wrapper is what the Python `scan()` calls — while the reconcile path
passes a literal `&mut |_| {}`. So the walk is not silent; the binding is deaf.
That is a much smaller thing to fix than adding a producer.

**Across the FFI boundary there is no callback, and there must not be.** A
Rust-to-Python callback has to hold the GIL for every invocation, on whichever worker
thread the walk is running, at per-entry frequency.
The walk is parallel and region-scheduled, so that serializes every worker on the GIL
and destroys the property that makes fdu worth adopting.
It also inverts control, which an event loop cannot accept, and it has no backpressure
story: a slow Python callback blocks a walker thread.

**The adapter is a bounded queue, and the watch layer is the working example.** The Rust
sink pushes into a queue; Python pulls.
`Watch.__next__` already does exactly this — `py.detach(|| session.next_batch(timeout))`
— so the GIL is taken once per *batch* rather than once per entry, and native work runs
with it released. The scan session is the same pattern pointed at the other producer,
which is what the architecture already claims: `scan.rs` and `watch.rs` are both
metadata-delta producers, and the index is the consumer.

**Backpressure is already modelled.** The change feed is bounded and a consumer that
falls behind is told so rather than blocking the producer — `Since::truncated`, the same
signal a watch overflow raises, which a client answers with a resync.
A slow reader degrades to “you missed some, re-read” and never to a stalled walk.

**One consequence worth stating, because it simplifies the client.** Both producers mint
the same delta type, so a consumer sees one stream shape for the boot fill and for live
changes. Metabrowser already converged on this independently: its walker and its watcher
both emit `FsChange(ops=(FsUpsert…))` over the same SSE channel.
So the boot path and the live path collapse into one code path on both sides of the
seam.

Metabrowser’s two-phase yield maps onto this without needing a second mechanism: where
it emits a placeholder and later a finalized row, fdu emits one delta whose roll-up
grows through `merge_upward`, with per-path status moving `Partial` to `Complete`. Its
two states are the approximation; provenance is the general form.

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
- `children()` takes its own bound, and now does.
  A listing is paged by a row limit and a name cursor, its rows carry scalar subtree
  totals, and the page states what it does not carry.
  The per-extension breakdown moves to `rollup()` for the one directory being inspected,
  because per row it cost a map clone per child to render one number per child.
- `WatchOptions.interval` default (2.0 s) is tuned for terminal repaints; embedder
  documentation states that live UIs set it near their frame budget (measured: 51 ms
  end-to-end at `interval=0.05`).

## Implementation Plan

Tracked under epic `fdu-u7vo`; every item names its bead, and every bead under the epic
appears here. Two beads linked to this spec are deliberately not phase items: `fdu-p02b`
tracks the metabrowser-side adoption, and `fdu-eu8t` is the older “specify a progressive
Python session” request that Phase 5 and Phase 6 between them answer.
Both stay open until the work they describe lands.

**This list says what to build; the bead status says what is left.** Much of it is
already closed — shared reads, `ScanOrder` on the surfaces, the group level, the runtime
registry, bounded extension rows, the watch contract’s four items, and listing-row
identity among them — with the file-and-function map and the implementation itself in
[the implementation spec](plan-2026-08-23-fdu-interactive-client-implementation.md) and
the branch stacked on this one.
Checkboxes here are not flipped as that work lands, because the code arrives in the
stacked branch rather than in this PR; read the tracker for current state and this
document for intent.
Two items that survived their bead are worth naming: the two extension levels and the
conformance packet split out to `fdu-5q6e` when `fdu-ctp5` closed carrying the registry
alone.

**These phases are fdu’s own order, and nothing here waits on the client.**
[Metabrowser’s refactor spec](https://github.com/jlevy/metabrowser/blob/3e563a8/docs/project/specs/active/plan-2026-08-23-inventory-provider-refactor-and-fdu-adoption.md)
ships its Python provider behind the sealed contract first, with no fdu dependency, and
only then implements the same contract against this engine.
So fdu refines and builds its native prerequisites independently rather than
interleaving with that refactor.
Phase 2 should still run before or beside Phase 1 — every cross-engine oracle depends on
classification agreement, while the plane needs it only at validation time — but that is
an fdu sequencing judgment, not a client dependency.
The conformance packet gates *verification* of the classification work, not the work
itself.

**One moment is genuinely coupled, and it is worth building toward.** Metabrowser’s
Phase 2 opens with the smallest real PyO3 spike: open a shared handle, perform one
bundled directory-plus-roll-up read returning a single version, cursor, state, and work
record, and converge after one live mutation with no mirror index in Python.
That slice is where this contract first meets an actual consumer, and a seam that
translates badly there revises both documents before either surface expands.
It draws on Phase 0’s shared reads and Phase 3’s bundled read, which is the argument for
landing those two early even though the phase list would otherwise let them drift.

### Phase 0: Two surface defects

Both are cases where the engine can already do the thing and no surface can reach it,
and both are small. `fdu-gav9` leads because it is the only outright drop-in blocker.

- [x] Python `Index` reads take a shared borrow over the engine handle rather than an
  exclusive borrow of the whole object, so `refresh()` and watch commits no longer raise
  in reader threads; tests pin that a concurrent read never raises and never returns a
  torn value (`fdu-gav9`)
- [x] `--order` on the scope axis and `ScanOptions.order` in Python, with goldens and
  parity rows, so the breadth-first default fdu already pays for is a stated contract
  rather than an engine-only one (`fdu-4vkz`)

### Phase 1: Partitioned tallies

Restructured 2026-08-24, after the owner directed that gitignore be one flag among
several rather than the feature’s name.
The model that results, stated once here and in detail on the beads:

- **Tags and planes are decoupled.** A tag is a named boolean fact stored as one bit
  beside `ext_id` and `group_id` — unbounded and cheap.
  A plane is a maintained aggregate for a rule explicitly *promoted*, and promotion is a
  small declared subset, because planes ride the ancestor-merge path and tags do not.
  Filtering by an unpromoted tag re-aggregates by walking — the two-tier rule the query
  surface already applies to every other predicate.
  The 1:1 coupling this replaces had already forced `hidden` out of the model once.

- **Rules carry a tier** — Name, Path, or Content — declaring what they may read.
  Content-tier rules are rejected at enable time in v1 with an error naming the cost
  class, so a future `binary` tag cannot silently turn a metadata walk into a content
  walk.

- **Categorical facts are not tags.** Mime type and its kin are interned-key tally maps,
  the mechanism extensions and groups already use.
  Two shapes; neither absorbs the other.

- **`ScanScope.ignore_rules_fingerprint` becomes `tag_rules_fingerprint`.** Same wire
  position, and an empty rule set still fingerprints to zero, so every existing snapshot
  stays valid.

- **Tag bits are derived, not serialized.** A snapshot carries no tag data; adopting a
  rule set re-tags the loaded index.
  That is not an optimization — the loader restores entries before the caller’s rules
  are known, so tagging only at insert would make a warm start answer “no tags” where a
  cold scan of the same tree answered correctly.

- **The `ignore` crate lands behind a default-on `gitignore` feature**, `notify`’s exact
  precedent, on measured evidence (+1.06 MiB stripped and LTO’d, nine new crates, no
  lean mode) — unpinned, because the workspace MSRV moved to 1.88 rather than holding
  the crate a release behind.
  The tag model itself stays dependency-free.

- [x] Tag model foundation: registry, tiers, entry bits computed at apply time, the
  fingerprint rename, the zero-dependency `dotfile` reference rule, tag filtering by
  re-aggregation, `--tag-rules` on the scope axis with `--tag`/`--not-tag` on selection,
  goldens and parity (`fdu-mvt3`)

  Shipped as `crates/fdu-core/src/tags.rs` plus wiring.
  Two names changed against this plan.
  The scope axis is `--tag-rules`, matching `--type-rules`, because it enables rules
  rather than naming tags; filtering is `--tag`/`--not-tag` on the Selection axis,
  repeatable and any-of, matching `--include`/`--exclude`. Keeping them one flag would
  have made `--not-tag` silently invalidate a snapshot, which is exactly the
  scope-versus-selection line this design exists to hold.
  Filtering on a rule that is not enabled is refused rather than answered with nothing:
  a mask of zero is indistinguishable from no constraint, so accepting it would hand
  back every entry to a caller who believed they had narrowed.

- [x] The gitignore rule: the feature-gated `ignore` dependency with its supply-chain
  and MSRV pins, an index-held evaluator with correct negation, `.gitignore`-edit
  escalation to `InvalidateSubtree`, watch re-tagging (`fdu-brt0`, blocked by
  `fdu-mvt3`)

- [x] Promotion: per-promoted-tag plane state through the reducer path, the partition
  property as property tests, plane reads gating the precomputed tier, dual-value
  listing rows — and the recorded subtlety that a derived complement’s `newest_mtime`
  cannot come from subtraction (`fdu-pxfz`, blocked by `fdu-mvt3`)

- [x] Surfaces: `--promote`/`--plane`, `Selection(plane=...)`,
  `ScanOptions(promote=...)`, per-plane values in Python, tagged-fixture goldens, parity
  rows, and plane-equals-all equivalence when nothing is tagged (`fdu-7rwf`, blocked by
  the two above)

  Shipped with one flag more than this plan named.
  `--plane` is Selection and `--promote` is Scope, kept apart for the reason the tag
  model already established: promotion moves the snapshot fingerprint and a selection
  flag that invalidated a cache would be the `--not-tag` mistake again.
  Three flags to reach one number is the honest price of a cache-correct model.

  The surfaces found three defects in the maintained state beneath them, all of the same
  shape — a plane read is fast because it reads state maintained elsewhere, so a wrong
  plane looks exactly like a right one.
  `ensure_dir_chain` built its placeholder’s contribution by hand with no planes, so on
  a real walk a plane’s directory count was near zero while its files and bytes were
  right; a rebind re-tagged every entry and left the planes derived from the old bits,
  which made `gitignore` — the rule planes exist for — report a plane equal to the tree;
  and an unfiltered `--view summary` was answered by a tier that retains aggregate
  tallies and no index, which returned the whole tree under the plane’s heading.
  None of the three could be seen from one tier.
  What found them was running the walking tier over the same restriction and requiring
  the two to agree, which is now `crates/fdu-core/tests/plane_equivalence.rs`.

- [x] Hidden-path admission as scope: prune hidden components except an exact-name
  allowlist, fingerprinted into snapshot identity — a scope rule, deliberately not a
  tag, and distinct from the `dotfile` tag, which filters with both numbers visible
  where this excludes from the index entirely (`fdu-xyvu`)

  Shipped as `crates/fdu-core/src/admission.rs` plus wiring, with `--hidden keep|prune`
  and `--hidden-allow LIST` on the Scope axis and `ScanOptions(hidden=, hidden_allow=)`
  in Python. The snapshot format moved to 3: `ScanScope` gained the rule’s fingerprint,
  which is positional and cannot be added without one.

  Two things about it were decided against this plan as written.

  **The admission rule and the `dotfile` tag share one predicate, deliberately.** They
  are distinguished by what they do with an entry, never by which entries they mean — a
  second definition of hidden would make `--hidden prune` and `--not-tag dotfile`
  disagree about one file, and the disagreement would read as a bug in whichever surface
  was consulted second.
  Windows’ `FILE_ATTRIBUTE_HIDDEN` bit is deliberately not read for the same reason: it
  would make the same tree admit different entries on different platforms, and the
  parity corpus would carry that as a permanent deviation.

  **The snapshot has to record where the pruned control files were.** “Governing control
  files stay readable without being retained” is satisfied during a walk by noticing the
  name, but a warm start has no walk: `Index::control_file_directories` finds
  `.gitignore` files by reading the index, and pruning is exactly what removes them from
  it. Under `CachePolicy::Only`, which is contractually forbidden to touch the tree, the
  file is the only record there is — so the walk hands its sightings to the index and
  the index writes them.
  The test that proves this had to use `Only` to say so: under `Auto` a revalidation
  re-walks and re-records them, so the section could be deleted and every assertion
  still passed.

- [x] Later fold-in: `Classification.flags` (generated, vendored, documentation) become
  Name-tier rules instead of per-query recomputation (`fdu-n7mv`, P3)

  Two of the three, and the third is the interesting one.
  `vendored` and `documentation` ship as `TagTier::Path` rules — they read the relative
  path, not just a basename, which is what that tier means — decided by a new pure-path
  matcher that needs no binding, so `needs_path` and `needs_binding` are now separate
  questions rather than one.
  The classification reports both unchanged, from the same predicate, so a caller
  filtering with `--not-tag vendored` and a row saying `vendored: true` cannot disagree
  about a file.

  `generated` cannot be a tag and the tag model’s own tier check is what says so: it
  reads the file’s opening bytes, which is `TagTier::Content`, and enabling it would
  turn a metadata walk into a content walk with no symptom but a slower scan.
  It stays on the classification of a file whose bytes were read for another reason,
  where it is free.

  The fold found the drift it was meant to prevent.
  The two copies of the stem check had already diverged: the classification’s used
  `get(..len)`, the newer one indexed, and the indexing form panics on a name whose stem
  length lands inside a multi-byte character.
  One function now, and the test names `réadme.md` for exactly that reason.

### Phase 2: The customizable taxonomy

Converts PR #38’s indexed tiers rather than competing with them; that work is merged.

- [x] A `group` level in the rule dialect and in roll-up state, so a browsing taxonomy
  is its own axis rather than a reinterpretation of the analysis families; the compiled
  default registry gains groups and the `groups` view renders them (`fdu-b2vy`)
- [x] A runtime-supplied registry: parse, validate, index, and fingerprint in Rust;
  supplied as an immutable packet at open with its expected identity echoed back and
  disagreement failing the open; accepted by `OpenConfig`,
  `ScanOptions`/`AnalysisOptions`, and the CLI scope axis; `type_rules_fingerprint`
  reads the active registry so a rule change invalidates the snapshot and sidecar
  through the path that already exists.
  PR #38’s `LazyLock` statics become a per-registry index, and its tie-break test
  generalizes to a property over any registry (`fdu-ctp5`). “Disagreement failing the
  open” is `from_manifest`’s expected-identity argument, added after the reconciliation
  review found the first pass had shipped only the supply and the echo — two
  fingerprints a caller could always have compared, and could never be prevented from
  skipping
- [x] The two extension levels: a raw logical extension per the format’s eligibility
  rule, plus canonical suffix matching for rule lookup and roll-up bucketing, with a
  test pinning that no existing bucket or type row moves (`fdu-5q6e`)
- [ ] The conformance packet vendored at a reviewed metabrowser revision, its manifest
  and hashes verified locally in CI, executed against fdu’s classifier.
  Blocked on the packet: its cases are matching-only, so they pass against a single
  extension level and would have gone green both before and after the level above
  (`fdu-gy3g`, split out of `fdu-5q6e`)
- [x] Bounded per-directory extension and filename rows with a stated remainder
  (`fdu-e2p7`)
- [ ] Loop job: what the maintained-state union costs on the ancestor-merge path — the
  `unignored` plane, browsing groups, composed subtree provenance, and non-directory
  leaf counts priced together rather than as four increments, since the reducer carries
  all of them or none.
  Measured against H86’s replacement shape on a dense real subject of at least 50,000
  entries rather than against today’s walk (`fdu-n4gn`, blocked by `fdu-mvt3` and
  `fdu-b2vy` — it cannot measure what does not exist yet)

### Phase 3: The coherent read surface

Every read a server composes into one response must observe one version, and every read
must cost what its output costs.
Neither was true when this phase was written: `children()` cloned each directory child’s
whole extension map, and nothing tied several Python calls to one clock.
This phase is now complete apart from the per-row tags, which wait on Phase 1.

- [x] Scalar paged child rows — per-child directory facts, classification identity, and
  provenance, with an explicit bound, a page cursor, and a stated remainder; the
  extension breakdown moves to its own bounded roll-up projection rather than riding
  every listing (`fdu-plwq`, with `fdu-e2p7` bounding the breakdown itself).
  Tags are the one part still outstanding, waiting on the planes in Phase 1
- [x] A bundled multi-projection read evaluated under one read guard, returning one
  engine version, the change cursor captured at the same boundary, index state, and the
  scope and registry fingerprints, so a composed response cannot straddle a commit and a
  consumer’s cache key derives from what it actually read (`fdu-2ivi`, blocked by
  `fdu-gav9`)
- [x] Per-result work counters — entries and directories visited, rows returned, lock
  wait, wall time, and the name bytes a result carries — so “no hidden O(index) pass” is
  an assertion rather than a review principle (`fdu-qgl9`). CPU time and total bytes
  across the binding are stated as absent, with the reason: the first is wall time less
  the lock wait on a read that does no I/O, and the second is something a binding can
  only estimate
- [x] Roll-up leaf counts for symlinks and special objects, so a complete subtree’s
  emptiness is decidable from the aggregate rather than by listing it; the partition
  property extends to the new field (`fdu-5hip`). No snapshot bump after all: the format
  persists kind and rebuilds roll-ups on load, so the count is derived from data already
  stored and a bump would discard every cache to gain nothing.
  The report views still cannot tell the two apart (`fdu-or38`)

### Phase 4: The embedder watch contract

- [x] Per-batch dirty roll-up set, engine through Python (`fdu-mz1a`)
- [x] `Index.refresh(path=...)` scoped reconciliation in the Python surface (`fdu-fh0k`)
- [x] Polling backend selection in `WatchOptions`, with its interval stated (`fdu-rhu3`)
- [x] The asyncio adapter and the thread-affinity documentation, with a tested
  SSE-resume example mapping `since`/`truncated` to `Last-Event-ID`/resync (`fdu-97pb`,
  blocked by `fdu-gav9`: an event-loop adapter over a surface that raises under
  concurrent access would only relocate the defect).
  The example carried the reconciliation’s verdict that this cursor was not yet complete
  for a production feed, because trust transitions did not ride the clock; `fdu-jxs0`
  put them on it, and the caveat is now a statement of what the cursor covers The
  adapter owns the affinity rule rather than documenting it — it opens, drains and
  closes the watch on the worker thread, because `PyWatch` is `unsendable` and handing
  one across panics

### Phase 5: Session integration shape

One bead, `fdu-4o0m`, blocked by the progressive-results session beads `fdu-e86o` and
`fdu-a0j0` which land the core.
Its three requirements:

- [ ] Mid-walk progress surface: entries applied, clock, completeness, with coverage
  labelled by phase and cause — it is monotone only while discovery is additive
- [ ] Async session adapter, same policy as watch — pull over a bounded queue, never a
  callback across the boundary
- [ ] Session-to-watch handoff as a stated sequence, not an outcome: capture watch
  events before or atomically with baseline discovery, accumulate them in a bounded
  native log while the walk or revalidation runs, reconcile each against observation
  expectations, publish complete and fresh only once reconciliation reaches a known
  cursor, and invalidate plus verify the affected scope on overflow.
  Cold and warm alike.
  Tested for the property that a mutation landing during the walk appears in the walk or
  in the feed, never in neither and never torn across both

Landing beside it, because subtree provenance and clocked trust are one mechanism rather
than two: an unverified-descendant count per directory yields both the composed subtree
value and a bounded event at its zero crossing, which is what keeps a revalidation sweep
from emitting millions of provenance-only entry changes into the feed it serves.

- [ ] Provenance composed through the reducer path, with the non-invertibility of those
  aggregates under deletion and revalidation given an explicit recompute path
  (`fdu-fka6`, `fdu-b1ts`)

- [x] Trust transitions on the committed delta contract, so `since()` and polling cannot
  disagree about the visible state (`fdu-jxs0`, `fdu-livs`). `AppliedDelta` carries
  `StateChange` beside `Op`; coverage, verification, the run envelope, and re-tagging
  each commit through it; and `Batch.transitions` and `ChangeSet.transitions` deliver
  them to both surfaces.
  Charged against the journal’s operation budget rather than being free, because a free
  transition is a retention bound a producer can walk past

- [x] A native walk budget as scope (`fdu-97dd`), stopping discovery at the cap and
  reporting partial coverage with reason `budget` plus a typed resource-stop issue.
  The consuming contract fingerprints the cap and its reference provider enforces it, so
  an engine without one returns a different inventory under the same fingerprint.
  The bound has to stop the walk rather than truncate an answer: a projection limit
  leaves the tree read anyway, which is the cost the cap exists to avoid.
  **Strict**: exactly the cap is retained, never one more, however many workers are
  reading -- a fingerprinted axis that admits “the cap plus whatever was in flight”
  makes the identity a claim the engine does not keep.
  A directory may therefore be listed only partly, which the partial coverage says.
  This is what finally makes `CoverageReason::Budget` reachable

- [x] Bounded, resumable flat pages (`fdu-91ru`): a required positive limit, a path
  cursor, an exact remainder paired with the continuation, selection-wide totals, and
  the caller’s version pin.
  Page until the continuation is absent and the concatenation is the whole answer, in
  order, with no repeats and no gaps -- which a truncating limit cannot give, because it
  returns a prefix and says how many it dropped with no way to ask for them

- [x] A batched scoped refresh (`fdu-nlhl`): a bounded set of observed paths, the union
  reconciled under one guard, one commit, one cursor, and per-path acceptance beside the
  counts. Iterating a single-path refresh is not equivalent -- N calls are N commits and
  N cursors, so a receipt covering them describes a range rather than a boundary

- [x] A closed invalidations-only interest mode (`fdu-vfx7`): the feed derives bounded
  dirty paths, query kinds, issues and terminal state in Rust and builds no entry row at
  all, because a consumer that re-reads on dirty never looks at them and materialising
  them costs a tag lookup and a path clone per operation.
  Plus the batch’s own cost measured across the whole boundary -- composed from its
  phases rather than taken end to end, since a wall clock around a blocking poll reports
  patience as cost

- [x] The terminal engine state on every batch and delta range (`fdu-vfx7`), captured
  inside the same guarded read as the journal slice and the cursor.
  Transitions are interval events and say what moved; this says where it ended up, which
  is the question a consumer resuming from a cursor actually has.
  A follow-up read is not equivalent and cannot be made so: the next commit can land
  between the two calls, and the index retains only its current image, so there is
  nothing to ask for the state as of a position already passed.
  Folding transitions into a consumer-side copy is the mirror the boundary forbids

- [x] Special filesystem objects excluded as a scope axis (`fdu-bjhy`): the consuming
  contract names three entry kinds and has nothing to call a socket, a FIFO or a device
  node, so a provider must exclude them rather than reclassify them -- a socket counted
  as a file makes every tally wrong by one in a way no field of the answer reveals.
  Scope rather than a projection filter, and that is the whole point: an adapter
  dropping the rows afterwards leaves them inside the roll-ups the same read returned,
  so the listing and its own header would describe different inventories.
  One predicate asked wherever a kind first becomes known -- after the `stat`, since a
  name does not say whether it belongs to a socket -- which is both walkers, both
  reconcilers, the single-path refresh, and the watcher’s apply funnel.
  Excluding is *removing*: a file replaced in place by a socket is one event on a path
  that never goes absent, so anything short of an explicit removal leaves the old row
  standing over it for as long as the index lives.
  The reference provider opens with it pruned and folds it into its scope digest, which
  is the case the axis exists for

- [x] The bounded scope a consumer opens is watchable (`fdu-7sou`, `fdu-97dd`): a
  positive depth and a positive file cap, both surviving a live watch, with no second
  watcher, no uncapped index, and no adapter-side mirror.
  The refusal was one rule over three axes, and splitting it by what each axis *is a
  property of* is the whole design: depth and the filesystem boundary belong to the
  entry an event names, so the boundary is redrawn per event; the file cap belongs to
  the whole inventory, so the index keeps it where the previous state of a path is
  already in hand. That second half also closed a gap nobody had connected to watching --
  reconciliation walks from the index and never consulted the walk’s budget, so one
  refresh turned a bounded inventory into an unbounded one.
  An out-of-scope upsert is a *removal*, since a path that crosses the boundary without
  going absent would otherwise keep its old row forever; an out-of-scope invalidation is
  dropped, because there is no subtree to reconcile.
  Directories are not counted, so a capped index keeps the shape of the tree even where
  its contents are truncated, and the refusal rides in the same commit as the coverage
  loss rather than at a later clock.
  What no rule can give, recorded rather than discovered: which files a long-lived
  capped index holds depends on the order events arrived, as which files a capped walk
  holds depends on the order it reached them

- [x] The reference embedder produces the consuming contract’s own scope-digest bytes
  (`fdu-vfyw`, in part): exactly `hidden_allowlist`, `max_depth` and `max_files`,
  compact UTF-8 JSON, bounds required rather than defaulted.
  The axes that dropped out are held as constants of the provider view and *checked*
  rather than hashed -- two indexes differing in symlink traversal or special-object
  admission really are different inventories, so ignoring a free axis would let a
  consumer cache across a change that invalidated it, while hashing an axis the consumer
  cannot name produces an identity it cannot reproduce.
  The fixture’s expected bytes come from running the consumer’s function, not from
  reading its spec twice: a recipe re-typed from prose agrees with the prose, which is
  exactly what the previous version did while agreeing with nothing.
  The cross-engine half -- one fixture *both* engines consume, covering the strict-cap
  boundary case and special-object replacement from a recorded observation stream -- is
  `fdu-kl7r` and needs the consuming repository

### Phase 6: Adoption proof

- [x] Classification identity in `children()` and files rows; registry identity readable
  from Python (`fdu-16l7`)

- [x] Walk telemetry as typed values beside report/session/watch results (`fdu-tib6`)

- [x] `TreeNode` remainder aggregates (`fdu-knyw`)

- [x] Reference embedder example under `crates/fdu-py/examples/` — boot, serve dual
  tallies, stream changes with dirty sets, resume from a cursor — plus the cross-engine
  agreement stack: the vendored conformance packet, a recorded-observation replay driven
  into both engines, and filesystem scenarios over immutable or stepwise-mutated
  fixtures (symlinks as leaves, hidden allowlist, gitignore negations).
  Running two live engines against one changing tree is not an oracle — the observation
  moments are incomparable and the dual walk perturbs what is being measured.
  Differences documented or eliminated (`fdu-vfyw`)

  Shipped as `crates/fdu-py/examples/browser_provider.py`, exercised from the smoke by
  loading the file that ships, so the tested code and the documented code are the same
  code. It carries `semantic_fingerprint` — named components, sorted by name, canonical
  JSON, SHA-256 — because fdu reports scope as several named fingerprints and a consumer
  keying on a subset caches an answer across a change that invalidated it.
  The recipe is asserted against bytes built by hand in the test rather than against a
  second call to the function: a test that compared the function to itself would accept
  any recipe, and a second implementation has to reproduce these bytes.

  The filesystem scenarios are in: symlinks as leaves (a symlink to its own parent,
  which hangs if it is followed), the hidden allowlist, and a nested `!keep.log` beating
  a broader `*.log` above it — decided by control files that pruning kept out of the
  index entirely. Exposing the fingerprint found a real gap left by `fdu-xyvu`:
  `hidden_fingerprint` reached `ScanScope` in the engine and never reached Python, so a
  consumer’s cache key could not have seen it.

  **Not done, and it needs the other repository:** the vendored conformance packet and
  the recorded-observation replay driven into *both* engines.
  This side of the contract — the semantics, the identity recipe, and fdu’s answers over
  the shared scenarios — is what a second engine now has something to be diffed against.

## Testing Strategy

Two properties are load-bearing and both are cheap to state.
Partition sums: for every enabled tag, plane plus complement equals the untagged totals,
by property test across scan, refresh, and watch mutations; and for a registry, every
group’s families and every family’s types sum to what the same selection reports, so a
custom registry cannot lose a file the way the extension view once did.
Concurrency gets the pair named in its section — a read during a write never raises and
never tears — driven by the existing thread test, which found the defect.
The streaming boundary gets two of its own, because the reason for the pull shape is a
property rather than a preference: a walk observed from Python holds the GIL a bounded
number of times per batch rather than once per entry, and a consumer that stops reading
degrades to a truncated feed rather than a stalled producer.
Both are assertions about what must *not* regress if someone later reaches for a
callback. Registry validation is tested for what it rejects as much as what it accepts:
duplicate ids, an unknown group, a tie-break ambiguity, and a manifest whose fingerprint
collides with the compiled default.
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

- What does the maintained-state union actually cost on the ancestor-merge path,
  measured against H86’s replacement shape?
  The plane, groups, composed provenance, and leaf counts are priced together by
  `fdu-n4gn`, and that measurement should choose the final representation.
  This is the one item in the contract that is neither settled nor cheap.
- Should the engine probe mount tables to choose the watch backend, or stay explicit and
  let clients own detection?
  Explicit ships first; probing is additive.
- Do prefix-scoped entry deltas ever beat invalidation plus a bounded read for expanded
  folders? The v1 change stream carries no entry rows either way; entry deltas enter the
  contract only if a live-change A/B shows lower end-to-end latency and copy cost once
  binding copies and browser convergence are counted.
- ~~Does fdu’s compiled default registry adopt the raw extension level for its own
  views, or expose it only to registry-supplied consumers?~~ Answered by building it:
  the compiled default adopts it, on listing and files rows and as the `unknown:` type
  id, while roll-up buckets and type rows stay canonical.
  It was indeed low-stakes — running the change against its own fixture gave
  byte-identical `--view types` and `--view extensions` output — and exposing the level
  only to registry-supplied consumers would have meant two answers to “what is this
  name’s extension” depending on who was asking.
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

- [The contract reconciliation](../research/research-2026-08-23-interactive-contract-reconciliation.md)
  — this spec read against metabrowser’s inventory-engine research, with the eight
  differences adjudicated and the amendment list this spec’s next revision applies
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
- PR #38, merged (`perf(content)`: roll-up ancestor walk and indexed type-rule tiers) —
  the indexed classification tiers this spec’s registry work converts from statics into
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
