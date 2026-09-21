# Feature: Directory Filters and List Presentation Formats

**Date:** 2026-09-20

**Author:** fdu project

**Status:** Implemented; final validation and stacked PR delivery in progress.
The plan is PR #96. The implementation layer builds directly on its file/function map.

**Tracking:** Epic `fdu-65x1`; plan publication `fdu-79n0`;
[issue #93](https://github.com/jlevy/fdu/issues/93).

## Overview

Find stale `.venv`, `node_modules`, and Cargo `target` directories by name and subtree
modification age, and report each matching directory’s size and age.
Keep the current default directory-tree output unchanged while making the axes clear:
entry kind is a filter, `list` is the metadata default view, and `tree`, `paths`, and
`long` are explicit human presentation formats.

This plan is a separate PR on `codex/directory-query-plan`, based on
`perf/campaign-linux-2026-09-19` at `c234da2b`, the latest stack layer:
[PR #91](https://github.com/jlevy/fdu/pull/91) →
[PR #92](https://github.com/jlevy/fdu/pull/92) →
[PR #94](https://github.com/jlevy/fdu/pull/94) → this plan.
Implementation will be reviewed separately above the plan.
No runtime or public usage behavior changes in this PR. All design and delivery context
needed to implement it is recorded below; bead descriptions carry execution status and
evidence.

## Goals

- Select directories using the same filter axis as files, including name/path, kind,
  size, and modification bounds.
- Report eligible subtree metrics once per matching directory in flat formats.
- Preserve ordinary default output exactly, including its directory-only tree shape and
  existing display bounds.
- Support the same capability in core, CLI, and Python from one retained index.
- Update all user documentation and help examples and prove surface/cache parity.

## Non-Goals

- File deletion, reclaimability estimates, access-time or last-use tracking
- A dedicated directories view or a Unix-find expression language
- A redesign of the default tree layout or an unbounded default terminal dump
- New scans for each name/format, cache-format changes, or new content analysis
- Dependency changes or performance claims

## Design Rationale

A directories view would conflate entry type with the report being requested.
Using `--kind dir` keeps it composable with the existing name/path and age/size filters.
Calling a mixed file/directory listing `list` makes its membership clear.

Putting tree, paths, and long on the format axis lets users request a presentation
without changing those filters.
Tree remains the current directory roll-up presentation; flat formats expose one
matching entry per row.
A tree therefore need not render the same literal path rows as a flat list, but all
metrics must follow the same selection contract.
Keeping the existing default output preserves the useful disk-usage overview.

## Background

Matching a directory such as `.venv` currently selects its inode instead of its subtree
size. Matching its descendants produces many rows and applies age predicates to
individual entries. Users need to find environments and build outputs by directory name,
filter them by aggregate size and modification age, and obtain one result per matching
directory from a reusable retained index.

The interface separates selection, the report being requested, and presentation.
Directories are an entry-kind filter, not a separate view.

## Accepted Interface

| Axis | Responsibility | Values and examples |
| --- | --- | --- |
| Scope | What is scanned and cached | Path, scan depth, filesystem boundaries |
| Selection | Which entries match | `--kind dir`, include/exclude name or path patterns, size and modification bounds |
| View | What information is reported | `list`, `summary`, `extensions`, `types`, `families`, `languages`, `documents`; largest/recent remain list presets |
| Format | How the report is presented | `tree`, `paths`, `long`, `json`, `jsonl`, `yaml`; automatic text tables for grouped views |

With metadata-only defaults, `fdu PATH`, `fdu PATH --view list`,
`fdu PATH --format tree`, and `fdu PATH --view list --format tree` are equivalent.
The default view is `list`; its default format is `tree`. Explicit `--format tree` is
the explicit form of that default, not a different query.
Analyzer-driven default views remain available so requested content analysis is shown.
An explicit view always wins and does not enable an analyzer.

**Default output must remain unchanged.** Compare against the pre-change behavior on the
same fixture: the directory hierarchy, allocated sizes, ordering, columns, bars, ignored
annotations, two-level depth, ten children per directory, omission notices, and
diagnostics remain the same.
This is an interface clarification and an additional set of formats, not a redesign of
the default report. Directory-filter corrections requested by issue #93 are tested
separately from unchanged ordinary default output.

The formats for `list` are:

- `tree`: the existing directory hierarchy and roll-up text presentation.
  Regular files contribute to directory totals rather than gaining individual leaf rows.
  Ancestors provide context for selected subtrees.
  Preserve this behavior for omitted format and explicit `--format tree` alike.
- `paths`: flat matching paths only, one per line, with the project’s existing safe
  path-escaping rules.
  No size/age columns or report headings on stdout.
- `long`: one flat row per match with the selected size metric, actual modification age,
  and path. Display columns are consistent for files and directories.
- JSON, JSONL, and YAML: structured representations of the same selected entries and
  metrics, including exact timestamps and the reference instant for age.

`--tree` and `--long` are convenience aliases for the corresponding formats, not
independent view switches.
No `short` spelling is needed; `paths` states its contract.
Conflicting explicit format choices fail clearly rather than using argument order.
Omitted format chooses human-readable tables for grouped/summary views and tree for
list. Keep `text` as the general automatic human presentation spelling.
Explicit list-only formats with incompatible views fail validation before scanning.
Define and test multi-view behavior; automatic text and machine formats support mixed
views, while `paths` cannot silently discard non-list sections.

| Requested view | Omitted format / `text` | `tree` | `paths` / `long` | JSON / JSONL / YAML |
| --- | --- | --- | --- | --- |
| `list` | Existing directory tree | Same directory tree | Flat matching entries | Structured list and metrics |
| Aggregate view (`summary`, `extensions`, and others) | Existing report/table | Usage error | Usage error | Structured aggregate |
| Mixed list and aggregate views | Tree plus existing reports/tables | Usage error | Usage error | All requested sections |
| `full` | Existing bounded digest | Usage error | Usage error | Existing bounded digest contract |

Largest/recent retain their documented regular-file selection and ranking presets.
Their automatic human presentation remains compatible; explicit Paths and Long render
the ranked regular files.
Tree is rejected because a directory-only hierarchy cannot faithfully display that
ranked file selection.
Legacy `--view files` keeps name order in flat formats and accepts explicit Tree with
tree ordering. Legacy `--view tree` retains its structured hierarchy with machine
formats; explicit Paths and Long override it.
This preserves existing tree JSON consumers while new List machine requests return a
complete flat inventory.

## Selection and Directory Metrics

Name/path and kind constraints remain ordinary filters.
Existing include syntax keeps its documented basename-versus-relative-path meaning; this
work does not introduce an expression language.
`--kind dir` restricts the result to directories.
A directory’s metrics have the same meaning when that kind filter is omitted.

Before positive include, kind, size, and age predicates are evaluated, compute directory
metrics over eligible retained contents.
Apparent and allocated sizes sum regular-file bytes using existing accounting, excluding
directory-inode and symlink bytes.
Counts exclude the matched root.
Modification time is the maximum observed mtime of the root and eligible descendants,
including directories and symlinks.
Empty directories use their own mtime.
This is modification activity, not access time or proof of last use.
Age uses one request reference instant; future timestamps produce negative age,
pre-epoch timestamps remain valid, and unrepresentable values are explicit unknowns.

Exclusions win throughout the subtree: a selected parent must not resurrect entries
rejected by `--exclude` or `--exclude-ignored`. Excluding a directory removes that
subtree from eligible measurements.
Preserve documented `--only-ignored` traversal through structural ancestors and count
only eligible contents.
Explain that directory size and recency reflect the remaining contents after exclusion.

Positive matching of a directory includes its eligible contents in aggregate views;
descendant names need not independently match.
Flat list formats emit only matching roots/entries, not every covered descendant.
Nested matching directories are all listed.
Their row sizes may overlap, while summary and grouped views aggregate the union of
covered regular files exactly once.
Symlinks are not followed.
The scan root remains structural context under the existing descendant-selection
contract. Hard links and shared extents retain existing accounting; reported size is not
a promise of uniquely reclaimable disk space.

## Ordering, Bounds, and Honesty

Keep the existing tree’s size-descending order and tie-breaking behavior unchanged.
The new flat presentations use size-descending order with deterministic path ties;
legacy files presets retain their documented name ordering where needed.
Explicit sort/reverse options use the same metrics across formats.
`--sort name` gives a stable complete inventory.
Largest/recent remain named regular-file list presets, with their documented ranking and
bounds. Selection and directory metrics must not depend on the chosen format.

Flat list output is complete by default.
Preserve the existing tree’s two-level and ten-children-per-directory display bounds and
current omission markers verbatim.
Preserve the documented per-group meaning of `--limit`: per directory in tree output,
and over the flat result list in flat output.
`--limit all` removes row caps and `--depth all` removes tree depth folding.
Do not introduce a new global top-N selection step into existing tree behavior.
Bounds are presentation constraints, not changes to which entries satisfy filters or
contribute to subtree metrics.
Machine list output must not inherit an implicit text-tree display cap.
Distinguish reported truncation, display folding, and scan incompleteness in the model
and tests.

`full` retains its existing bounded digest output, including its directory-tree section.
It must not acquire an unbounded flat listing or a different preview through the rename.
Retain source, freshness, partial results, scan-depth coverage, and cache-only labels.
Path-only output keeps its stdout contract; any necessary truncation/completeness notice
goes to the documented diagnostic channel with the normal exit status.

## Engine and Surface Work

Implement measurement and selection in `fdu-core` as iterative readers over the retained
index. Repeated names or formats must not trigger a filesystem walk, content read, or
cache identity change.
Preserve index ownership, reducers, snapshot format, and serving lifecycle.
Account for extra traversal in opened-report work budgets.
Preserve the current default tree’s bounded projection cost: supporting flat formats
must not force an unfiltered default tree to materialize every matching entry.
Share measurement and selection semantics without eagerly building unused presentations.
Use native path identity for one-shot queries and portable identity where required by
opened reports. Keep raw native entry metadata explicitly distinct from report subtree
metrics; do not silently alter the native entry projection contract.

Expose the shared list view and format model while retaining the existing directory
roll-up renderer. Tree is an aggregate presentation of selected contents; flat output is
a row per match, so rendered path sets need not be identical.
Implement validation/default resolution in core, with thin CLI and Python adapters.
Expose typed list rows with kind, both byte metrics, subtree counts where applicable,
modification time, age/reference time, and ignored classification.
Use versioned machine-schema changes for altered meanings or shapes.
Rendering the same report in different formats preserves its reference clock.

Audit released CLI and library contracts before choosing aliases.
Support the existing `files` spelling as a compatibility name/preset where needed and
translate legacy tree requests at the boundary, without creating duplicate engine
machinery. Preserve default output and document the directory metric correction and new
spellings. An explicit new format wins over a legacy spelling’s presentation default.

## Documentation and Examples

Update README, `docs/usage.md` (`--docs`), portable `--skill`, short and long help, Rust
API docs, Python README/models/stubs, machine-schema reference, architecture axes and
selection semantics, and appropriate release notes.
Keep examples aligned across these surfaces.
Cover individual and combined searches for `.venv`/`venv`, `node_modules`, and Cargo’s
default `target` build directory older than 7 or 30 days.
Use the existing `--modified-before=30d` grammar for age constraints.

Examples to validate include:

```shell
fdu ~/projects --kind dir --include .venv --modified-before 30d
fdu ~/projects --kind dir --include .venv --modified-before 30d --format tree
fdu ~/projects --kind dir --include node_modules --modified-before 30d --format long
fdu ~/projects --kind dir --include target --modified-before 30d --format paths
fdu ~/projects --kind dir --include .venv --modified-before 30d --format json
```

Repeated includes form a union of names.
For a combined inventory, oldest first:

```shell
fdu ~/projects --kind dir --include .venv --include venv \
  --include node_modules --include target --modified-before 30d \
  --format long --sort mtime --reverse
```

These names are conventions: a directory named `target` is not proof that Cargo owns it.
Include patterns select names, while size and age measure the eligible contents.

Include oldest-first sorting, both size metrics, ignored inventory, multiple queries
over one retained index, tree expansion, and machine-readable size/age extraction.
Explain nested overlaps, exclusion effects, modification age versus last use,
completeness, and non-unique disk accounting.
Do not include deletion commands.

## Acceptance and Delivery

Deterministic tests cover empty and nested roots, fresh file/directory/symlink activity,
future and pre-epoch timestamps, unknown ages, half-open modification-time bounds and
exact size boundaries, descendant exclusions, ignored policies, both sizes, stable
sorting, tree context, and union aggregation.
Verify repeated queries perform no filesystem work, portable names, deep-tree stack
safety, opened budgets, and mixed views.

Add a view/format compatibility and defaults matrix, explicit default equivalence,
aliases/conflicts, exact matching-path equivalence across paths/long/machine list
output, agreement of tree roll-ups with selected contents, folding versus limits, and
structured schema tests.
Keep pre-change default-output goldens as regression requirements; do not regenerate
them to accept new file leaves, columns, ranking, depth, or row counts.
Exercise real cold/warm/cache-only sessions and Python parity using reviewed portable
goldens. Preserve named patterns; record platform-dependent parity artifacts on Linux.

Run `make docs-format` and required `make check`; use `make cross-lint` if
platform-gated code changes.
Review, commit, push, open/update an implementation PR above the reviewed plan layer,
and watch CI to completion.
Close finished beads and sync after passing CI. No dependency changes or performance
claims are planned.

## Implementation Plan

The six implementation beads form one delivery sequence.
Core precedes formats; formats precede CLI integration; documentation and help depend on
both surface changes; final validation depends on documentation and help.
The plan-publication task is `fdu-79n0` and does not close the implementation epic.

### fdu-hw5k: Core list selection with directory subtree metrics and union aggregation

- [x] Complete and validate this delivery step.

Implement the epic’s shared list query in `fdu-core`. Entry kind is a filter; directory
size, counts, and newest activity are subtree metrics even without `--kind dir`. Apply
exclusions throughout eligible contents before aggregate bounds; positive directory
matches cover descendants for summary/grouped union aggregation without adding unmatched
descendants to flat listings.
Nested roots are emitted independently and aggregate totals count their covered contents
only once.

Provide the shared list query with existing directory roll-ups for tree presentation and
exact matching rows for flat/machine formats.
Preserve default tree output; files contribute to directory totals without new
individual leaf rows.
Separate selected matches from structural ancestors and report truncation from
presentation folding.
Use iterative retained-index readers, preserve cache/snapshot identity and raw-entry
semantics, support portable identity, and charge opened-report work budgets for all
traversals.

Acceptance: deterministic size/count/mtime and boundary tests, empty/nested/fresh roots,
file/directory/symlink recency, future/pre-epoch/unknown age, descendant exclusion and
ignored policies, all-kind matching, union totals, repeat queries without I/O, stable
sort and limit, deep-tree safety, and default/explicit-format-independent metrics.

### fdu-ywh0: Add tree, paths, and long list formats across core and Python

- [x] Complete and validate this delivery step.

Implement core-owned `tree`, `paths`, and `long` presentations of the shared list
report, alongside JSON, JSONL, and YAML. Tree preserves the existing directory roll-up
output, including hierarchy, columns, annotations, ordering, bounds, and omission
markers. Regular files contribute to directory totals rather than appearing as leaves.
Paths contains matching paths only.
Long contains size, actual age, and path.
Formats preserve filter semantics, directory metrics, and the request reference clock.
Flat output lists matches; tree output groups selected contents into directory roll-ups.

Keep grouped views as automatic human-readable tables.
Support mixed views in automatic text and machine formats; validate list-only format
incompatibilities in the engine before work begins.
Preserve existing tree depth and per-directory limits; flat limits apply to flat rows.
Disclose bounds without contaminating paths stdout.
Keep full’s existing bounded digest unchanged; do not add an unbounded list or a
different preview.

Expose the new view/format model through Python enums, report models, render APIs,
native adapters, and stubs.
Provide exact machine metrics, timestamps, signed age and reference instant with
explicit unknown handling, counts, kind, and ignored state.
Version changed report meanings/shapes.
Test unchanged default-output goldens and omitted/explicit tree equivalence, flat-format
membership equivalence, tree aggregate consistency, aliases, multi-view validation,
truncation/folding, machine decoding, and Python/Rust parity.

### fdu-y5ya: Expose list defaults, format aliases, and compatibility through the shared request model

- [x] Complete and validate this delivery step.

Expose the accepted selection/view/format model through shared engine defaults and
validation. Metadata-only default view is list and its default format is tree.
Prove `fdu PATH`, `--view list`, `--format tree`, and their explicit combination are
equivalent to the pre-change default output, including directory-only tree rows,
columns, ordering, depth, limits, and omission notices.
Preserve analyzer-driven default views and automatic grouped tables.

Add `--format tree|paths|long` alongside text/json/jsonl/yaml.
Make `--tree` and `--long` aliases for formats.
Explicit conflicting choices fail clearly.
Incompatible view/format combinations fail before scanning; mixed views never disappear
silently. Keep the existing tree’s ordering, per-directory limits, and depth behavior;
document flat-row limits and tree expansion.
Formatting does not change filter membership or subtree metric definitions.
Use existing age grammar (`--modified-before=30d`).

Audit released `--view files`, `--view tree`, and Rust/Python public contracts; retain
needed compatibility names/presets at request boundaries without duplicate engine logic.
Explicit formats override legacy presentation defaults.
Preserve default output and document new spellings and any necessary schema changes.
Exercise one-shot, watch, opened reports, and cache-status format dispatch so adding
human formats does not bypass diagnostics or status handling.

Acceptance: unchanged default-output regression goldens, shared default-resolution
tests, CLI parsing and help tests, compatibility and invalid-combination matrix,
aliases/conflicts, early failures, and cross-surface request parity.
CLI must not invent filtering, aggregation, or format semantics.

### fdu-2v7o: Document list formats, directory filters, and stale build inventories

- [x] Complete and validate this delivery step.

Update README, docs/usage.md (--docs), portable --skill, Rust API docs, Python README,
models/stubs, machine-schema reference, architecture axes and selection semantics, and
appropriate release notes to the accepted list-view and format model.

Explain unchanged default output, default list/tree equivalence, formats
tree/paths/long/json/jsonl/yaml, automatic grouped tables, format aliases, compatibility
names, invalid combinations, analyzer-driven defaults, and bounds/folding.
Directory kind is a filter.
Size/age describe eligible subtrees; exclusions win, nested rows overlap, summary unions
do not double-count, and structural tree ancestors are not extra matches.
Tree retains its existing directory roll-ups; matching regular files contribute bytes
without new individual leaves.
Flat formats show one row per match.
Describe current tree bounds and flat-list bounds without implying identical rendered
path sets.

Include .venv/venv, node_modules, Cargo target, individual and combined names, 7d/30d
modified-before examples, long size/age columns, exact machine values, oldest-first,
ignored inventory, and repeated queries over one retained index.
Distinguish modification age from last use, scan coverage from display bounds, and
counted bytes from uniquely reclaimable space.
Apply common-doc guidelines and run docs-format.
Check every example against the implementation and keep README/help/skill/Python
terminology aligned.

### fdu-ia8p: Enhance help examples with README workflows and age/size directory searches

- [x] Complete and validate this delivery step.

Enhance short and long help with README workflows and stale-directory inventory
examples. Teach selection (kind/name/path/age/size), view (list and aggregate reports),
and format (tree default for list, paths, long, machine formats).
Show omitted versus explicit tree equivalence and convenient --long/--tree format
aliases. Default output remains the existing directory roll-up tree, including its
display bounds.

Include .venv/venv older than 7d, node_modules and Cargo target older than 30d, combined
name matching, actual size and age via --format long, paths-only output, JSON, and
oldest-first sorting using existing flag grammar.
Explain subtree recency, exclusions, tree context, and expansion of folded output
concisely. Make format compatibility and legacy names discoverable without retaining
directories-as-a-view terminology.

Review help goldens and execute every example’s syntax against a deterministic fixture.

### fdu-arv8: Verify unchanged defaults, directory formats, surface parity, and stacked PR CI

- [ ] Complete and validate this delivery step.

Validate the epic end to end with portable product goldens, engine tests, and Python
parity. Cover metadata default list/tree equivalence, explicit formats and aliases,
incompatible combinations, grouped/mixed views, legacy names, existing directory
hierarchy, ancestor context, sorting, folding, per-group limits, and unchanged bounded
full reports. Retain pre-change default-output goldens: no new file leaves, columns,
ordering, depth, limits, or omission markers under ordinary defaults.

Cover directory subtree size/age, nested overlap and union totals, descendant
exclusions, ignored policies, cold/warm/cache-only sessions, repeated names over one
index, no extra filesystem/content work, partial coverage/freshness, portable paths,
deep trees, and opened read budgets.
Compare matching paths and exact metrics across flat/machine formats and surfaces;
verify tree roll-ups agree with selected contents without requiring flat file leaves.
Validate JSON/JSONL/YAML schema changes and age reference consistency.

Review expected golden diffs and preserve named portability patterns.
Record parity artifacts on Linux per repository policy.
Run make docs-format and make check; run cross-lint if platform-gated code changes.
Review and commit the implementation separately, stack its PR above this plan layer, and
watch CI pass. Close finished beads and tbd sync after successful delivery.

## File and Function Implementation Map

This map is tracked by `fdu-iwt8`. The refreshed parent remains PR #94 at `c234da2b`;
other open PRs based on that branch are siblings, not prerequisites for this change.
The implementation branch starts from this plan branch after this map is committed.
The six runtime beads above own the changes below.

### Request, view, and projection resolution

In `crates/fdu-core/src/query/query_request.rs`, add an optional presentation format to
`ReadSpec`, parse it in `build_query`, and retain the resolved format in `Query`.
`Request::validate_against` owns the view/format compatibility matrix so invalid
requests fail before scans, cache access, or opened-report work.
Add the format axis to `AxisNames` and render the same rejection with CLI flag or Python
field spelling. Typed requests receive the same validation as parsed requests.

In `query/query_report.rs`, add `ViewSpec::List`, make `default_for` choose it for
metadata, and update `parse`, `vocabulary`, `label`, `resolve_rejecting`, and defaults.
Preserve the bounded `full` expansion rather than mechanically adding List to its
sections. Retain legacy `Tree` and `Files` presets: omitted/text formatting preserves
those presentations.
Paths and Long override Tree; machine serialization preserves the legacy Tree hierarchy.
Files also accepts explicit Tree.
Largest/recent still select regular files and retain their existing ranking and
automatic human rendering.

`Query` resolves whether a list request needs a tree or flat projection before
`report_in` reads the index.
Tree shaping uses the current two-level depth and ten-child limit; flat shaping defaults
to all rows and size order.
Legacy Files retains name order.
Keep `tree_node`, `expand`, and `child_rows` as the tree implementation rather than
reconstructing a tree from a globally truncated flat result.
Carry the originating view on tree sections so machine output can identify List without
changing Full’s legacy tree section.

The requested format is part of the read, not the cached scan basis.
In Python this is `Query(format=Format.LONG)` or `Query(format=Format.JSON)` before
requesting a report.
A detached Report owns only its requested projection.
Rendering it again preserves its selection, bounds, and reference clock; changing
serialization within that projection requires no query.
Converting a bounded tree report to a complete flat inventory requires another
retained-index report request.
Reject incompatible re-rendering with an actionable error instead of returning folded
tree rows as a complete list.
Do not retain or clone an Index inside Report and do not eagerly build a flat inventory
for an ordinary unfiltered tree.

### Subtree measurement and selected-content union

Add `query/query_subtrees.rs` and register it in `query.rs`. Its iterative `measure`
reader computes directory apparent/allocated bytes, descendant file/directory counts,
and newest eligible mtime.
Its `with_candidate` helper preserves native versus portable name identity, and `pruned`
applies exclusions before positive selection.
Apply ignored policy while traversing structural ancestors; empty eligible roots retain
their own timestamp.
Keep the index reducers and snapshot schema unchanged.

In `query_report::walk`, compute directory measurements before applying kind, name,
size, and age predicates.
Keep three distinct products: matching rows for flat output, regular-file members of the
selected-subtree union for grouped metrics, and visible directories/ancestors for tree
context. Propagate coverage from a matching directory, prune excluded descendants even
under coverage, and add each regular file once.
`metric_summary` consumes union members; `file_rows` consumes matches.
`child_rows` omits unrelated branches, and `expand` only reports depth folding when an
eligible directory child is hidden.

Extend `FileRow` with subtree counts where applicable and signed optional age, derived
from `Request::now`, not a renderer clock.
Keep inode attributes in native raw-entry projections.
Add a report-level reference instant for exact machine interpretation.
A future timestamp has negative age; a pre-epoch mtime remains valid; an unrepresentable
reference produces unknown age.
All directory bytes exclude symlink and directory-inode sizes, including mixed-kind
listings.

In `opened/read.rs`, update `report_work` and maintained-tree cost accounting for List’s
resolved projection and the extra subtree measurement traversal.
Preserve rejection before work when the budget cannot cover the read.
No producer, journal, continuation, cache identity, observer, or shutdown changes are
needed.

### Rendering and surface adapters

In `crates/fdu-core/src/report_format.rs`, extend `Format::parse` and `ALL` with Tree,
Paths, and Long and expose shared human/machine classification.
Tree reuses `render_text_tree` unchanged.
Paths emits escaped paths only, with no report headings or notes on stdout.
Long renders size, signed modification age, and path with consistent columns.
Use a checked rendering entry point to reject incompatible projections.

Update `section_json`, `file_json`, JSONL records, and YAML writers together with the
schema constants. List rows expose kind, both sizes, subtree counts, mtime, optional
signed age, and ignored classification; the envelope exposes the reference instant.
Keep completeness, freshness, provenance, and bounds.
Update `render_change` and `render_cache_status` dispatch for the added human formats,
with documented watch behavior rather than treating new human formats as machine output
by accident.

In `crates/fdu/src/cli.rs`, pass format through `ReadSpec` in request construction.
`Cli::render_watch_changes` keeps invalidations on stderr for Paths/Long; `render_live`
and the initial watch report preserve flat bounds and coverage notices on the same
diagnostic stream. Add `--tree` and `--long` aliases with explicit conflict checks.
Use core format classification in `machine_format`, normal output, cache-status, and
watch dispatch. Send path-only completeness/bound notices to diagnostics.
Update short and long help constants and clap field descriptions together.
CLI owns argument spelling and output streams; core owns selection, defaults,
compatibility, and rendering.

In `crates/fdu-py/src/lib.rs`, pass format through `build_read`/`build_request`, native
one-shot and retained report entry points, and watch report construction.
Use the core format parser instead of a separate vocabulary.
Update `PyOneShot::render` to use checked rendering and map rejection into the existing
Python error boundary.
In `python/fdu/_models.py`, extend Format, Query, and typed report/entry models; in
`_api.py`, forward Query’s format through `_query_kwargs` and preserve the report’s
chosen human presentation when rendering without an override.
Update native stubs and public exports as required.
Machine decoding must accept both tree and flat List sections rather than assuming the
view label alone determines shape.

### Tests and documentation inventory

Core tests in `query_report.rs`, `query_request.rs`, and `report_format.rs` cover the
metric/selection boundaries and presentation matrix.
Include all-kind selection, empty/nested matches, eligible directory/symlink activity,
ignored/excluded descendants, overlapping roots, both sizes, age edge cases, ordering,
bounds, mixed views, and detached-report re-render validation.
Retain iterative deep-tree tests.
Extend opened-read tests to prove budget rejection and portable matching; repeat queries
over one retained index after removing the backing fixture to establish no I/O.

Extend CLI integration/golden tests and `tests/parity/py/parity_cli.py` so both surfaces
send the same format at request time.
Add Python tests for typed List rows, age/reference values, repeated retained queries,
legacy presets, and incompatible combinations.
Run real cold, warm, and cache-only commands over a deterministic stale-build fixture.
Compare Paths, Long, JSON, JSONL, and YAML membership and metrics.
Keep the pre-change default tree golden unchanged; update only deliberate vocabulary and
schema changes. Never expand named golden patterns into local literals.

Documentation bead `fdu-2v7o` covers `README.md`, `docs/usage.md`, the portable CLI
skill, `crates/fdu-py/README.md`, Rust module/API docs, Python models/stubs, the
machine-output schema reference, design/surface architecture axes, and release notes.
Find every active reference to Files as the default inventory, Tree as the default view,
and the old four-format vocabulary; update or explicitly label compatibility usage.
Historical reports/specs remain historical.
Help bead `fdu-ia8p` executes the README and help examples against the same fixture and
covers individual and combined stale `.venv`, `venv`, `node_modules`, and Cargo `target`
searches.

Delivery bead `fdu-arv8` records reviewed golden differences, `make docs-format`,
`make check`, any required cross-lint, and CLI/Python end-to-end evidence in the
implementation PR. Update this plan PR with the map, then open the runtime PR above it
in stack #95 and wait for both PRs’ current heads to pass CI. The implementation PR
contains the final behavior, test evidence, remaining limitations, and links to this
complete spec; no implementation context is stored only in temporary files.

## Rollout and Review

Publish this plan as a documentation-only stack layer, link it from `TODO.md`, and
attach its path to the epic and child beads.
Keep the existing implementation work separate.
After the plan is reviewed, implement and review the six delivery steps in a subsequent
PR stacked above this plan.
Do not close issue #93 or the implementation beads merely because the plan PR is
published.

Checks run for this plan-only PR validate the existing implementation and document
formatting. The feature-specific acceptance tests above belong to the subsequent
implementation; publishing the plan does not claim those behaviors already work.

Before implementing public aliases or schema changes, inventory the released contracts
and write the compatibility matrix.
The settled requirements are the unchanged default output, default list/tree
equivalence, independent entry-kind filters, and explicit flat formats.
Exact schema version numbers and age-column formatting are implementation choices to
record with their tests; they must not weaken these requirements.

## Implementation Review and Validation

The implementation keeps query semantics and format validation in `fdu-core`. The CLI
and Python package pass the same request and use the same renderers.
Directory metrics are computed by an iterative post-order pass before positive
predicates; grouped totals consume the covered regular-file union once.
Default Tree continues to use maintained roll-ups without materializing a flat
inventory. A complete flat read pays for its measurement/selection work in the
opened-root budget. No cache identity, snapshot encoding, retained owner, dependency, or
content-read policy changes are required.

Precommit review covered the implementation layer above PR #96, including Rust request,
selection, formatting, Python models/bindings, golden updates, and current
documentation. Verdict: implementation ready for the full validation gate; no unresolved
code findings. Review fixes include deterministic clocks confined to test helpers,
retaining legacy Tree JSON, rejecting incompatible detached re-rendering, non-mutating
age normalization that still detects incorrect ages, and exposing cache/scope notices
separately from flat stdout.
There are no deferred correctness findings.

The default overview golden is unchanged.
Focused validation passed all Rust workspace tests (757 core tests passed, one existing
ignored test), CLI integration tests, and the existing 165 CLI golden sessions.
Python validation caught and fixed compatibility with hand-built rows lacking the new
optional fields. New shared golden sessions exercise old `.venv`, `node_modules`, and
`target` roots, a recent descendant, and empty matches.
Final gate and CI evidence will be recorded on the implementation PR before completion.

## References

- [Issue #93: directory roll-up queries](https://github.com/jlevy/fdu/issues/93)
- [Design principles](../../architecture/fdu-design-principles.md)
- [Engine architecture](../../architecture/fdu-engine-architecture.md)
- [Surface architecture](../../architecture/fdu-surface-architecture.md)
- [Current usage guide](../../../usage.md)
- [View vocabulary and output contract](plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
