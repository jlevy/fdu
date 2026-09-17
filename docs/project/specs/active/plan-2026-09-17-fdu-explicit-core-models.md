# Feature: Explicit Core Models, So Caching Never Changes Semantics

**Date:** 2026-09-17

**Author:** fdu project, with Claude assistance

**Status:** Draft. Ships in 0.1.0: the release waits for the acceptance criteria below,
and scope may shrink only by the deferrals this plan names.

## Overview

fdu answers one kind of question, which directories and files hold what, through many
routes: cold scans, warm revalidation, cache-only reads, a summary reducer, retained
indexes, watch sessions, opened roots, three surfaces, and four output formats.
Those routes exist for speed and convenience.
None of them may change what an answer says.

A review of 0.1.0 release candidate `5f2d36d` found that the metadata core already
behaves that way, while content analysis does not: the same `--analyze` request returns
different numbers depending on which requests ran before it.
The root cause is not one defect.
The concepts every route depends on (what a request is, how it is delivered and
executed, what stored state is valid for, what a measured value means, where a value
came from, and what an answer looks like) have no explicit model.
Each route and surface declares its own version imperatively, and the versions have
drifted.

This plan gives each of those concepts one explicit model in `fdu-core`, makes every
route and surface consume the models instead of re-deriving them, and enforces the
consequence with a test: **caching improves performance and never changes semantics**.

## Decision Summary

