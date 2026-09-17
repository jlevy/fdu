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
  `Request { basis: Basis { root, scope, content }, query, now }`, where `query` carries
  the selection and views.
  `Basis` is what a retained index or opened root holds for its lifetime; `query` and
  `now` are what each read supplies and validates against it.
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
- **Inputs:** a validated request, a typed `Delivery { cache, accept_partial, watch }`,
  which gains `workers` when Phase 2 moves worker counts out of `ScanConfig` and
  `AnalysisRequest`, and the stored state available.
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
| Content records | The entry tier’s identity (so not `.gitignore` observation, which no metric depends on) and each analyzer’s identity, version, and options | Equality of analyzer sets; subset projection is a [deferral](#scope-deferrals) | With records verified under verified subtrees, when the scan completed or a snapshot of the same entry identity is already stored; an absent record is a miss that re-reads the file |

The analysis candidate set is every regular file in scope, independent of `.gitignore`
observation; an option that excluded ignored files from analysis would put observation
back into the content identity.

Classification is not a tier.
A path’s type and family are a pure function of its name and the type registry, whose
identity is part of scope, computed on read.

**Measured values.** The analyzer registry owns every metric’s definition and coverage
semantics; the content tier stores results and the answer model renders them.
- Each metric belongs to exactly one requestable analyzer unit (`lines`, `code`, or
  `words`), and its value for a file depends only on the file and that unit.
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

- `crates/fdu-core/src/query/query_request.rs` (new): the request model (`Basis`,
  `Request`, `RequestSpec`, `RequestError`), value grammars, and defaults.
- `crates/fdu-core/src/query/query_status.rs` (new): tree status and report provenance.
- `crates/fdu-core/src/stored_state.rs` (new): tier identities, `serves_snapshot`, and
  per-tier write rules, used by `snapshot.rs`, `content/`, `cache.rs`, and `lib.rs`.
- `crates/fdu-core/src/content/`: per-analyzer records, the `METRICS` table, and content
  detection.
- `crates/fdu-core/src/execution.rs` and `lib.rs`: the execution plan model and
  `Delivery`, used by every route.
- `crates/fdu-core/src/emit.rs` and `emit/` (new): the zero-dependency JSON and YAML
  sinks and scalar policy; `report_format.rs` keeps the walks, the field schema, and
  text.
- `crates/fdu-core/src/watch_session.rs` and `opened/`: sessions consume the plan,
  provenance, and request models.
- `crates/fdu/src/cli.rs`: flag parsing into the request and delivery, rendering, and
  exit codes only.
- `crates/fdu-py/`: bindings construct requests and deliveries and expose answer models.
- `tests/path_independence/` and `crates/fdu-core/tests/metric_independence.rs`: the
  path-independence harness and the metric-independence test; writer equality lives in
  `scripts/check-yaml.mjs` and the Python tests.

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
- **Schema versions:** `fdu.report/7` replaces both report schemas and describes shape
  only; `fdu.stream/2` adds `path_raw`; `fdu.cache/2` adds tier identities.
- **Cache formats:** snapshot format 5 and content-sidecar format 5 with tier
  identities, and a further sidecar format for per-analyzer records unless both land
  together. Existing files read as stale and are removed by `--cache-clear`, as today.
- **Writes:** Python `Index.refresh()` and `Index.watch()` write snapshots under `auto`,
  as the command line does; `read-only` opts out.

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
  Measure the registry afterwards: it should hold only the projection-route and
  unverified-subtree classes.
- [ ] Build the request model by composition; carry `content` and `now` to the reader;
  move validation and defaults into it (allocated sizes, tree view for watch); make
  opened reads and Python `Index` validate through it; refuse `--watch --cache only`,
  and refuse an analyzed index in the Rust `Session` and Python `Index.watch()`.
- [ ] Build the provenance model with the tree-status split; compute tree status and
  provenance on every route, including watch repaints; give `scan_started_at` one
  meaning; record every failed path during a walk; drop retained facts under a subtree a
  pass cannot verify.

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
- [ ] Apply the `.gitignore` observation projection on every route, at snapshot load.
- [ ] Empty the known-violation registry; remove the Known Gaps sections from the
  architecture documents and the cache design.

## Implementation Detail

File and function names below were verified against `5f2d36d`. Paths are relative to
`crates/fdu-core/src/` unless they start with `crates/`, `tests/`, `scripts/`,
`explorations/`, `.github/`, or `Makefile`. Each item is an epic bead whose children are
its commits, one bead per commit; the beads carry the line locators verified at
`5f2d36d`, and the plan keeps the file and function names, which survive as code moves.

### Beads

The epic `fdu-h7xy` holds the eight items, and `fdu-xgjx` closes conformance after all
of them.
Commit beads are named `P<phase>.<item>.<commit>`, and their blockers encode the
sequencing below.

| Item | Epic bead | Commit beads |
| --- | --- | --- |
| Phase 1 item 1: The path-independence harness | `fdu-pncf` | `fdu-6dwc` (P1.1.1), `fdu-d3se` (P1.1.2), `fdu-whdi` (P1.1.3), `fdu-zf0f` (P1.1.4), `fdu-u9do` (P1.1.5), `fdu-km7w` (P1.1.6) |
| Phase 1 item 2: Store identity, equality serve, and per-tier writes | `fdu-kg2i` | `fdu-nz0e` (P1.2.1), `fdu-ziu0` (P1.2.2), `fdu-7swa` (P1.2.3), `fdu-o6vm` (P1.2.4), `fdu-ss0m` (P1.2.5), `fdu-ptj8` (P1.2.6) |
| Phase 1 item 3: The request model | `fdu-vi8z` | `fdu-ve5k` (P1.3.1), `fdu-15gv` (P1.3.2), `fdu-xnjy` (P1.3.3), `fdu-sm9z` (P1.3.4), `fdu-890m` (P1.3.5), `fdu-cevv` (P1.3.6) |
| Phase 1 item 4: Provenance and tree status | `fdu-awjm` | `fdu-c7gc` (P1.4.1), `fdu-rjv3` (P1.4.2), `fdu-wuip` (P1.4.3), `fdu-8x1d` (P1.4.4), `fdu-szll` (P1.4.5), `fdu-tngk` (P1.4.6) |
| Phase 2 item 1: Measured values | `fdu-azz3` | `fdu-ogg0` (P2.1.1), `fdu-xras` (P2.1.2), `fdu-ibu1` (P2.1.3), `fdu-am8r` (P2.1.4), `fdu-ufjb` (P2.1.5) |
| Phase 2 item 2: The answer model and writers | `fdu-fft9` | `fdu-aow4` (P2.2.1), `fdu-3ex4` (P2.2.2), `fdu-5at8` (P2.2.3), `fdu-7bfu` (P2.2.4), `fdu-joqd` (P2.2.5), `fdu-lkuj` (P2.2.6), `fdu-jppg` (P2.2.7), `fdu-8ihl` (P2.2.8) |
| Phase 2 item 3: The execution plan model | `fdu-838z` | `fdu-ehk4` (P2.3.1), `fdu-a9wi` (P2.3.2), `fdu-rtfc` (P2.3.3), `fdu-sbg9` (P2.3.4), `fdu-b7df` (P2.3.5), `fdu-3uit` (P2.3.6), `fdu-xm3m` (P2.3.7), `fdu-7t60` (P2.3.8) |
| Phase 2 item 4: The `.gitignore` observation projection | `fdu-ssyf` | `fdu-ay3c` (P2.4.1), `fdu-xwmv` (P2.4.2), `fdu-qxgh` (P2.4.3), `fdu-brun` (P2.4.4), `fdu-u767` (P2.4.5) |

### Interfaces Between Items

| Type or function | Defined by | Used by |
| --- | --- | --- |
| `Basis { root: PathBuf, scope: ScanConfig, content: AnalysisSet }`, `Request { basis, query: Query, now: SystemTime }`, `RequestError` | Phase 1 item 3 | Every other item |
| `Delivery { cache, cache_path, accept_partial, watch: Option<WatchDelivery> }`, gaining `workers: Workers` in Phase 2 | The struct in Phase 1 item 3 commit 1; `workers` and planning behavior in Phase 2 item 3 | Harness, sessions, execution plan |
| `EntryScope`, `SnapshotIdentity { entries: EntryTierIdentity, controls: ControlTierIdentity }`, `ContentTierIdentity { entries, analysis, provenance: AnalyzerProvenance { options_fingerprint, analyzers } }` (the entry tier alone holds the type-rules fingerprint), `serves_snapshot` (`Exact` or `Refuse`), `entries_writable`, `content_record_writable` | Phase 1 item 2 (`stored_state.rs`); Phase 2 item 4 adds `ProjectControlsOff` | Projection, execution plan, cache status |
| `Index::content_set() -> AnalysisSet` | Phase 1 item 2 | Request validation, sessions |
| `TreeStatus { complete, coverage, errors, errors_omitted }`, `ReportProvenance { source, freshness, scan_started_at, generated_at, tiers }` | Phase 1 item 4 | Answer model, execution plan |
| Per-analyzer `FileAnalysis`, `METRICS: &[MetricDef]`, `document_words(row) -> Option<u64>`, `pages(row, words_per_page)` | Phase 2 item 1 | Answer model, content store |
| `Sink`, `JsonSink`, `YamlSink`, `emit_report`, `emit_change`, `emit_cache_status`, `Field` schema | Phase 2 item 2 | Every writer, Python models |
| `Plan`, `plan(request, delivery, route)`, `Plan::admit`, `Plan::writes`, `Plan::outcome` | Phase 2 item 3 | Every route |

### Sequencing Across Items

- Phase 1 items 1 and 2 land first; item 2’s equality serve should clear the
  content-containment and mixed-records classes, leaving projection-route and
  unverified-subtree. Removing a class from the registry needs a full Linux run, because
  only the full matrix shows a class is empty.
- Before item 2 writes snapshot format 5, it settles the `EntryScope` split and the
  pass-start header field, so the format is written once.
- The request model (item 3) lands before provenance (item 4), because `report` takes
  the request and computes tree status and provenance itself.
- In Phase 2, the answer model’s JSON Lines, JSON, and YAML walks land before measured
  values make metrics optional and bump the report schema, so absence is implemented
  once in the walk rather than in each writer.
- Per-analyzer records either share the sidecar format bump with item 2 or take the next
  one; both land before 0.1.0.
- Phase 2 item 1 commit 4 waits on Phase 2 item 2 commits 3 to 5, which is why every
  item is split into one bead per commit and the blockers sit on the commits (P2.1.4
  waits on P2.2.5).
- The `.gitignore` projection needs item 2’s snapshot identity and item 3’s refusal of
  `--watch --cache only`; if the execution plan lands first, `Plan::admit` calls
  `serves_snapshot`.

### Phase 1, Item 1: The Path-Independence Harness

The harness is Python `unittest` using only the standard library, run through uv’s
Python 3.12 as `release-test` is (`Makefile`). The invariant spans the command line and
the Python package, which a Rust test cannot reach, and tryscript compares bytes rather
than parsed answers.
The case against is speed (one subprocess per case) and the installed wheel the Python
half needs; model unit tests stay in Rust.

| File | Function or type | Change |
| --- | --- | --- |
| `tests/path_independence/fixture.py` | `build_fixture(root) -> FixtureFacts` | Port `explorations/path-independence/make_fixture.sh`: seeded bytes instead of `/dev/urandom`, `os.utime` instead of `touch -t`, symlinks skipped and recorded where `os.symlink` fails |
| `tests/path_independence/matrix.py` | `REQUESTS`, `WARMERS`, `MUTATIONS`, `ROUTES`, `SUBSET` | Move the exploration’s `R`, `W`, `mutate`, and `MUTATIONS`; add the `unreadable` mutation (`chmod 000` on `src/nested`, restored in `finally`, skipped where permission bits are not enforced); routes `cli-report`, `py-report`, `py-open`, `py-scan`, and later `cli-watch-initial` |
| `tests/path_independence/runner.py` | `run_cli`, `run_py`, `normalize`, `compare -> Verdict`, `case_key` | Move the exploration’s runner; `normalize` drops only `source`, `freshness`, `scan_started_at`, and `generated_at`; verdicts are `same`, `stale`, and `refused` (the Rule’s allowed outcomes), `differs`, or `outcome_class`; lists of objects align by `path`, `id`, or `name`, and the answer’s root is a placeholder |
| `tests/path_independence/pyrun.py` | `main` | Move; refuse to run when `fdu` imports from `crates/fdu-py/python`, the parity safety property |
| `tests/path_independence/registry.py` | `load`, `verify`, `record`, `merge` | Add; `merge` combines judged runs from several platforms |
| `tests/path_independence/known-violations.toml` | registry | Add, merged from the full matrix’s judged cases on Linux, macOS, and Windows |
| `tests/path_independence/test_harness.py` | comparator and registry unit tests | Add; needs no fdu build |
| `tests/path_independence/test_path_independence.py` | `PathIndependence.test_matrix` | Add; `FDU_PI_TIER=subset\|full`, `FDU_PI_SURFACES=cli\|cli,python` |
| `Makefile` | `path-independence`, `test-path-independence`, `path-independence-full`, `path-independence-record`; `check`, `UV_BACKED_TARGETS`, `PYTHON_LINT_PATHS` | Add the targets; `check` runs the subset after `parity-check` with `FDU_PYTHON=$(SMOKE_PYTHON)` |
| `.github/workflows/ci.yml` | `test` job, `parity` job | Add the pinned `setup-uv` step to the `test` job, which has none, and run the command-line subset on three platforms through the Make target, so the harness runs on uv’s Python 3.12 (`tomllib` needs 3.11); the two-surface subset runs in the parity job with `.venv-parity` |
| `.github/workflows/path-independence.yml` | full matrix | Add: schedule, `workflow_dispatch`, and the `path-independence-full` label; three platforms; failing diffs uploaded; toolchain and uv pins inventoried in `supply-chain-policy.json` |
| `explorations/path-independence/` | scripts | Delete once moved; keep `results/` and a README pointing to `tests/path_independence` |

`FDU_BIN` is an absolute path, defaulting to `target/debug/fdu` from `make build` and
never resolved through `PATH`. Each invocation gets its own `XDG_CACHE_HOME`, which
`user_cache_dir` honors on every platform (`lib.rs`).

The registry is TOML, read with `tomllib` and reviewed like a golden:

```toml
[classes.content-containment]
description = "An analysis request answered from stored records of another analyzer set"
clears_with = "Phase 1 item 2: content identity and equality serve"
bead = "fdu-gija"

[[violation]]
class = "content-containment"
paths = ["analysis.analyze[]", "reports[].metrics.total.metrics.physical_lines"]
keys = ["warm/cli-report/auto/W_all/-/a_lines"]
```

A run fails on an unregistered difference, a registered key whose generalized paths
changed, a registered key that now matches cold and the other routes, a class with no
entries, an entry still marked `unclassified`, or a run with zero cases or zero
parseable cold answers.
An optional `platforms` field records an entry that occurs, or takes its shape, only on
some platforms, which is itself a finding to explain; one case may carry a different
shape per platform. Entries sharing a class, shape, and platforms are grouped under one
`keys` list.

The subset is 17 requests (`default`, `nogi`, `budget1k`, `scandepth1`, `exclign`,
`onlyign`, `v_summary`, `v_summary_nogi`, `v_types`, `a_lines`, `a_code`, `a_words`,
`a_all`, `a_lines_v_documents`, `a_code_langs_name_lim1`, `a_all_nogi`, `onefs`) across
five warmers and three policies on `cli-report`, seven mutations (one per way a change
is detected) after two warmers, and the Python routes after two warmers: about 1,600
cases, budgeted at 90 seconds on Linux and 4 minutes on Windows, where process creation
is slower. The full matrix is about 20,000 invocations, budgeted at 30 minutes per
platform.

The seed classes each name the item that clears it: `content-containment` and
`mixed-records` (item 2), `unverified-subtree` (item 4), `refusal-kind` and
`refusal-order` (item 3), and `projection-route` (Phase 2 item 4). `windows-change-time`
records that the validity fingerprint reads no change time on Windows, so a same-size
rewrite that keeps its mtime goes unseen there; `fdu-6act` decides whether to read the
NTFS change time.

**Commits:**
1. Move the fixture, comparator, and matrix with `test_harness.py`; add lint paths.
2. Add `registry.py`, its tests, and `--record`.
3. Seed the registry from the full matrix’s judged runs on all three platforms; add the
   Make targets and the subset in `make check`.
4. Add the `unreadable` mutation and its entries.
5. Add the CI steps and the full-matrix workflow with its supply-chain inventory.
6. Retire the exploration scripts and update links.

**Risks:** the Python routes use a release wheel while the command line uses a debug
build, so a stale `.venv-parity` goes undetected, as parity already accepts;
`samesize_keepmtime` may differ on Windows, where ctime is creation time; a third
workflow must pass `validateWorkflowSecurity` in `scripts/check-supply-chain.mjs`.

### Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes

| File | Function or type | Change |
| --- | --- | --- |
| `stored_state.rs` (new) | `EntryScope` (today’s `ScanScope` without `type_rules_fingerprint`, `reducers_fingerprint`, and `ignore_rules_fingerprint`), `serves_snapshot(stored, wanted) -> Serves::{Exact, Refuse}`, `EntryTierIdentity { engine, scope: EntryScope, type_rules_fingerprint, reducers_fingerprint }`, `ControlTierIdentity::{NotObserved, Observed { limits }}`, `SnapshotIdentity { entries, controls }`, `ContentTierIdentity { entries, analysis, provenance: AnalyzerProvenance }` with `serves` as equality, where the entry tier alone holds the type-rules fingerprint and a record’s `ContentProvenance` is those rules plus the `AnalyzerProvenance`, `entries_writable(&Index)`, `content_record_writable(&Index, &Path, &FileAnalysis)`, and shared fixed-width codecs | Add |
| `snapshot.rs` | `FORMAT_VERSION`; `save`; `put_scope`/`read_scope`; `read_controls` limits; `parse_header_fields`; `parse_stream` | Format 5 with header identities and the writing pass’s start (`writing_pass_started_at_ns`), which `save` writes; today’s `captured_at_ns` is the file’s modification time (`snapshot.rs`); the save guard becomes `entries_writable`; control limits move into the header and a disagreeing table is refused. `identify_prologue` keeps its offsets, so format 4 reads as `OlderFormat` |
| `content/content_cache.rs` | `FORMAT_VERSION`; `save_content_cache`; `load_content_cache`; `parse` | Format 5 with the engine fingerprint and `ContentTierIdentity`; the save filter becomes `content_record_writable`; loading compares identity by equality |
| `content/content_cache.rs` | `identify_sidecar(path)` | Add, mirroring `snapshot::identify`; `content_sidecar_bytes` stays magic-only so older sidecars are still reclaimed |
| `engine_contract.rs`, `scan.rs` | `ScanScope` (`engine_contract.rs`), `observes_controls()`, `ScanConfig::scope()` (`scan.rs`) | `ScanConfig::scope()` builds `EntryScope` and `ControlTierIdentity`; observation is read from `ControlTierIdentity` at `execution.rs`, `watch_session.rs`, `query/query_report.rs`, and `crates/fdu-py/src/lib.rs` |
| `content/content_model.rs` | `ContentProvenance::satisfies`, `AnalysisSet::contains` | Delete |
| `content/content_index.rs` | `ContentIndex`, `profile`/`provenance`, `prepare`, `commit` | Hold `identity: Option<ContentTierIdentity>`; `prepare` clears on any inequality; `commit` refuses a record of another identity instead of calling `prepare` |
| `index.rs` | `prepare_content_analysis`, `pending_analysis_candidates`, `apply_analysis` | Build the identity from the index’s scope and registry; pending compares fingerprint and identity; `apply_analysis` returns `Stale` when `commit` refuses; add `content_set()` |
| `lib.rs` | `load_content`, cache-only check, `SaveTargets`, `cold_scan_save_targets_with`, `spawn_save` | Pass the content identity; keep the cache-only count, which now means complete; drop the joint completeness gate for per-tier rules; write the sidecar after a partial scan only when a snapshot of the same entry identity is already stored, so a partial run under another identity never evicts the sidecar that pairs with the stored snapshot |
| `cache.rs` | `SnapshotInfo`, `CacheStatus`, `status_at` | `SnapshotInfo.identity`; `CacheStatus.content` from `identify_sidecar`; pairing, `clear_cache`, and the `OrphanedContent` rule stay magic-based |
| `report_format.rs`, `crates/fdu-py/src/lib.rs`, `crates/fdu-py/python/fdu/_models.py` | `CACHE_SCHEMA`, `render_cache_status`; `cache_status_dict`; `CacheStatus` model | `fdu.cache/2` with identities, coordinated with the answer model |
| Documentation | `fdu.cache/2` and formats 5 in `docs/project/guides/cache-design.md`, `docs/project/architecture/fdu-surface-architecture.md`, `docs/project/architecture/fdu-engine-architecture.md`, `docs/project/release-notes/0.1.0.md`, and `docs/project/guides/release-process.md` | Update |

**Call sites:** `SnapshotInfo` constructions at `snapshot.rs`, `cache.rs`,
`report_format.rs`, and its readers at `crates/fdu-py/src/lib.rs`; `satisfies` at
`content/content_cache.rs`, `content/content_index.rs`, `index.rs`; `save_content_cache`
at `lib.rs`; `load_content_cache` at `lib.rs`; `analyze_index`, whose behavior changes,
at `lib.rs`, `crates/fdu-py/src/lib.rs`, `examples/perf_probe.rs`, and the
`analyzed_index` and `containment_fixture` test helpers in `content/content_cache.rs`.

**Tests:**
- `content/content_cache.rs`: turn the containment tests into
  `a_wider_sidecar_is_a_clean_miss_for_a_narrower_request`,
  `another_analyzer_set_is_a_clean_miss`, and
  `a_different_analyzer_set_replaces_the_sidecar`; recheck `corruption_is_a_clean_miss`
  offsets; add `a_sidecar_from_another_engine_or_scope_is_a_clean_miss`,
  `an_analyzer_version_change_invalidates_records`, and
  `records_under_an_unverified_subtree_are_not_written`.
- `content/content_model.rs`: delete
  `containment_is_reflexive_and_ordered_by_membership`. `content/content_index.rs`: add
  `prepare_clears_on_any_identity_change` and
  `commit_refuses_a_record_of_another_identity`.
- `snapshot.rs`: update `semantic_scan_scope_round_trips` and
  `control_limits_that_disagree_with_the_scope_are_refused_at_save_and_load`; add
  `a_v4_snapshot_is_older_format` and
  `controls_on_and_off_snapshots_share_the_entry_identity`.
- `lib.rs`: add `a_partial_scan_writes_verified_content_but_no_snapshot`; recheck
  `content_sidecar_skips_unchanged_reads_and_serves_cache_only` and
  `cache_only_analysis_fails_closed_without_its_sidecar`. `cache.rs`: add
  `a_stale_sidecar_beside_a_current_snapshot_is_labelled_and_cleared`.
- Goldens: `cli-lifecycle` cache status in JSON and YAML and the stale listing change
  for `fdu.cache/2`; `cli-surface`’s `--docs` text changes; `cli-cache` and
  `cli-content` should not change.
  The parity artifact is re-recorded by CI.

**Commits:**
1. `stored_state.rs` types, `EntryScope`, `serves_snapshot` with `Exact` and `Refuse`,
   and codecs, with unit tests.
2. Snapshot format 5, including `writing_pass_started_at_ns`.
3. Sidecar format 5 and `identify_sidecar`.
4. Equality serve; delete `satisfies` and `contains`; clear two registry classes after a
   full Linux run.
5. Per-tier write rules.
6. Cache status identities, bindings, and goldens.

**Risks:** alternating analyzer sets re-read files and replace the sidecar, giving back
the gain #37 measured until subset projection lands; two sidecar format bumps if
per-analyzer records land separately.

### Phase 1, Item 3: The Request Model

The request model lives in `query/query_request.rs`, re-exported from `query.rs`:

```rust
pub struct Basis { pub root: PathBuf, pub scope: ScanConfig, pub content: AnalysisSet }
pub struct Delivery { pub cache: CachePolicy, pub cache_path: Option<PathBuf>, pub accept_partial: bool, pub watch: Option<WatchDelivery> }
pub struct Request { pub basis: Basis, pub query: Query, pub now: SystemTime }
pub struct RequestSpec<'a> { /* surface-neutral raw values, Option<&'a str> per axis */ }
pub enum RequestError { InvalidValue { axis: &'static str, value: String, expected: String },
  ViewNeedsContent(ViewSpec), IgnoredWithoutObservation(IgnoredEntries),
  ContentMismatch { held: AnalysisSet, requested: AnalysisSet },
  WatchScope, WatchContent, WatchCacheOnly, ViewLimit { attempted: usize, limit: usize } }
impl Request {
  pub fn build(spec: &RequestSpec, now: SystemTime, axes: AxisNames) -> Result<Self, RequestError>;
  pub fn validate(&self) -> Result<(), RequestError>;
  pub fn validate_read(&self, held: &Basis) -> Result<(), RequestError>;
  pub fn validate_delivery(&self, delivery: &Delivery) -> Result<(), RequestError>;
}
```

`build` resolves relative time windows against `now`, so `Selection.modified` is
absolute; a watch builds its request once at session start.
Typed command-line values (for example `--scan-depth`) reach `RequestSpec` through their
`Display`, so a mismatch there shows up as a golden difference.

| Axis | Default | Where it differs today |
| --- | --- | --- |
| Size | Allocated | `SizeMetric` defaults to apparent (`query/query_selection.rs`); the opened-root binding uses `"apparent"` (`crates/fdu-py/src/opened_binding.rs`) |
| Views (report) | `ViewSpec::default_for(content)` | None |
| Views (watch) | `tree`, the default for no content | Python passes `Files` (`crates/fdu-py/src/lib.rs`); `WatchOptions` defaults to `files` (`crates/fdu-py/python/fdu/_models.py`) |
| `words_per_page` | 250 | Declared at `crates/fdu/src/cli.rs`, `query/query_report.rs`, and the binding signatures |
| Analysis and controls | None; `read_controls` on; `ControlLimits::default()` | None |

| File | Function or type | Change |
| --- | --- | --- |
| `query/query_request.rs` | new | The types above, and the value grammars moved in from both surfaces: `parse_kind` (`cli.rs`, `crates/fdu-py/src/lib.rs`), `parse_bound`, `parse_sort`, `parse_size_metric`, `bound_nanos`, `parse_cache_policy` |
| `query/query_report.rs` | `Query::validate_analysis`, `validate_controls`, `AxisNames`, `report`, `report_in`, share metric in `metric_summary` | Delete both validators into `Request`; extend `AxisNames`; `report` and `report_in` take `&Request`, validate the read against the index’s basis, and read `analysis` and the share metric from the request |
| `query/query_selection.rs` | `SizeMetric` default | Allocated |
| `execution.rs` | `prepare_report`, `prepare_report_with_scan_diagnostics`, `prepare_report_internal` | Take `(&Request, &Delivery)`, the root being in `Basis`; build today’s `OpenConfig` from them internally until Phase 2 item 3 deletes it; delete its `validate_controls` call |
| `watch_session.rs` | `Session::new`, `query()` | `new(handle, Request, WatchConfig)`; refuse when `content_set()` is not empty and run the delivery checks; `query()` becomes `request()` |
| `scan.rs` | `validate_for_watch_scope` | Keep scope equality for its callers; the depth and one-filesystem rule becomes `RequestError::WatchScope` |
| `engine_contract.rs` | `ReportRequest`, `Error` | A read spec whose `now` is `generated_at`; add `Error::InvalidRequest(RequestError)` |
| `opened/read.rs` | `report_projection`, `validate_report` | Validate reads against `Basis { content: NONE }`, so `documents` is refused |
| `crates/fdu/src/cli.rs` | `run`, `scan_config`, `parse_query`, `parse_analysis`, `resolved_query`, `resolve_views`, `run_watch` | `Cli::spec()` and `Request::build(.., SystemTime::now(), AxisNames::FLAGS)`; delete its `validate_analysis` and `validate_controls` calls and its two watch guards; add the `--watch --cache only` refusal |
| `crates/fdu-py/src/lib.rs` | `PyIndex`, `build_query_at`, `build_report`, `watch`, `report_once`, `open`, `scan`, `to_py_err` | Hold `basis`; `build_request(now, basis, spec)`; delete the validation in `watch`, `build_report`, `report_once`, and `build_query_at`; `watch` refuses an analyzed or cache-only index; map `InvalidRequest` to `ValueError` |
| `crates/fdu-py/src/opened_binding.rs`, `crates/fdu-py/python/fdu/_models.py` | `parse_selection`, `parse_report`; `WatchOptions.query` | Defaults from the model; `WatchOptions` defaults to `Query()` |

**Call sites:** `query::report` at `execution.rs`, `watch_session.rs`,
`crates/fdu-py/src/lib.rs`, `examples/perf_probe.rs`, and tests in
`query/query_report.rs`, `report_format.rs`, and `content/content_analysis.rs`;
`report_in` at `opened/read.rs`; `Session::new` at `crates/fdu/src/cli.rs`,
`crates/fdu-py/src/lib.rs`, `crates/fdu-core/tests/watch_session_integration.rs`;
`prepare_report*` at `crates/fdu/src/cli.rs`, `crates/fdu-py/src/lib.rs`, three calls in
`examples/perf_probe.rs`, and 17 in `execution.rs` tests; `ReportRequest` at
`opened.rs`, `opened/golden_tests.rs`, `crates/fdu-py/src/opened_binding.rs`; the seven
`validate_controls` and two `validate_analysis` sites.

**Tests:** in `query/query_request.rs`, the defaults table, every `RequestError` with
flag and field wording, `now` resolution against a fixed instant, the watch refusals,
and `ContentMismatch`; in `watch_session_integration.rs`, refusal of an analyzed index;
move `language_grouping_is_metadata_only_while_documents_require_analysis` and the
refusal half of
`an_index_that_observed_no_control_state_has_no_ignored_share_to_select_by` from
`query/query_report.rs` into the module; repoint the grammar tests in
`crates/fdu/src/cli.rs`;
`a_report_that_reads_no_gitignore_refuses_to_select_by_ignored_state` in `execution.rs`
expects `InvalidRequest(IgnoredWithoutObservation)`. Goldens: add
`fdu --watch --cache only .` near `cli-surface.tryscript.md`; regenerate
`opened-root/coherent-projections-and-continuations.golden`. Python: assert
`WatchOptions().query == Query()`, and add refusals for `Index.watch()` and an opened
`documents` read.

**Commits:**
1. The model, including the `Delivery` struct without `workers`, grammars, defaults, and
   `validate`, with unit tests.
2. Allocated as the size default; regenerate the opened-root golden.
3. `report`, `report_in`, and `prepare_report*` take `&Request`; migrate every caller
   and delete the old validators.
4. The command line and the binding build through `RequestSpec`.
5. `validate_delivery`: the watch refusals and the tree default.
6. Opened reads validate their read spec.

**Risks:** the allocated default reaches opened-root selection through `EntrySelection`,
which MetaBrowser sees; `RequestError` must reproduce each surface’s current wording,
which goldens and the parity label class check; `ScanConfig` still holds delivery fields
(`threads`, `batch_size`, `order`), and `ScanConfig.threads` stays the authoritative
scan worker count until Phase 2 item 3 moves those fields into `Delivery.workers`.

### Phase 1, Item 4: Provenance and Tree Status

```rust
pub struct TreeStatus { pub complete: bool, pub coverage: Coverage, pub errors: Vec<String>, pub errors_omitted: u64 }
pub struct ReportProvenance { pub source: ReportSource, pub freshness: Freshness,
  pub scan_started_at: Option<SystemTime>, pub generated_at: SystemTime, pub tiers: TierProvenance }
pub struct TierProvenance { pub entries: TierState, pub content: Option<TierState> }
pub struct TierState { pub source: Source, pub freshness: Freshness, pub observed_at_ns: i64 }
impl TreeStatus { pub fn of(index: &Index, request: &Request) -> Self; pub(crate) fn of_walk(scan: &ScanReport) -> Self }
impl ReportProvenance { pub fn of(index: &Index, generated_at: SystemTime) -> Self;
  pub(crate) fn of_walk(started: SystemTime, generated_at: SystemTime, complete: bool) -> Self }
```

The types live in `query/query_status.rs`; the report type is renamed from `Provenance`,
which also names the per-entry type at the crate root (`lib.rs`).
`report(index, request, generated_at)` computes both, so no caller builds provenance.
`scan_started_at` means the start of the oldest verification pass whose facts the answer
serves: for a stale answer, the pass that wrote the snapshot, read from the format-5
header’s `writing_pass_started_at_ns`. That field is the start of the pass that last
wrote the image.
A later pass that verifies the same facts keeps the image and its stamp,
so the value is a lower bound on when the facts were last verified, and P1.4.4 advances
it on completed passes.
P1.4.4 also passes the execution instant down, so one pass-start instant exists, and
pins the stamp to an instant taken before the walk.

| File | Function or type | Change |
| --- | --- | --- |
| `query/query_report.rs` | `Provenance`, `Report`, `report_in`, `report_summary` | Split into `status` and `provenance`; compute both; stop reading `index.freshness()` directly |
| `scan.rs` | `consolidate_detached_index`; error branches in `reconcile_target_inner` | `index.record_walk(&errors, started_at)` marks each failed path `Partial` and retains its issue; a pass that cannot list a directory removes its retained descendants through `remove_known_children` |
| `index.rs` | `set_initial_freshness`, `begin_reconcile`, `finish_reconcile`, `IndexState.source` (set at `snapshot.rs`) | Mark failed paths rather than the pass root; stamp `writing_pass_started_at_ns`; producers keep `source` current (`Scanned`, `Revalidated`, `Cached`) |
| `watch_session.rs` | `live_provenance`, `report` | Delete `live_provenance`; `report(generated_at)` |
| `opened/read.rs` | Provenance construction in `report_projection` | Replaced by `TreeStatus::of` and `ReportProvenance::of` |
| `execution.rs` | Both provenance constructions in `prepare_report_internal` | Delete; the reader computes both |
| `crates/fdu/src/cli.rs` | Provenance construction in `run_watch` and `render_live`; `run` | Delete construction; read `report.status` |
| `crates/fdu-py/src/lib.rs` | `PyIndex` fields `errors`, `operation_complete`, `scan_started_at`, `source`, set in `open`, `scan`, `refresh`; `status_dict`; getters; `build_report` | Delete the fields; compute from the index |
| `crates/fdu-py/python/fdu/_api.py` | `Index.report` | Delete the errors override |
| `report_format.rs` | JSON envelope, YAML | Read the split fields with the same output |

**Call sites:** the six production constructions (`execution.rs`;
`crates/fdu/src/cli.rs`; `watch_session.rs`; `crates/fdu-py/src/lib.rs`;
`opened/read.rs`), `examples/perf_probe.rs`, test constructions in
`query/query_report.rs`, `report_format.rs`, `content/content_analysis.rs`, and
`execution.rs`, and `live_provenance` callers at `crates/fdu/src/cli.rs`,
`crates/fdu-py/src/lib.rs`, and `watch_session_integration.rs`.

**Tests:** `TreeStatus::of` names each failed path on a partial one-shot index; a watch
repaint over an unreadable subtree reports `complete: false`; cold and warm answers are
equal with an unreadable subtree; cache-only `scan_started_at` is the writing pass’s
start; content freshness is stale under cache-only; a tree with more than 64 failed
paths gives equal cold and warm `errors` and `errors_omitted`. Update
`reconciling_an_unreadable_control_file_root_keeps_its_rules_and_stays_partial` and
`partial_shared_pending_reconciliation_settles_instead_of_retrying` in `scan.rs` for
dropped descendants, review
`one_sweep_reports_one_as_of_time_for_everything_it_verified` and
`withdrawn_trust_beats_a_verification_interval` in `index.rs`, and regenerate the
opened-root golden. Envelopes are unchanged, so machine goldens should not change.

**Commits:**
1. Split the `Report` fields; writers emit the same bytes.
2. Record walk failures per path and add `TreeStatus::of`.
3. Producers maintain `IndexState.source`; add `ReportProvenance::of`; delete
   `live_provenance` and the Python fields.
4. One `scan_started_at`, reading `writing_pass_started_at_ns` from the snapshot header
   item 2 writes.
5. Reconciliation drops facts under unverified directories; clear `unverified-subtree`.
6. Per-tier content provenance.

**Order of errors:** `errors` holds the 64 failures smallest by path and
`errors_omitted` counts the rest.
One-shot scan errors are already sorted (`scan.rs`), while retained issues are capped at
insertion in walk order (`index.rs`), so `record_walk` keeps retained failures in path
order regardless of walk order.

**Risks:** issues are capped at 64 while scan errors are not, so `errors` becomes
bounded with `errors_omitted`; dropping descendants matches a cold walk and keeps
roll-ups exact, but an opened root loses last-known children it could have shown as
unknown, and a transient permission error forces a re-walk.

### Phase 2, Item 1: Measured Values

The engine currently loses measured work in two ways, both explained by the evidence:
- The `Unsupported` gate (`content/content_analysis.rs`) returns
  `MetricValues::default()` for a Haskell file under `code` (no arm in
  `content/content_code_metrics.rs`), which accounts for every loss in the matrix: 5
  physical lines, 14 raw words, and the 666-to-656 logical-word difference, because
  logical words are derived after summing.
- Paragraphs are counted only when `collect_logical` is on
  (`content/content_basic_metrics.rs`), which only `words` enables
  (`content/content_analysis.rs`), so `lines` and `code` report a false zero; code
  files’ paragraphs are zeroed (`content/content_analysis.rs`), and Markdown paragraphs
  are overwritten by the prose analyzer (`content/content_analysis.rs`).

| File | Function or type | Change |
| --- | --- | --- |
| `content/content_model.rs` | `MetricSlotId` (unused internally but re-exported from `content.rs`, so its removal is public) | Replace with `MetricDef { name, owner: AnalysisSet, analyzer: AnalyzerId, doc }` and `METRICS`: `lines` owns `physical_lines`, `blank_lines`, `nonblank_lines`, `raw_words`; `code` owns `code_lines`, `comment_lines`, `code_blank_lines`; `words` owns `logical_words`, `paragraphs`, `visible_words`, `visible_logical_words`, `document_words` |
| `content/content_model.rs` | `MetricValues`, `FileAnalysis` | `BasicMetrics`, `CodeMetrics`, `WordMetrics`; `FileAnalysis { fingerprint, bytes, detection: ContentDetection, lines: AnalyzerOutcome<BasicMetrics>, code: Option<AnalyzerOutcome<CodeMetrics>>, words: Option<AnalyzerOutcome<WordMetrics>>, error }`; drop `classification`, `profile`, `provenance`; add `document_word_stats` |
| `content/content_analysis.rs` | `analyze_open_file`, `record`, `analyzed_record`, `io_record`, `count_coverage`, `AnalysisReport` | Remove the early return; unsupported code becomes a code outcome while lines and words continue; paragraphs come only from `words`; probe results go into `detection`; file-level reasons apply to every requested unit |
| `content/content_index.rs` | `MetricTally`, `ContentRollUp`, `commit`, `prepare` | Tally per unit; delete `by_type` and `by_family`, which nothing reads; a public API removal, since `content.rs` re-exports `ContentRollUp` |
| `content/content_cache.rs` | `put_record`, `parse`, `put_metrics`/`read_metrics`, classification codecs | Write detection and one block per requested unit |
| `index.rs` | `pending_analysis_candidates`, `apply_analysis` | Drop `record.profile`; compare against the name-only classification |
| `query/query_report.rs` | `metric_summary`, `MetricRow`, `document_words`, `share_value`, `ShareMetric` | Group by `index.classify` only (stop preferring the cached record’s classification); aggregate per unit; the share metric comes from the request (`CodeLines` only with `code`; `DocumentWords` only with `words`, otherwise the new `RawWords`); `document_words` and `pages` return `Option` and name their source |
| `examples/perf_probe.rs` | `attach_content_summary` | Rebuild the digest per unit and version it as `fdu-content-summary-v2` |

**Call sites:** `lib.rs` and tests at `lib.rs` and `index.rs`; content tests in
`content/content_index.rs`, `content/content_cache.rs`, and
`content/content_analysis.rs`; the text, JSON, and YAML writers; `query.rs` and
`content.rs` re-exports; `crates/fdu-py/python/fdu/_models.py` `MetricValues` and
`_metric_row`; `scripts/content-selfcheck.mjs`.

**Tests:** a new `crates/fdu-core/tests/metric_independence.rs` over Rust, Python,
Haskell, Markdown, `.txt`, a C++ `.h`, an extensionless shebang script, an extensionless
`%PDF` file, invalid UTF-8, a NUL file, and a generated marker: for every `METRICS`
entry on every row and total, the cold value is identical under every analyzer set
including its owner and absent under every set that does not, and grouping is identical
under every set. Update
`code_profile_partitions_supported_languages_and_marks_others_unsupported`
(`content/content_analysis.rs`),
`deep_detection_drives_named_consumers_and_report_evidence`, and
`code_carries_word_volume_but_never_paragraphs`,
`document_profile_uses_visible_markdown_and_logical_plain_text`, and
`content_reports_preserve_empty_profiles_and_unavailable_shares`. Goldens in
`cli-content.tryscript.md` change where values were erased or files regroup.

**Commits:**
1. The `METRICS` table and a test that every emitted metric key appears in it once.
2. Per-analyzer records and sidecar layout, read back into today’s wire shape.
3. Name-based grouping and content detection counts.
4. Presence follows the request under `fdu.report/7`, after the answer model’s walks.
5. The metric-independence test.

**Risks:** `paragraphs` is owned by the `words` unit, which covers two analyzers; if
`words` is ever split, the metric splits too.
Per-unit coverage maps break readers of a flat `coverage.analyzed`. Regrouping is
user-visible: extensionless scripts leave `languages`, and C++ headers named `.h` group
as C. The content digest in the performance ledger changes at this commit.

### Phase 2, Item 2: The Answer Model and Writers

| File | Function or type | Change |
| --- | --- | --- |
| `emit.rs`, `emit/emit_json.rs`, `emit/emit_yaml.rs`, `emit/emit_scalar.rs` (new) | `trait Sink { begin_map(Shape), end_map, begin_seq(Shape), end_seq, key(&'static str), str, u64, i64, bool, null }` with `enum Shape { Block, Inline }`; `JsonSink::{pretty, line}`; `YamlSink`; `is_plain_safe`, `write_json_string`, `write_yaml_scalar` | Add; zero dependencies; generic, not `dyn`, so scalar calls monomorphize |
| `emit/emit_scalar.rs` | replaces `quote` (`report_format.rs`) and `yaml_scalar` | The `fdu-4xy9` policy: plain only for non-empty ASCII `[A-Za-z0-9._/+-]` that does not start with a digit or sign, is not dot-numeric or `.inf`/`.nan`, and is not a YAML 1.1 boolean or null spelling; otherwise double-quoted, escaping `"`, `\`, controls below 0x20, 0x7F-0x9F, U+2028, U+2029, U+FEFF, U+FFFE, and U+FFFF |
| `report_format.rs` | new `emit_report`, `emit_change`, `emit_cache_status`; `Field { name, presence: Presence }` with `Presence::{Always, Nullable, WhenLossy, WhenAnalyzer(AnalysisSet), WhenSet}` | The only declarations of document structure; the tree walk uses an explicit stack |
| `report_format.rs` | `render` | Keep, and add `write(report, format, color, out: &mut dyn io::Write)` for streaming |
| `report_format.rs` | JSON writers, YAML writers, `indent`, `collapse`, `json_count` | Delete |
| `report_format.rs` | `render_change`, `render_cache_status`, `report_schema` and schema constants, `render_text_metrics`, `share_metric_note` | Machine formats through the walks, with YAML change records as `---` documents; `fdu.report/7`, `fdu.stream/2`, `fdu.cache/2`; text decides what to show from unit presence and `pages()` |
| `query/query_report.rs` | `Report` | Carries `status`, `provenance`, and the request echo; `notes` and `ignored_entries` stay text-only and the schema marks them off the wire |
| `crates/fdu-py/src/lib.rs` | `PyIndex::report`, `report_dict` through `tree_dict`; `cache_status_dict`; `Index.since` change dicts | Delete the native dict after migrating its test; cache status and change sets read wire keys (`invalidate`, a labelled reason) |
| Documentation | `fdu.report/7` and `fdu.stream/2` in `docs/project/guides/cache-design.md`, `docs/project/architecture/fdu-surface-architecture.md`, `docs/project/architecture/fdu-engine-architecture.md`, `docs/project/release-notes/0.1.0.md`, `docs/project/guides/release-process.md`, `crates/fdu/src/skills/SKILL.md`, and `README.md` | Update |
| `crates/fdu-py/python/fdu/_models.py`, `_api.py` | `MetricValues`, `MetricRow`, `Detection`, `_metric_row`, `_tree`, `report_from_dict`; `_cache_status`, `_change` | Optional metric fields, `Pages`, per-unit coverage, paths preferring `path_raw`, an iterative `_tree`; read only wire keys |

**Call sites:** `crates/fdu/src/cli.rs` and its tests
`no_surface_names_a_schema_the_binary_does_not_emit` and
`formats_parse_and_machine_formats_are_never_colorized`; `execution.rs`;
`examples/perf_probe.rs`; `crates/fdu-py/src/opened_binding.rs` and `opened.py`;
`crates/fdu-py/src/lib.rs`; `_api.py`; `_models.py`; documentation in
`crates/fdu/src/skills/SKILL.md` and `README.md`.

**Tests:**
- `emit` unit tests: the scalar policy over the 121-string corpus from
  [`explorations/yaml-conformance`](../../../../explorations/yaml-conformance/README.md),
  copied to `crates/fdu-core/src/testdata/`; the JSON escape set; sink nesting and
  errors; a `SchemaCheck` adapter checking key order and presence against `Field`
  tables.
- `report_format.rs`: rewrite the YAML quoting, JSON escaping, JSON Lines,
  schema-version, stream-record, and cache-schema tests; extend stack-safety and
  non-Unicode path tests to YAML; add a JSON Lines test with `{ ` in a name.
- `scripts/check-yaml.mjs`, reusing the unmerged YAML fixes recorded on `fdu-c2ml`:
  awkward and non-UTF-8 names, `documents`, strict YAML 1.2 (`strict`, `uniqueKeys`,
  `intAsBigInt`) and YAML 1.1 parsing of every document kind deep-equal to exactly
  parsed JSON (the `JSON.parse` reviver’s `context.source`) and to reassembled JSON
  Lines, across views and analyzer sets, cache status, and a watch stream; no raw C1,
  U+2028, or U+FFFE.
- Python: move `crates/fdu-py/tests/smoke.py` to parsed `render("json")`; add a
  writer-equality test in `test_models.py`. YAML parser parity stays in Node unless a
  reviewed Python YAML dependency is added.
- Goldens: `cli-json`, `cli-content`, `cli-axes`, `cli-lifecycle`, `cli-cache`,
  `cli-surface`, and `cli-watch`; `crates/fdu/tests/cli_color.rs`.

**Commits:**
1. The emit module and scalar policy with unit tests.
2. The policy inside the existing writers, with strict YAML parsing in `check-yaml.mjs`.
3. JSON Lines through the walk; delete `collapse`.
4. Pretty JSON through the walk with a declared layout; golden diffs are whitespace
   only, proven by comparing parsed values before and after.
5. YAML through the walk, matching JSON’s shape.
6. Change records and cache status through walks.
7. Text over the model.
8. Python models, the `smoke.py` migration, and deleting the native dict.

**Risks:** byte-identical pretty JSON would need layout hints for today’s accidents, so
the plan accepts whitespace-only golden diffs verified by parsed equality; the sink
removes today’s repeated copying (`indent` and `collapse`), but `render` still
materializes a string for Python, and YAML indentation still grows with tree depth, so
record JSON, JSON Lines, and YAML render modes in `perf_probe` with `make perf-record`;
`Index.since` change sets are folded into the change-record model rather than kept as a
fourth shape.

### Phase 2, Item 3: The Execution Plan Model

```rust
pub struct Delivery { pub cache: CachePolicy, pub cache_path: Option<PathBuf>, pub workers: Workers,
  pub accept_partial: bool, pub watch: Option<WatchDelivery> }
pub struct Workers { pub scan: Option<usize>, pub analysis: usize }
pub enum Route { OneShot, Retained, Refresh, Watch, Opened }
pub struct Plan { route: Route, retained: RetainedState, load: Load, verify: Verify, delivery: Delivery }
pub fn plan(request: &Request, delivery: &Delivery, route: Route) -> Result<Plan, RequestError>; // the root is request.basis.root
impl Plan {
  pub(crate) fn admit(&self, stored: &StoreHeader, basis: &Basis) -> Admission;
  pub(crate) fn writes(&self, run: &RunFacts) -> SaveTargets;   // the only write decision
  pub fn outcome(&self, status: &TreeStatus) -> OutcomeClass;
}
impl Delivery { pub fn enumerate() -> impl Iterator<Item = Delivery> }
```

Plans for the same request may differ only in what they load and in provenance.
`Delivery::enumerate()` yields representative values for the axes that can be enumerated
(cache policy, `accept_partial`, and watch), with worker counts and `cache_path` fixed.

| File | Function or type | Change |
| --- | --- | --- |
| `execution.rs` | `ReportPlan`, `plan_report`, `prepare_report_internal` | Replace with `Plan` and `plan(.., OneShot)`, keeping `RetainedState`; execute the plan |
| `lib.rs` | `OpenConfig`, `open`, `open_with_pending_save`, `open_for_report`, `SaveTargets` and warm targets, `SNAPSHOT_MIN_ENTRIES`, `cold_scan_save_targets`, `load_content`, `spawn_save` | Delete `OpenConfig` in favor of `Basis` and `Delivery`; `open(&Basis, &Delivery)`; `open_for_report` becomes `execute(&Plan, &Basis)`, with the cache-only content check moving into `admit`; `Plan::writes` replaces the save-target logic; `load_content` and `spawn_save` become pure executors |
| `lib.rs` | new `refresh(&mut Index, &Basis, &Delivery)` | `Route::Refresh`: reconcile, load the sidecar, analyze, and write per the plan |
| `watch_session.rs` | `Session` | Hold the plan; add `Session::start(request, delivery)` and `persist_due(now) -> SaveOutcome` |
| `crates/fdu/src/cli.rs` | `SaveOutcome`, `save_is_due`, `pending_after`, `save_if_pending`, `save_live`, `run_watch`, `allow_partial`, `run`, `finish` | Move throttling into `Session`; `accept_partial` in `Delivery`; exit status from `Plan::outcome` |
| `crates/fdu-py/src/lib.rs` | `refresh`, `watch`, `PyWatch.__next__`, `open`, `scan`, `report_once` | Build a `Delivery`; `refresh` calls core `refresh`; `__next__` calls `persist_due`; the `Index.refresh` and `Index.watch` docstrings and the CHANGELOG say both write under `auto` |
| `opened.rs` | `OpenedIndex::open` | Take a plan with `Route::Opened` |
| `content/content_model.rs`, `content/content_analysis.rs`, `content/content_cache.rs` | `AnalysisRequest.workers`, `analyze_index`, `save_content_cache` | Workers move to `Delivery.workers` |

**Call sites:** `OpenConfig` literals (36 in `lib.rs` tests, 10 in `execution.rs`,
`cache.rs`, `opened.rs`, `crates/fdu-core/tests/watch_session_integration.rs`, five in
`examples/perf_probe.rs`, `crates/fdu-py/src/lib.rs`, `crates/fdu/src/cli.rs`); `open`
calls in `lib.rs`, `execution.rs`, `cache.rs`, `opened.rs`, `examples/perf_probe.rs`,
`watch_session_integration.rs`, `crates/fdu-py/src/lib.rs`, and `crates/fdu/src/cli.rs`;
`plan_report` in `execution.rs` tests.

**Tests:** for every `Delivery::enumerate()` value and route, `writes` is identical for
the same run facts; content is written after a partial scan when verified and a snapshot
of its entry identity exists, entries never; Python `Index.refresh()` writes so a later
cache-only open succeeds; a Python watch persists under `auto`. Retarget the planner
tests, turn `save_tests` (`lib.rs`) and `cold_scan_persistence_tests` into
`Plan::writes` tests, move the throttle tests (`crates/fdu/src/cli.rs`) into
`watch_session.rs`, and use `OutcomeClass` in `crates/fdu/src/cli.rs` and
`crates/fdu/tests/cli_exit.rs`.

**Commits:**
1. `Plan` for the one-shot route with no behavior change.
2. `open_for_report` consumes the plan.
3. `Plan::writes` as the only write decision, with per-tier rules and their tests.
4. Delete `OpenConfig` and migrate callers.
5. Core `refresh`, used by Python.
6. `Session::start` and `persist_due`, used by the command line and Python.
7. `accept_partial` through `Plan::outcome`.
8. Opened roots take a plan.

**Risks:** `open_for_report` is the engine’s densest function, so commits 2 and 3 land
only with the harness subset passing; Python refresh and watch start writing snapshots
under `auto`; `persist_due` takes the clock as a parameter so tests stay deterministic.

### Phase 2, Item 4: The `.gitignore` Observation Projection on Every Route

A controls-on snapshot whose other identity fields equal the request’s is parsed into an
index of the requested scope, without installing its control table.
That index equals a controls-off scan, because entries do not depend on observation, so
the projection happens at load and serves `open`, Python `open`, the watch’s initial
report, and warm revalidation alike.

| File | Function or type | Change |
| --- | --- | --- |
| `stored_state.rs` | `serves_snapshot` | Extend item 2’s `Exact` and `Refuse` with `ProjectControlsOff`; it takes no policy and no consumer |
| `snapshot.rs` | `load_serving(path, types, wanted) -> LoadOutcome::{Served(Index, Serves), Refused(SnapshotIdentity), Absent}`; `parse_stream` | Add; a projection builds the index with the requested scope and skips installing the control section |
| `lib.rs` | `snapshot_scope_serves`, `SnapshotUse`, `RefusedSnapshot`, `open_for_report` load filter, `unusable_snapshot_message`, `OpenReport` | Delete the first three; load through `load_serving`; the refusal message takes the refused identity; add `OpenReport.projected`, and a projected load does not overwrite the stronger snapshot |
| `execution.rs` | retag | Delete, with a debug assertion that the answer’s scope equals the request’s |
| `query/query_report.rs` | `forget_ignore_classification`, export at `query.rs` | Delete |
| `crates/fdu/src/cli.rs` | `run_watch`, `save_live` | The initial report warm-starts through `open_with_pending_save`; `save_live` skips writing while projected |
| Documentation | `lib.rs`; `execution.rs`; `_api.py`; `cli-cache.tryscript.md`; cache design Known Gaps | Rewrite |

**Tests:** `snapshot_serving_is_equality_plus_observation_on_to_off`;
`a_projected_load_equals_a_controls_off_scan`;
`controls_on_snapshot_does_not_serve_controls_off_auto_open` and
`controls_on_snapshot_does_not_serve_controls_off_cache_only_open` in `lib.rs` become
projection tests, plus `a_projected_open_leaves_the_stronger_snapshot_in_place`; extend
`controls_on_snapshot_projects_to_an_equivalent_controls_off_cache_only_report` in
`execution.rs` across policies and routes and flip
`controls_on_snapshot_does_not_serve_controls_off_auto_report`;
`a_session_over_a_projected_index_refuses_an_ignored_selection`; a Python smoke check
that a controls-off open answers from a default snapshot; a `cli-cache` golden answering
`warm_revalidate`; the `cli-watch-initial` harness route, clearing `projection-route`.

**Commits:**
1. `ProjectControlsOff` in `serves_snapshot`, and `load_serving`, with tests.
2. Route `open_for_report` through them and delete `SnapshotUse` and
   `snapshot_scope_serves`.
3. Delete the retag and `forget_ignore_classification`.
4. The no-overwrite rule and the `save_live` guard.
5. The Python check, golden, harness route, and documentation.

**Risks:** a controls-off watch never persists over the stronger snapshot, so each run
revalidates from an older one, a cost rather than a different answer; confirm that a
projected index’s empty control table cannot later be saved claiming limits the request
never had.

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
- **Retaining facts under an unverified subtree.** A pass drops them, which matches a
  cold walk and keeps roll-ups exact; retaining them marked stale and excluding them at
  read would let an opened root keep showing last-known children, at the cost of
  per-subtree subtraction in every reader and roll-up.
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
- Does `--watch --format json` keep a pretty initial report followed by one-line
  records, or behave as `jsonl`?
- Does Python expose a partial outcome that was not accepted without raising, given that
  the command line maps it to exit status 2?

## References

- [Design principles](../../architecture/fdu-design-principles.md),
  [engine architecture](../../architecture/fdu-engine-architecture.md),
  [surface architecture](../../architecture/fdu-surface-architecture.md), and
  [cache design](../../guides/cache-design.md)
- The [path-independence harness](../../../../tests/path_independence/README.md) and its
  2026-09-17 summary
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
