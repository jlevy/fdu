---
type: is
id: is-01m2y7b2f9f1cv1dssrtref7zd
title: Directory filtering and list presentation formats for stale build inventories (#93)
kind: epic
status: in_progress
priority: 1
version: 17
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
delegate: claude-code@spud10
labels: []
dependencies: []
child_order_hints:
  - is-01m2ykpygq70t4j60rw8tnjm22
  - is-01m2y7by0xkr9zre4es53fwjm0
  - is-01m2y7c1v3etw9wkzz87vgm6ke
  - is-01m2yhsw73csne6aef665mmp4n
  - is-01m2y7c6yzccahx1ntvy2wf1s1
  - is-01m2y7cbprn6w292gjenv0twd7
  - is-01m2y7cf9fsawdtq8p5grnr3nq
hold: null
hold_until: null
created_at: 2026-09-20T01:36:54.755Z
updated_at: 2026-09-20T05:32:36.416Z
started_at: 2026-09-20T01:38:30.390Z
---
Plan: `docs/project/specs/active/plan-2026-09-20-directory-query-formats.md`. Publish the plan separately on `codex/directory-query-plan`, stacked on the latest branch, PR #94 (`perf/campaign-linux-2026-09-19`) above #92. Implementation follows in a separate PR above the reviewed plan. The tracked spec owns the complete design and implementation breakdown.

## Problem and Goal

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

Include oldest-first sorting, both size metrics, ignored inventory, multiple queries
over one retained index, tree expansion, and machine-readable size/age extraction.
Explain nested overlaps, exclusion effects, modification age versus last use,
completeness, and non-unique disk accounting.
Do not include deletion commands.

## Acceptance and Delivery

Deterministic tests cover empty and nested roots, fresh file/directory/symlink activity,
future and pre-epoch timestamps, unknown ages, half-open age/size bounds, descendant
exclusions, ignored policies, both sizes, stable sorting, tree context, and union
aggregation. Verify repeated queries perform no filesystem work, portable names,
deep-tree stack safety, opened budgets, and mixed views.

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
Review, commit, push, open/update an implementation PR above the reviewed plan layer, and watch CI to
completion. Close finished beads and sync after passing CI. No dependency changes or
performance claims are planned.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->

## Notes

The full design and all six implementation steps are tracked in docs/project/specs/active/plan-2026-09-20-directory-query-formats.md and published as plan-only PR https://github.com/jlevy/fdu/pull/96, stacked on #94 above #92/#91. Publication task is fdu-79n0. Preserve default output. Implementation remains separate in the original worktree and should form a subsequent layer above the reviewed plan; do not close implementation beads or issue #93 upon plan publication.