- **Two first principles**, recorded in
  [the design principles](../../architecture/fdu-design-principles.md):
  [Model Every Key Concept Explicitly, in One Place](../../architecture/fdu-design-principles.md#model-every-key-concept-explicitly-in-one-place)
  and its consequence,
  [Caching Improves Performance, Never Semantics](../../architecture/fdu-design-principles.md#caching-improves-performance-never-semantics).
- **Five core models** (request, execution plan, stored state, provenance, and answer),
  with **delivery** as the execution plan’s typed second input and the **analyzer
  registry** as the owner of what each measured value means.
  Validation, defaults, compatibility, write rules, metric definitions, and
  serialization live in them and nowhere else.
- **One invariant, enforced mechanically:** for every request and every history, a run
  returns the cold answer, the cold answer over exactly the part of the tree it
  verified, a named failure, or a labelled stale answer, and every route and surface
  returns the same kind of outcome.
- **Classification groups by name.** A path’s type and family are a function of its name
  and the type registry; content probes are reported as detection, never used to
  regroup.
- **The semantic core comes first:** stored-state identity with equality, a request that
  carries its content axis to the reader, and computed provenance close every difference
  the evidence found; the answer and execution-plan models follow.
- **Ships in 0.1.0.** The north star, a clean design covering the supported analyses and
  views, is not negotiable.
  Scope may shrink only by the [deferrals](#scope-deferrals) listed here, each of which
  costs performance or convenience but never changes an answer.

## Goals

- One explicit model per core concept, defined once in `fdu-core`, consumed by the
  command line, the Python package, one-shot reports, retained indexes, watch sessions,
  and opened roots.
- The path-independence invariant holds for every supported request, delivery, route,
  surface, and parsed machine format, and a test proves it.
- Every measured value has one definition that does not depend on what else the request
  asked for.
- Every stored tier states its identity, validates its items the same way, and answers a
  request only through the stored-state model’s compatibility rule.
- Every answer is one typed value serialized by every writer, with no writer altering
  values.

## Non-Goals

- Making any route faster.
  This plan may add reads where a cache cannot prove its answer; it records any cost it
  adds and leaves recovery to the performance plans.
- New views, analyzers, selection filters, or output formats.
- Journal-driven freshness, which
  [the FSEvents plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) owns.
- Shared or cross-machine caches.
- Changing what metadata-only answers report, except where surfaces disagree today:
  sizes default to allocated bytes and a watch defaults to the tree view on every
  surface.

## Background

### Evidence

Four read-only investigations ran against `5f2d36d`:

- a stored-state audit of the metadata and `.gitignore` control tiers;
- a stored-state audit of the content-analysis and classification tiers;
- a map of what a request and an answer are on every surface and route;
- an empirical path-independence matrix on the release build: 63 request variants, each
  run cold and after 10 different warming requests under `auto`, `read-only`, and
  `only`, after file mutations, and through both the command line and Python, about
  20,000 runs. The harness is in
  [`explorations/path-independence`](../../../../explorations/path-independence/README.md).

| Requests | Result |
| --- | --- |
| 38 metadata-only variants (every scope flag, selection filter, view, and size metric) | Identical to cold on every history, policy, mutation, and surface |
| Analysis requests warmed by a wider analyzer set | 89 of 1,890 warm cases differ: unrequested metrics appear, requested metrics change (`physical_lines` 171 to 166, `document_words` 259 to 647), `languages` switches its share metric and reorders rows, and `analysis.analyze` names the stored set |
| Warm histories followed by a file mutation | 432 of 5,670 cases differ from cold. None differs where the same history without the mutation matched cold, so per-item validity already detects every change; 36 cases show differences beyond the widening above, including totals that match neither cold answer (`document_words` 38 where both cold runs give 54) |
| Cold analysis requests with different analyzer sets | The same metric changes with the set: `--analyze code` drops a Haskell file’s line counts, and `logical_words` is 666 under `words` but 656 under `all` |
| Cross-surface | Python `open` with `--cache only` fails in 9 cases where the command line’s one-shot report answers, because one projection exists on one route |

The mixed-record cases persist.
A narrower request analyzes a changed file under its own analyzer set inside a wider
tier; the save keeps only records whose set equals the tier’s
(`content/content_cache.rs:73-76`), so the sidecar is incomplete for every analyzer set,
every `--cache only` analysis request then fails closed, and only a run whose set equals
the stored one, or `--cache refresh`, repairs it.

### Seven Concepts Without One Model

| Concept | How it is declared today | Consequence |
| --- | --- | --- |
| Request | Scope in `ScanConfig`/`ScanScope`; the analyzer set in `OpenConfig.analysis`, outside `Query` (`query/query_report.rs:353`, `lib.rs:158`); assembled separately in `cli.rs` and `fdu-py`; `validate_analysis` only in the surfaces, so a direct `prepare_report` call skips it; `validate_controls` called at seven sites, the last inside `report_in` (`query/query_report.rs:976`); the size metric parsed separately and defaulting to apparent in Rust but allocated in the command line and Python; watch defaulting to `tree` in the command line and to `files` in the native binding and `WatchOptions`; relative time windows resolved against the parse-time clock (`cli.rs:1187`) | `query::report(index, query, provenance)` cannot see the requested analyzers, so it presents whatever the index holds |
| Delivery | Cache policy on `OpenConfig` (`lib.rs:155`); format consumed in `cli.rs`; `--allow-partial` only as an exit-code mapping (`cli.rs:1745-1746`); worker counts and watch interval as command-line values | No route can enumerate the deliveries it must agree across |
| Execution plan | The one-shot planner (`execution.rs:147-182`), `open_for_report`, the watch initial report, and Python `Index` each decide what to read and write | Under `auto`, a one-shot metadata report never reads the snapshot while `open` does; the summary path writes nothing under `auto`, `off`, or `read-only`; Python refresh never writes |
| Stored state | The snapshot records an engine fingerprint and the scan scope (`snapshot.rs:201-214`, `:924`); the content sidecar records neither, and never compares analyzer versions or options (`content/content_cache.rs:88-96`, `content/content_model.rs:266-273`); entries compare six attributes, control files compare bytes, content records compare a five-field fingerprint; snapshots need exact scope equality plus one projection (`.gitignore` on to off) that only one-shot cache-only reads use (`lib.rs:377-395`); content uses containment with no projection | A sidecar survives an upgrade and is shared across scopes; the same stored state answers one route and not another; content answers carry stored rather than requested analyzers |
| Measured value | One coverage outcome per file for the whole analyzer set (`content/content_analysis.rs:302-313`); `document_words` switches meaning per record profile (`query/query_report.rs:1448-1464`, `:1577-1583`); a path’s type and family prefer a content record’s probe-based classification (`query/query_report.rs:1390`) | An unsupported `code` analyzer erases the file’s line counts; one metric has several meanings; grouping depends on history |
| Provenance | Built at six production sites (`execution.rs` twice, `cli.rs`, `watch_session.rs`, `fdu-py`, `opened/read.rs`) and a benchmark example; `scan_started_at` means three things; watch repaints hard-code `complete: true`, `errors: []`, and `warm_revalidate` (`watch_session.rs:396-404`) | A watch repaint over a partial index claims completeness |
| Answer | Four formats and the Python models rendered by independent writers with no field schema; YAML flattens metric rows and omits `root_raw`/`path_raw`; JSON Lines’ `collapse` rewrites strings (`report_format.rs:1271`); `--watch --format yaml` emits JSON change records; the schema version follows stored content (`report_format.rs:1324`) | Formats that claim one schema disagree, and one corrupts paths |

### Why Tests Did Not Catch It

- Parity replays golden sessions through a Python reimplementation of the command line
  over the public package (the parity shim), which renders through `Report.render` and
  so the same Rust writers.
  It proves both surfaces build the same request for the same route; it never compares
  routes or writers.
- 21 of 27 content-analysis goldens run with `--cache off`; the six cached ones repeat
  the `lines` set, and none serves a narrower request from a wider set.
- Content reuse unit tests assert hit counts, never answers.
- `report()` is a pure function of an index, a query, and provenance, as designed, but
  the index is a function of request history.
  Purity at that boundary never implied the property that matters.

## Design

### The Rule

A **request** determines what an answer says: root, scope, content (analyzers),
selection, views and view options, and `now`, the instant relative time windows resolve
against.
`now` is fixed when a request is constructed; a watch session fixes it at start.

**Delivery** determines how a request is carried out: cache policy, worker counts,
whether a partial result is accepted, and whether and how often the answer repeats as a
watch. Format and color are the answer model’s rendering choices.

An **answer** has content (rows, totals, metrics) and **tree status** (`complete`,
`errors`, and coverage), which describe the tree as the run found it.
**Provenance** (`source`, `freshness`, `scan_started_at`, `generated_at`, and timings)
describes the delivery.
The invariant compares content and tree status and excludes only provenance.

For request `R`, tree state `T`, delivery `D`, and any history `H` of earlier requests,
cache operations, and file changes, a run returns exactly one of:

- **Complete:** the content and tree status a cold run of `R` at `T` returns.
- **Partial:** when a cold run is partial too, or `D` cannot verify part of the tree (an
  unreadable subtree, an opened root still discovering), the answer contains exactly the
  verified part, equal to a cold answer over that part, with `complete: false` and
  errors or coverage naming what is missing.
  Retained facts under an unverified subtree are never served.
- **Failure:** an error naming why `D` cannot answer, such as no usable stored state
  under `--cache only`, or a request the delivery refuses.
- **Stale:** under `--cache only`, the content and tree status a cold run would have
  returned at a recorded earlier state, with `freshness: stale`.

For one request, delivery, and history, every route and surface returns the same kind of
outcome, and a projection between stored identities exists on every route or on none.
Every machine format parses to the same value, and text renders that value.

### Concepts and Models

| Concept | Owner | Engine architecture row |
| --- | --- | --- |
| Request | Request model | Request |
| Delivery | Execution plan model (typed input) | Delivery |
| Execution plan | Execution plan model | Execution plan |
| Stored state (identity, validity, compatibility) | Stored-state model (`serves`, `project`, write rules) | Stored-state identity; per-item validity |
| Measured value | Analyzer registry | Measured value |
| Provenance | Provenance model | Provenance |
| Answer | Answer model | Answer shape |

Each model is a typed value in `fdu-core` with its invariants as methods and tests.
Surfaces construct and render models; they never re-derive a rule a model owns.

**Request model.**
- **Shape:** composed from the existing typed parts rather than replacing them:
  `Request { scope, content, selection, views, now }`. `(scope, content)` is what a
  retained index or opened root holds for its lifetime; `(selection, views, now)` is
  what each read supplies and validates against it.
  One constructor and one `validate` cover the composite.
- **Owns:** one table of defaults; value grammars; validation with typed errors (a view
  that needs analysis, an ignored-state selection without observation, a delivery that
  cannot serve the request, limits); derived choices (the default view for an analyzer
  set, views `full` omits); and the identities other models key on.
- **Invariant:** every answer is produced from a validated request, and the reader that
  renders it receives the whole request, content included.
- **Replaces:** separate assembly in `cli.rs` and `fdu-py`, surface-only validation, the
  seven `validate_controls` sites, and per-surface defaults.

**Execution plan model.**
- **Inputs:** a validated request, a typed
  `Delivery { cache, workers, accept_partial, watch }`, and the stored state available.
  The path-independence harness iterates deliveries through this type.
- **Owns:** which tiers are needed, which stored tiers answer through the stored-state
  model, what is verified, computed, and written, and which provenance results.
- **Invariant:** one-shot reports, `open`, Python `Index`, watch sessions, and opened
  roots all obtain their plan from this model.
  Alternative plans for one request differ only in cost and provenance.
- **Replaces:** `plan_report`, the read and write decisions in `open_for_report`, the
  command line’s watch save path, and Python’s `Index` refresh and watch.

**Stored-state model.** A **tier** is a unit of identity: entries and roll-ups,
`.gitignore` control state, and content records.
A **store** is a file or in-memory container holding one or more tiers: the metadata
snapshot holds the entry and control tiers, the content sidecar holds the content tier,
and a retained index holds all three.
A store’s header records the identity of every tier it holds.
The summary path is neither: it retains nothing, so it stores nothing.

| Element | Rule |
| --- | --- |
| Identity | The engine fingerprint (crate version, formats, rules versions) plus exactly the request parts that change the tier’s values |
| Validity | One per-item fingerprint (size, allocated bytes, mtime, ctime, inode, device), plus source bytes where a tier derives from file content |
| Serves | `serves(stored, requested)`: equality by default |
| Projects | Any relation beyond equality comes with `project(stored, requested)`, which yields exactly what a cold run of the requested identity would; the invariant test proves each one |
| Writes | Per tier, by what an absent item means (below) |

| Tier | Identity beyond the engine fingerprint | Serves and projects | Written when |
| --- | --- | --- | --- |
| Entries and roll-ups | Scope | Equality | The scan completed, because an absent entry changes totals |
| `.gitignore` control state | `.gitignore` observation and limits | Equality, plus observation on to off, on every route | With its entries, for the files read |
| Content records | Scope, type registry, and each analyzer’s identity, version, and options | Equality of analyzer sets; subset projection is a [deferral](#scope-deferrals) | With any verified records; an absent record is a miss that re-reads the file |

Classification is not a tier.
A path’s type and family are a pure function of its name and the type registry, whose
identity is part of scope, computed on read.

**Measured values.** The analyzer registry owns every metric’s definition and coverage
semantics; the content tier stores results and the answer model renders them.
- Each metric belongs to exactly one analyzer, and its value for a file depends only on
  the file and that analyzer.
- Records store results and coverage per analyzer, so an unsupported `code` analyzer
  never erases `lines` results.
- A metric whose analyzer was not requested is absent: no key in JSON or YAML, `None` in
  the Python models. The field schema states that a metric is present exactly when its
  analyzer was requested.
- Content probes (shebangs, modelines, signatures) are an analyzer’s detection result,
  reported under `detection`. They decide which analyzer applies to a file and never
  change its grouping.
- `document_words` exists only when the `words` analyzer ran.
  Without it, the `documents` view has no `document_words`, and `pages` names
  `raw_words` as its source.

**Provenance model.**
- **Owns:** per tier, the source (cold scan, revalidated, cache, journal-scoped),
  freshness, and observation time; composition into the report envelope; one meaning for
  `scan_started_at`. Tree status (`complete`, `errors`, coverage) is computed by the
  same model from the index, as opened-root reads do today (`opened/read.rs:254-263`),
  and belongs to the answer.
- **Invariant:** every route computes provenance and tree status; nothing asserts them.
- **Replaces:** watch repaints’ hard-coded fields, the three `scan_started_at`
  definitions, and the absence of content freshness.

**Answer model.**
- **Owns:** one typed value per document kind (report, change record, cache status) with
  a field schema (`fdu-c5v1`); content, tree status, and provenance; an echo of the
  request identity (scope, content, size metric, views); schema versions that describe
  shape, never history.
- **Invariant:** every writer serializes the model: text renders it, JSON, JSON Lines,
  and YAML parse back to the same value, and the Python models are constructed from it.
  No writer alters a string.
  YAML uses the conformant scalar policy chosen by the YAML evaluation (`fdu-4xy9`):
  plain only when unambiguous under YAML 1.1 and 1.2, otherwise double-quoted with the
  shared escape set.
- **Replaces:** the independent writers, JSONL’s string collapse, YAML’s hand-copied
  field lists, and the native Python dict (`fdu-py/src/lib.rs:719-803`), which the
  public package never uses and which is deleted rather than migrated.

### Sessions

A **watch session** is a request plus a plan re-evaluated on each commit, with `now`
fixed at start. It serves a tier only if it keeps that tier current.
The command line already refuses `--analyze` with `--watch` (`cli.rs:647-651`); the Rust
`Session` and Python `Index.watch()` refuse an analyzed index the same way.
`--watch --cache only` is refused, because the window between the snapshot and the
session’s start is never verified.
Each repaint’s provenance and tree status come from the provenance model.

An **opened root** holds `(scope, content)` and validates each read with the request
model, so a `documents` read without content analysis is refused rather than answered
with zero words.

A **Python `Index`** holds the `(scope, content)` it was opened with.
A report on it with a different content axis is refused; with equality-serve there is
nothing to project.

### Components

- `crates/fdu-core/src/query/`: the request model (composing `Query`, `Selection`,
  `ViewSpec` resolution, validation, and defaults) and the answer model.
- `crates/fdu-core/src/execution.rs` and `lib.rs`: the execution plan model and
  `Delivery`, used by every route.
- `crates/fdu-core/src/snapshot.rs`, `cache.rs`, `content/`, and `control*`: store
  headers, tier identities, per-tier write rules, per-analyzer content records.
- `crates/fdu-core/src/classify*`: name-based grouping; probe detection as an analyzer
  result.
- `crates/fdu-core/src/watch_session.rs` and `opened/`: sessions consume the plan,
  provenance, and request models.
- `crates/fdu-core/src/report_format.rs`: writers over the answer model.
- `crates/fdu/src/cli.rs`: flag parsing into the request and delivery, rendering, and
  exit codes only.
- `crates/fdu-py/`: bindings construct requests and deliveries and expose answer models.
- `tests/`: the path-independence harness, metric-independence test, and writer-equality
  test.

### API Changes

These are breaking changes, and they land before 0.1.0 is published.

- **Rust:** a composed `Request` accepted by the report reader; a typed `Delivery`;
  typed validation errors; per-analyzer `FileAnalysis` results and coverage; store
  headers with tier identities, exposed through cache status; provenance and tree status
  separated; answer model types.
- **Python:** `report`, `open`, `scan`, `Index`, and `fdu.opened` accept the same
  request and delivery; `AnalysisMetadata.analyze` reports the requested analyzers;
  unrequested metrics are `None`; `Index.watch()` refuses an analyzed index and defaults
  to the tree view.
- **Command line:** no new flags; `--watch --cache only` becomes a usage error; cache
  status shows each store’s tier identities.
- **Schemas:** new versions of the report, stream, and cache-status schemas for absent
  unrequested metrics, the request echo, the tree-status and provenance split, and
  identities in cache status.
- **Cache formats:** new snapshot and content-sidecar formats with tier identities.
  Existing files read as stale and are removed by `--cache-clear`, as today.

## Implementation Plan

### Phase 1: The Semantic Core

- [ ] Land the path-independence harness in `tests/` with a registry of known violations
  seeded from the evidence (the warm, mutation, and cross-surface cases), which fails on
  any new difference and on any stale entry.
  Add an unreadable-subtree mutation.
  A bounded subset runs in `make check` and on every pull request; the full matrix runs
  on a schedule or by label.
- [ ] Give the content sidecar a complete identity (scope, engine fingerprint, analyzer
  versions and options) and serve content by equality of analyzer sets; apply the
  per-tier write rule; give the snapshot header both tier identities.
  Measure the registry afterwards: it should hold only the reader, provenance, writer,
  and default classes.
- [ ] Build the request model by composition; carry `content` and `now` to the reader;
  move validation and defaults into it (allocated sizes, tree view for watch); make
  opened reads and Python `Index` validate through it; refuse `--watch --cache only`,
  and refuse an analyzed index in the Rust `Session` and Python `Index.watch()`.
- [ ] Build the provenance model with the tree-status split; compute tree status and
  provenance on every route, including watch repaints; give `scan_started_at` one
  meaning; never serve retained facts under an unverified subtree.

### Phase 2: The Complete Design

- [ ] Move metric definitions and coverage semantics into the analyzer registry; store
  per-analyzer results and coverage; make unrequested metrics absent; group by name and
  report probes as detection; define `document_words` and `pages` as above.
  Add the metric-independence test.
- [ ] Build the answer model with a field schema; move every writer onto it, including
  the YAML scalar policy and JSON Lines without string rewriting; build the Python
  models from it; delete the native dict; add the writer-equality test.
- [ ] Build the execution plan model with the typed `Delivery`; route one-shot reports,
  `open_for_report`, Python `Index` refresh and watch, the command line’s watch, and
  opened roots through it.
- [ ] Apply the `.gitignore` observation projection on every route.
- [ ] Empty the known-violation registry; remove the Known Gaps sections from the
  architecture documents and the cache design.

## Scope Deferrals

Each of these may be cut from 0.1.0 to ship sooner, because each costs performance or
convenience and none changes an answer or leaves a concept without its model:

- **Stores keyed by identity.** A root keeps one snapshot and one sidecar; a request in
  another scope or analyzer set misses and rescans, which is a cost, not a different
  answer. Store headers carry tier identities now, so keying later changes file naming
  only (`fdu-w3l5`).
- **Content subset projection.** Serving fewer analyzers from a record set holding more
  arrives after per-analyzer records ship; until then a different analyzer set re-reads.
- **Equivalent scopes.** A depth bound beyond the tree’s height, or a `.gitignore`
  budget no file reaches, stays a distinct identity.
- **Live re-analysis in watch sessions.** Sessions refuse content analysis.
- **Source and freshness in human text.** Text reports errors and partial results on
  standard error and carries no source or freshness label; machine formats carry both,
  as
  [the principle](../../architecture/fdu-design-principles.md#fastest-answer-the-data-allows-never-silently-stale)
  requires.

## Testing Strategy

- **Path independence:** the harness replays requests across every scope flag, selection
  filter, view, analyzer set, and size metric; warming histories; every delivery the
  `Delivery` type enumerates; mutations including an unreadable subtree; and the command
  line, Python one-shot reports, Python `Index`, and opened reads.
  It compares parsed content and tree status with cold answers, and outcome classes
  across routes and surfaces.
- **Metric independence:** for each metric, the cold value on a fixture is identical
  under every analyzer set that includes its analyzer, and absent under every set that
  does not.
- **Writer equality:** every document kind parses to the same value from JSON, JSON
  Lines, and YAML (strict 1.1 and 1.2), and equals the Python model, over a corpus of
  adversarial names and non-UTF-8 paths where the platform allows them.
- **Identity:** an engine, format, registry, or analyzer version change invalidates
  every affected tier; a store never serves another identity except through a declared
  projection.
- **Sessions:** repaint tree status over a partial index; refusal of content and
  cache-only in sessions; opened reads refuse invalid requests.
- **Goldens:** updated deliberately, with each diff attributed to a model change.

## Rollout Plan

- Everything lands on `main` before the 0.1.0 tag; the release epic `fdu-gjc2` depends
  on this plan’s acceptance criteria through `fdu-xgjx`.
- The CHANGELOG describes the shipped models and schemas as the 0.1.0 contract, not as
  changes from an unreleased shape.
- The release rehearsal and the end-to-end verification (`fdu-tyvq`) run on the final
  commit, with the full path-independence matrix included in that verification.

## Acceptance Criteria

- The path-independence, metric-independence, and writer-equality tests pass on Linux,
  macOS, and Windows with an empty known-violation registry.
- No request field is parsed, defaulted, or validated outside the request model, and no
  route decides reads or writes outside the execution plan model.
- Every tier has an identity in its store’s header, the shared validity fingerprint,
  `serves`, and its write rule in the stored-state model.
- Every route computes tree status and provenance from the provenance model.
- The design principles carry no conformance pointer, and the architecture documents and
  cache design carry no Known Gaps entries for these models.

## Open Questions

- Does the answer echo the whole request or a request identity?
- Should a later request option allow probe-based grouping for extensionless files, as a
  distinct request rather than a history-dependent regrouping?
- How do the new public model types interact with the extensibility decision in
  `fdu-lmfd`?
- When stores are keyed by identity, how many identities does a root keep, and what does
  cache status list?

## References

- [Design principles](../../architecture/fdu-design-principles.md),
  [engine architecture](../../architecture/fdu-engine-architecture.md),
  [surface architecture](../../architecture/fdu-surface-architecture.md), and
  [cache design](../../guides/cache-design.md)
- The [path-independence harness](../../../../explorations/path-independence/README.md)
  and its 2026-09-17 summary
- Related plans:
  [file content metrics](../done/plan-2026-08-12-fdu-file-content-metrics.md),
  [cache layers and defaults](../done/plan-2026-08-15-fdu-cache-layers-and-defaults.md),
  [view vocabulary and output contract](plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md),
  [opened-root inventory engine](plan-2026-08-25-fdu-opened-root-inventory-engine.md),
  and [disk-usage checkpoints](plan-2026-09-13-fdu-disk-usage-checkpoints.md), whose
  checkpoint compatibility should follow the stored-state model
- Beads: release epic `fdu-gjc2`; model epic `fdu-h7xy`; `fdu-gija` (cache-dependent
  analysis answers), `fdu-snv3` (watch with analysis), `fdu-c2ml` (YAML contract),
  `fdu-azz3` (content tier), `fdu-fft9` (answer model), `fdu-4xy9` (YAML evaluation),
  `fdu-ky5m` and `fdu-7dj6` (per-analyzer records), `fdu-w3l5` (stores keyed by
  identity), `fdu-c5v1` (field schema), `fdu-kq8c` (code comments that contradict these
  documents)
- Content reuse by containment was introduced in `2aa7da1` (#37)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
