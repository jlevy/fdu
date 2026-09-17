# Feature: Explicit Core Models, So Caching Never Changes Semantics

**Date:** 2026-09-17

**Author:** fdu project, with Claude assistance

**Status:** Draft. Ships in 0.1.0: the release waits for the acceptance criteria below,
and scope may shrink only by the deferrals this plan names.

## Overview

fdu answers one kind of question, which directories and files hold what, through many
routes: cold scans, warm revalidation, cache-only reads, a summary reducer, retained
indexes, watch sessions, opened roots, three surfaces, and five output formats.
Those routes exist for speed and convenience.
None of them may change what an answer says.

A review of 0.1.0 release candidate `5f2d36d` found that the metadata core already
behaves that way, while content analysis does not: the same `--analyze` request returns
different numbers depending on which requests ran before it.
The root cause is not one defect.
The concepts every route depends on (what a request is, how it is executed, what stored
state is valid for, where a value came from, and what an answer looks like) have no
explicit model. Each route and surface declares its own version imperatively, and the
versions have drifted.

This plan gives each of those concepts one explicit model in `fdu-core`, makes every
route and surface consume the models instead of re-deriving them, and enforces the
consequence with a test: **caching improves performance and never changes semantics**.

## Decision Summary

- **Two first principles**, recorded in
  [the design principles](../../architecture/fdu-design-principles.md):
  [Model Every Key Concept Explicitly, in One Place](../../architecture/fdu-design-principles.md#model-every-key-concept-explicitly-in-one-place)
  and its consequence,
  [Caching Improves Performance, Never Semantics](../../architecture/fdu-design-principles.md#caching-improves-performance-never-semantics).
- **Five core models:** request, execution plan, stored state, provenance, and answer.
  Validation, defaults, compatibility, write rules, and serialization live in them and
  nowhere else.
- **One invariant, enforced mechanically:** for every request and every history of
  earlier requests and file changes, the answer equals the cold answer apart from
  provenance fields, or the run fails with a named reason, or it labels itself stale.
- **Ships in 0.1.0.** The north star, a clean design covering the supported analyses and
  views, is not negotiable.
  Scope may shrink only by the [deferrals](#scope-deferrals) listed here, each of which
  costs performance or convenience but never semantics.

## Goals

- One explicit model per core concept, defined once in `fdu-core`, consumed by the
  command line, the Python package, one-shot reports, retained indexes, watch sessions,
  and opened roots.
- The path-independence invariant holds for every supported request, cache policy,
  execution path, surface, and parsed machine format, and a test proves it.
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
- Changing what metadata-only answers report; they are already path-independent.

## Background

### Evidence

Four read-only investigations ran against `5f2d36d`:

- a stored-state audit of the metadata and `.gitignore` control tiers;
- a stored-state audit of the content-analysis and classification tiers;
- a map of what a request and an answer are on every surface and route;
- an empirical path-independence matrix on the release build: 63 request variants, each
  run cold and after 10 different warming requests under `auto`, `read-only`, and
  `only`, after file mutations, and through both the command line and Python, about
  20,000 runs.

| Requests | Result |
| --- | --- |
| 38 metadata-only variants (every scope flag, selection filter, view, and size metric) | Identical to cold on every history, policy, mutation, and surface |
| Analysis requests warmed by a wider analyzer set | 89 of 1,890 warm cases differ: unrequested metrics appear, requested metrics change (`physical_lines` 171 to 166, `document_words` 259 to 647), `languages` switches its share metric and reorders rows, and `analysis.analyze` names the stored set |
| Analysis requests after a mutation | 36 of 5,670 cases differ beyond that, including totals that match neither cold answer (`document_words` 38 where both cold runs give 54); afterwards every `--cache only` analysis request fails until an `--analyze all` run |
| Cold analysis requests with different analyzer sets | The same metric changes with the set: `--analyze code` drops a Haskell file’s line counts, and `logical_words` is 666 under `words` but 656 under `all` |

### Seven Concepts Without One Model

| Concept | How it is declared today | Consequence |
| --- | --- | --- |
| Request | Scope in `ScanConfig`/`ScanScope`; the analyzer set in `OpenConfig.analysis`, outside `Query` (`query/query_report.rs:353`, `lib.rs:158`); parsed separately in `cli.rs` and `fdu-py`; `validate_analysis` only in the surfaces, so a direct `prepare_report` call skips it; `validate_controls` called at seven sites; the size metric parsed separately in `cli.rs` and `fdu-py`, defaulting to apparent in Rust but allocated in the command line and Python; watch defaults to `tree` in the command line and to `files` in both the native binding and Python `WatchOptions` | `query::report(index, query, provenance)` cannot see the requested analyzers, so it presents whatever the index holds |
| Measured value | One coverage outcome per file for the whole analyzer set (`content/content_analysis.rs:302-313`); `document_words` switches meaning per record profile (`query/query_report.rs:1448-1464`, `:1577-1583`); a path’s classification depends on whether a content record exists (`query/query_report.rs:1390`) | An unsupported `code` analyzer erases the file’s line counts; one metric has several meanings |
| Stored identity and validity | The snapshot records an engine fingerprint and the scan scope (`snapshot.rs:201-214`, `:924`); the content sidecar records neither, and never compares analyzer versions or options (`content/content_cache.rs:88-96`, `content/content_model.rs:266-273`); entries compare six attributes, control files compare bytes, content records compare a five-field fingerprint | A sidecar survives an upgrade and is shared across scopes |
| Compatibility | Snapshots need exact scope equality plus one projection (`.gitignore` on to off) that only one-shot cache-only reads use (`lib.rs:377-395`); content uses containment with no projection | The same stored state answers one route and not another; content answers carry stored rather than requested analyzers |
| Execution | The one-shot planner (`execution.rs:147-181`), `open`, the watch initial report, and Python `Index` each decide what to read and write | Under `auto`, a one-shot metadata report never reads the snapshot while `open` does; the summary tier writes nothing under `auto`, `off`, or `read-only`; Python refresh never writes; content saves drop records from other analyzer sets |
| Provenance | Built at six sites (`execution.rs` twice, `cli.rs`, `watch_session.rs`, `fdu-py`, `opened/read.rs`); `scan_started_at` means three things (`execution.rs:241`, `fdu-py/src/lib.rs:1653`, `watch_session.rs:398`); watch repaints hard-code `complete: true`, `errors: []`, and `warm_revalidate` (`watch_session.rs:396-404`); content has no freshness | A watch repaint over a partial index claims completeness; decayed content metrics report fresh |
| Answer | Six writers (text, JSON, JSONL, YAML, the native Python dict, Python models) and no field schema; YAML flattens metric rows and omits `root_raw`/`path_raw`; JSONL’s `collapse` rewrites strings (`report_format.rs:1271`); `--watch --format yaml` emits JSON change records; the schema version follows stored content (`report_format.rs:1324`) | Formats that claim one schema disagree, and one corrupts paths |

### Why Tests Did Not Catch It

- Parity replays golden bytes through the Python console command, which calls the same
  Rust renderer, so it proves that both surfaces build the same request for the same
  route. It never compares routes or writers.
- 21 of 27 content-analysis goldens run with `--cache off`; the cached ones repeat the
  same analyzer set.
- Content reuse tests assert hit counts, not answers.
- `report()` is a pure function of an index, a query, and provenance, as designed, but
  the index is a function of request history.
  Purity at that boundary never implied the property that matters.

## Design

### The Rule

A **request** is what determines an answer’s content: root, scope, content (analyzers),
selection, and views.
**Delivery** is how the answer is produced and shown: cache policy, worker counts,
format, color, watch and interval, and whether a partial result is accepted.
An **answer** is a function of the tree and the request.
Delivery may change speed, provenance fields, and serialization, never content.

Formally, for request `R`, tree state `T`, and any history `H` of earlier requests,
cache operations, and file changes, a run under any delivery returns one of:

- the answer a cold run of `R` at `T` returns, with provenance describing how it was
  produced;
- a failure naming why the delivery cannot answer (for example, no usable cache under
  `--cache only`);
- under `--cache only`, the answer a cold run of `R` would have returned at a recorded
  earlier state, labelled stale.

The same request through any surface returns the same answer, and every machine format
parses to the same value.

### The Core Models

Each model is a typed value in `fdu-core` with its invariants as methods and tests.
Surfaces construct and render models; they never re-derive a rule the model owns.

**Request model.**
- **Owns:** root; scope (depth bound, filesystem boundary, `.gitignore` observation and
  limits, type rules, hidden and special-file handling); content (analyzer set and
  analyzer options); selection; views and view options (`words_per_page`); one table of
  defaults; value grammars; validation with typed errors (a view that needs analysis, an
  ignored-state selection without observation, watch-incompatible scope or cache policy,
  limits); derived choices (the default view for an analyzer set, views `full` omits);
  and the identities other models key on (`scope_identity`, `content_identity`).
- **Invariant:** every answer is produced from a validated request, and the reader that
  renders it receives the whole request, content included.
- **Replaces:** separate parsing in `cli.rs` and `fdu-py`, surface-only validation, the
  seven `validate_controls` sites, and per-surface defaults.

**Execution plan model.**
- **Owns:** given a request, a delivery, and available stored state, which tiers are
  needed, which stored entries answer through the stored-state model, what is verified,
  computed, and written, and which provenance results.
- **Invariant:** one-shot reports, `open`, Python `Index`, watch sessions, and opened
  roots all obtain their plan from this model.
  Alternative plans for one request may differ in cost, never in answer.
  Writes follow one rule: a run may write only tiers it verified completely, keyed by
  their identity, and never replaces another identity’s entry with narrower data.
- **Replaces:** `plan_report`, the imperative read and write decisions in `lib.rs`, the
  command line’s watch path, and Python’s `Index` refresh and watch.

**Stored-state model.** One contract for every retained or cached tier:

| Element | Rule |
| --- | --- |
| Identity | The engine fingerprint (crate version, formats, rules versions) plus exactly the request parts that change the tier’s values |
| Validity | One per-item fingerprint (size, allocated bytes, mtime, ctime, inode, device), plus source bytes where a tier derives from file content |
| Serves | `serves(stored, requested)`: equality by default |
| Projects | Any relation beyond equality comes with `project(stored, requested)`, which yields exactly what a cold run of the requested identity would; the invariant test proves each one |
| Storage | Entries keyed by identity, not one slot per root; eviction is a miss, never a different answer |
| Provenance | Each tier reports its own source, freshness, observation time, and coverage |

| Tier | Identity beyond the engine fingerprint | Serves and projects |
| --- | --- | --- |
| Entries and roll-ups | Scope | Equality |
| `.gitignore` control state | `.gitignore` observation and limits | Equality, plus observation on to off, on every route |
| Classification | Type rules, and whether the request’s content axis enables content probes | Equality |
| Content | Scope, type rules, and per analyzer: identity, version, options | Per analyzer: a record set holding more analyzers answers fewer by returning only the requested analyzers’ results |

Content records store results and coverage per analyzer, so an unsupported `code`
analyzer never erases `lines` results, and serving fewer analyzers is an exact
projection rather than a relabelling.
Each metric belongs to exactly one analyzer and depends only on the file and that
analyzer. A metric whose analyzer was not requested is absent from the answer, not zero.
A view that derives a value from alternative metrics (pages from prose words or raw
words) names the metric it used.

The summary reducer is an execution plan, not a stored tier: it must return the same
answer as the full index, and it writes nothing because it verifies no complete tier.

**Provenance model.**
- **Owns:** per tier, the source (cold scan, revalidated, cache, journal-scoped), the
  freshness, the observation time, completeness, coverage, and errors; the composition
  of tier provenance into a report’s envelope.
- **Invariant:** every route computes provenance from the model; nothing asserts it.
  `scan_started_at` has one meaning.
- **Replaces:** watch repaints’ hard-coded fields, the three `scan_started_at`
  definitions, and the absence of content freshness.

**Answer model.**
- **Owns:** one typed value per document kind (report, change record, cache status) with
  a field schema; an echo of the request identity (scope, content, size metric, views);
  per-tier provenance; schema versions that describe shape, never history.
- **Invariant:** every writer serializes the model: text renders it, JSON, JSONL, and
  YAML parse back to the same value, and Python values are constructed from it.
  No writer alters a string.
  YAML uses the conformant scalar policy chosen by the YAML evaluation (`fdu-4xy9`):
  plain only when unambiguous under YAML 1.1 and 1.2, otherwise double-quoted with the
  shared escape set.
- **Replaces:** six independent writers, the native Python dict, JSONL’s string
  collapse, and YAML’s hand-copied field lists.

### Sessions

A **watch session** is a request plus a plan re-evaluated on each commit.
It serves a tier only if it keeps that tier current; content analysis is refused at
session creation until live re-analysis exists.
Each repaint’s provenance comes from the provenance model.
`--watch --cache only` is refused, because cache-only never touches the tree.

An **opened root** validates its reads with the request model, so a `documents` read
without content analysis is refused, not answered with zero words.

A **Python `Index`** carries the request it was opened with.
A report on it with a different content axis is either projected by the stored-state
model or refused.

### Components

- `crates/fdu-core/src/query/`: the request model (absorbing `Query`, `Selection`,
  `ViewSpec` resolution, validation, and defaults) and the answer model.
- `crates/fdu-core/src/execution.rs` and `lib.rs`: the execution plan model, used by
  every route.
- `crates/fdu-core/src/snapshot.rs`, `cache.rs`, `content/`, `control*`, and
  `classify*`: the stored-state contract, identity headers, keyed storage, per-analyzer
  content records.
- `crates/fdu-core/src/watch_session.rs` and `opened/`: sessions consume the plan,
  provenance, and request models.
- `crates/fdu-core/src/report_format.rs`: writers over the answer model.
- `crates/fdu/src/cli.rs`: flag parsing into the request model, rendering, and exit
  codes only.
- `crates/fdu-py/`: bindings construct request models and expose answer models.
- A path-independence test harness and a writer-equality test under `tests/`.

### API Changes

These are breaking changes, and they land before 0.1.0 is published.

- **Rust:** a request type carrying scope, content, selection, and views, accepted by
  the report reader; typed validation errors; per-analyzer `FileAnalysis` results; tier
  identity types exposed through cache status; per-tier provenance; answer model types.
- **Python:** `report`, `open`, `scan`, `Index`, and `fdu.opened` accept the same
  request model; `AnalysisMetadata` reports the requested analyzers; unrequested metrics
  are absent.
- **Command line:** no new flags; `--watch --cache only` becomes a usage error; cache
  status shows each entry’s identity.
- **Schemas:** new versions of the report, stream, and cache-status schemas for absent
  unrequested metrics, the request echo, per-tier provenance, and identity in cache
  status.
- **Cache formats:** new snapshot and content-store formats keyed by identity.
  Existing files read as stale and are removed by `--cache-clear`, as today.

## Implementation Plan

### Phase 1: Models and Enforcement

- [ ] Land the path-independence harness in the repository with a registry of known
  violations that fails on any new difference and on any stale entry, and run it in CI
  on Linux, macOS, and Windows.
- [ ] Add the metric-independence test and the writer-equality test (JSON, JSONL, YAML
  under strict YAML 1.1 and 1.2, Python models), including adversarial names, both
  starting as registered known violations.
- [ ] Build the request model; route the command line and Python through it; move
  validation and defaults into it; give the report reader the whole request.
- [ ] Build the answer model and move every writer onto it, including the YAML scalar
  policy and JSONL without string rewriting.
- [ ] Build the provenance model and use it in one-shot reports, watch repaints, Python
  `Index`, and opened reads.

### Phase 2: Stored State and Execution Conform

- [ ] Build the execution plan model and route one-shot reports, `open`, Python `Index`,
  watch sessions, and opened roots through it, with the single write rule.
- [ ] Give every tier an identity header and the shared per-item fingerprint; key
  storage by identity (subsumes `fdu-w3l5`).
- [ ] Store content results and coverage per analyzer; give each metric one analyzer and
  one definition; make unrequested metrics absent; make classification a function of the
  request.
- [ ] Implement `serves` and `project` for the `.gitignore` observation projection and
  content analyzer subsets on every route.
- [ ] Apply the session rules: watch refuses content and cache-only, repaints compute
  provenance, opened reads validate requests, Python `Index` projects or refuses.
- [ ] Empty the known-violation registry; remove the known-gaps sections from the
  architecture documents.

## Scope Deferrals

Each of these may be cut from 0.1.0 to ship sooner, because each costs performance or
convenience and none changes an answer or leaves a concept without its model:

- Content reuse across different analyzer sets: ship per-analyzer records with exact-set
  reuse only, and add subset projection afterwards.
- Treating scopes that cannot change an answer as one identity (a depth bound beyond the
  tree’s height, a `.gitignore` budget no file reaches); they stay distinct identities.
- Bounded multi-identity eviction policy: ship a fixed small number of identities per
  root.
- Live re-analysis in watch sessions: sessions refuse content analysis.
- Source and freshness in human text output.
- A string representation for JSON integers beyond 2^53 (`fdu-cggg`): document the
  limitation instead.
- Free-threaded CPython wheels (`fdu-4itz`).

## Testing Strategy

- **Path independence:** the harness replays requests across every scope flag, selection
  filter, view, analyzer set, and size metric, after warming histories, under each cache
  policy, after mutations, and through the command line, Python one-shot reports, and
  Python `Index`, comparing parsed answers with cold answers apart from provenance.
  A fast subset runs in `make check`; the full matrix runs in CI.
- **Metric independence:** for each metric, the cold value on a fixture is identical
  under every analyzer set that includes its analyzer.
- **Writer equality:** every document kind parses to the same value from JSON, JSONL,
  and YAML (strict 1.1 and 1.2), and equals the Python model, over a corpus of
  adversarial names and non-UTF-8 paths where the platform allows them.
- **Identity:** an engine, format, rules, or analyzer version change invalidates every
  affected tier; a scope change never serves another scope’s entry except through a
  declared projection.
- **Sessions:** watch repaint provenance over a partial index; refusal of content and
  cache-only in sessions; opened reads refuse invalid requests.
- **Goldens:** updated deliberately, with each diff attributed to a model change.

## Rollout Plan

- Everything lands on `main` before the 0.1.0 tag; the release epic `fdu-gjc2` depends
  on this plan’s acceptance criteria.
- The CHANGELOG describes the shipped models and schemas as the 0.1.0 contract, not as
  changes from an unreleased shape.
- The release rehearsal and the end-to-end verification (`fdu-tyvq`) run on the final
  commit, with the path-independence harness included in that verification.

## Acceptance Criteria

- The path-independence, metric-independence, and writer-equality tests pass on Linux,
  macOS, and Windows with an empty known-violation registry.
- No request field is parsed, defaulted, or validated outside the request model, and no
  route decides reads or writes outside the execution plan model.
- Every stored tier has an identity header, the shared validity fingerprint, and
  `serves`/`project` in the stored-state model.
- Watch repaints, opened reads, and Python `Index` reports compute provenance from the
  provenance model.
- The design principles carry no conformance caveat, and the architecture documents
  carry no known-gaps sections for these models.

## Open Questions

- Does the request model extend `Query` and `ScanConfig`, or replace them with one type
  that contains both?
- Which pages metric does the `documents` view use without the `words` analyzer: raw
  words, or no pages at all?
- Should classification use content probes whenever any analyzer runs, or should
  grouping always use name-based classification, with probe results reported as
  detection only?
- Are unrequested metrics absent keys or explicit `null` values in JSON and YAML?
- Does the answer echo the whole request or a request identity?
- Are relative time windows in watch sessions fixed at session start or re-evaluated on
  each repaint?
- How do the new public model types interact with the extensibility decision in
  `fdu-lmfd`?

## References

- [Design principles](../../architecture/fdu-design-principles.md),
  [engine architecture](../../architecture/fdu-engine-architecture.md),
  [surface architecture](../../architecture/fdu-surface-architecture.md), and
  [cache design](../../guides/cache-design.md)
- Related plans:
  [file content metrics](../done/plan-2026-08-12-fdu-file-content-metrics.md),
  [cache layers and defaults](../done/plan-2026-08-15-fdu-cache-layers-and-defaults.md),
  [view vocabulary and output contract](plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md),
  [opened-root inventory engine](plan-2026-08-25-fdu-opened-root-inventory-engine.md),
  and [disk-usage checkpoints](plan-2026-09-13-fdu-disk-usage-checkpoints.md), whose
  checkpoint compatibility should follow the stored-state model
- Beads: release epic `fdu-gjc2`; `fdu-gija` (cache-dependent analysis answers),
  `fdu-snv3` (watch with analysis), `fdu-c2ml` (YAML contract), `fdu-azz3` (content
  model), `fdu-fft9` (conformant output), `fdu-4xy9` (YAML evaluation), `fdu-ky5m` and
  `fdu-7dj6` (per-analyzer records), `fdu-w3l5` (snapshots keyed by scope), `fdu-c5v1`
  (field schema)
- Content reuse by containment was introduced in `2aa7da1` (#37)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
