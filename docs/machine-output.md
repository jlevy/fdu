# Machine Output and Directory Inventories

All reports use `fdu.report/10`, including metadata-only and content-analyzed reports.
Cache status uses `fdu.cache/3`, and raw watch changes use `fdu.stream/2`. Check the
schema before decoding.
The [schema rule](project/guides/release-process.md) requires a new version when a
published shape changes.
Content analysis does not select another schema.
`request.analyze` lists the analyzers the request enabled, whether `--analyze` named
them or a content view implied them, so `--view=code` and `--analyze=code` serialize one
request; `analysis.analyze` and the content sidecar identity follow it.
`request.views` and `request.omitted_views` are the views shown and the ones `full` had
to drop.

## Recursive Trees and Hidden Content

For a full recursive roll-up, use `--view tree --full` without a scan-depth restriction.
Fully observed trees then have `remainder: null` and no omission records.
Check `status.complete` independently: unrestricted display cannot repair an unreadable
directory or incomplete discovery.

Each bounded tree section includes a shared `remainder`: recursive `files`, apparent
`bytes`, `allocated` bytes, and applicable `reasons` for usage outside the displayed
root-level rows. A displayed directory represents its whole subtree, even when its
children are hidden by a display bound; those children do not enter the section
remainder again. Unknown totals are `null`, never zero.
Text prints the same size and root share in the remainder row’s columns, followed by
`… and N more files`. The section’s `limits` give exact bound values.
Per-node `omissions` retain detailed first-exclusion boundaries: `entries` counts direct
hidden roots, while `files` counts regular files throughout those hidden subtrees.
The root’s direct rows and section remainder partition its selected usage; never sum
parent and child directory totals.
See the [output design system](project/architecture/fdu-output-design.md).

## List Rows

```shell
fdu ~/projects --kind dir --include node_modules --modified-before 30d --format json
```

The default List in JSON, JSONL, or YAML exposes complete matching rows unless an
explicit limit bounds them.
A flat section has `view: list`, a `files` array, and `bound`, which is null when no
rows were omitted. A bounded section gives shown/total counts.
`--view tree --format json` gives the `tree` hierarchy with explicit limits and
omissions. A Python List requested in Tree format also serializes its stored tree
projection as a `tree` object, so inspect the payload key as well as `view`.

| Field | Meaning |
| --- | --- |
| `path`, optional `path_raw` | Relative path and lossless platform-tagged identity for non-Unicode paths |
| `kind` | `file`, `dir`, `symlink`, or `other` |
| `bytes` | Apparent bytes; a directory sums eligible regular-file contents |
| `allocated` | Allocated bytes under the same accounting |
| `files`, `dirs` | Directory descendant counts, excluding the matching root; null for other kinds |
| `complete` | Whether a directory’s eligible subtree was listed in full; false makes `bytes`, `allocated`, `files`, `dirs`, and `mtime_ns` lower bounds and `age_ns` null; null for other kinds |
| `mtime_ns` | Signed epoch nanoseconds; a directory uses the newest eligible root/descendant timestamp |
| `age_ns` | Signed age at `age_reference_ns`; negative for a future timestamp, null if the reference cannot be represented or the subtree is incomplete |
| `ignored` | Boolean classification, or null when rules were unobserved or governing classification is unknown |
| `sort_value` | Value of the requested content sort metric, or null when absent/unavailable |
| `classification` | For a regular file: stable file type, family, detection source/confidence, and generated/vendor/documentation flags; null for other kinds |

The request’s nullable `sort_metric` identifies the content metric used to rank rows.
Unavailable values sort last in either direction, with deterministic path ties.

The envelope’s `age_reference_ns` is the fixed request instant in epoch nanoseconds.
When representable, `age_ns = age_reference_ns - mtime_ns`. Re-rendering a Report never
samples another clock.
`provenance.generated_at` describes report generation and `provenance.scan_started_at`
is the conservative incremental-sync watermark; neither should be substituted for the
age reference. Signed age can exceed an i64 even though each timestamp fits one.
Use an integer-preserving parser; nanosecond values exceed JavaScript’s exact binary64
integer range.

Directory metrics apply exclusions throughout the subtree before positive name/kind,
size, and time predicates.
They include directory and symlink activity but no directory inode or symlink bytes.
Nested matching roots may overlap; summary/grouped totals count the covered union once.
These are modification times and counted bytes, not last-use or uniquely
reclaimable-space claims.
Native entry/lookup APIs retain inode metadata.

Summary, tree, and extension rows report `ignored` as null when their contributing scope
includes an entry whose governing `.gitignore` rules could not be verified.
Known sibling tree rows retain their exact ignored subtotals.
The report notes this condition; null never means zero ignored entries.

## Tree Bounds

Each tree section states `limits`: `depth`, `min_share`, `breadth`, and `rows`. A null
integer bound means unlimited; `min_share` is an exact percentage string.
The default is depth 5, share `1%`, and unlimited breadth and rows.
`tree` is null when no data row is admitted, including `--limit=0`. Each tree node has
`entry_ignored`: `true` or `false` for that entry’s own `.gitignore` classification, or
null when the governing controls were not observed or could not be verified.
This remains independent of the node’s selected-subtree `ignored` tally, including for
empty directories and zero-byte files.

Sections and nodes carry `omissions`. Each item names `reason` (`share`, `breadth`,
`depth`, or `rows`), the number of direct child roots omitted in `entries`, the
recursive regular-file count in `files`, and the `bytes`/`allocated` remainder.
Counts and sizes are null when unknown.
Each omission belongs to its first excluding bound, so these disjoint child subtrees can
be counted without double counting.
The root denominator and aggregate totals are unchanged by display bounds.

## Code Overview

`--view=code`, and `--analyze=code` with no view, give a `view: code` section with a
`code` object. It states `population`, `share_metric: code_lines`, `analyzed_languages`,
`selected` totals, `non_ignored` and `ignored` population totals when available, an
`unknown` tally, `unclassified_files`, and the complete `languages` table unless an
explicit display bound applies.
`unclassified_files` counts selected regular files whose detected family is unknown;
known document, data, and other non-code families are excluded from that count.
Each language row includes its selected and population totals and an exact share
fraction.
Code, grouped metric, and extension sections expose `share_omitted`: the number
of rows removed by an explicit `min_share` before the section row limit.
Their `bound.total` counts rows eligible after that filter and before the row limit.
Thus the number of groups before either display bound is `bound.total + share_omitted`
when `bound` is present, or the displayed row count plus `share_omitted` when it is
null. Use `--min-share 0%` to restore share-filtered rows and `--limit all` to restore
rows removed by a row cap.
Selected aggregate totals stay unchanged by both display bounds.
A grouped metric section’s share denominator is the sum of every row’s numerator before
either bound, which is also the total row’s share.
For `document_words` it can differ from the total row’s own `document_words` and
`pages.words`, in either direction: logical words are derived after pooling, so a pooled
total of mixed formats is not the sum of its rows.
Each grouped metric row, and its total row, carries a `detection` object: `sources` and
`confidence` count its files by how their type was detected, and `flags` counts its
files detected as `generated`, `vendored`, or `documentation` (under a `doc`, `docs`, or
`documentation` directory, or named like a README, CHANGELOG, or CONTRIBUTING file).
Human rows show the generated and vendored counts but not the documentation count.

A code tally has `source_files`, `analyzed_files`, `code_lines`, `comment_lines`,
`blank_lines`, `missing_records`, and a `coverage` reason map.
A counted zero is distinct from an unsupported analyzer or a missing record.
Language shares use measured selected code lines; inspect coverage before treating that
denominator as complete.

`request.scope.population` states the retained discovery population.
A one-shot derives it from `--ignored`; a narrowed query over a broad retained index may
hold more facts.
The cache entry identity includes population and the control fingerprint
that governs pruned admission.
Cache status reports these fields too.

The envelope’s `ignore_rules` is null when controls were not observed.
Otherwise it states `applied` files, accepted `rules` counted per governing location,
`refused` files, limits, and bounded refusal details.
A refusal names its rule file by the path git opens, `<dir>/.gitignore`, even where a
case-insensitive volume stores that file as `.GITIGNORE`. These counts describe retained
control state and do not imply every rule file was reread on a cache-only run.

## Coverage and Formats

Inspect `status.complete`, `status.coverage`, `status.errors`, and
`status.errors_omitted` separately from `provenance.freshness`, `provenance.source`,
per-tier provenance, ignore-rule coverage, and section bounds.
Display folding is not incomplete scanning; an incomplete scan cannot establish a whole
subtree’s absence or size.
A directory row says so itself: `complete` is false at a `--scan-depth` boundary, for a
directory an opened root has not listed yet, and for a failed subtree or an ancestor of
one. A partial cold scan retains completeness for successfully listed siblings; an
unscoped failure leaves the whole tree incomplete.
Such a row’s sizes and counts are lower bounds, its `age_ns` is null, and no
`modified_since`/`modified_before` bound matches it, because a lower-bound maximum is
not an age; `min_size` still can, since a lower bound at or above the minimum proves the
true size is too. A cache-only result carries staleness in report provenance and
preserves the snapshot’s directory completeness.
A snapshot is published only from a complete scan, but an unlisted directory at its
scan-depth boundary still has `complete: false` and unknown age.
Content metric coverage remains separate from metadata completeness.

Tree and flat List are different report projections over the same selection.
Set format on `ReadSpec`/`Query` before reading.
A detached Report can change serialization, and a flat one can alternate Paths and Long.
Changing a folded tree into a complete flat inventory requires another report from the
retained index. Rust rendering returns a Result; Python raises InvalidArgumentError for
incompatible conversion.

Paths contains one path per line, control characters escaped and everything else, the
separator and a literal backslash included, verbatim; it is lossy, so byte identity
lives in `path_raw` here.
Long contains size, signed human age, and path.
They omit the performance footer; CLI bound, rule, coverage, cache-only, and watch
invalidation notices go to stderr.
The exact nanoseconds and native path identity belong to machine output.
Automatic Text and machine formats support grouped/mixed views; explicit Tree/Paths/Long
require one compatible list section.
Largest/recent retain their file ranking and support Paths and Long.
Cache-status human aliases render its table.
A List watch repaints snapshots.
A Files watch streams raw changes in Text or machine formats; explicit Tree, Paths, and
Long repaint snapshots instead.

See [the usage guide](usage.md) for filter grammar and examples, and
[the surface architecture](project/architecture/fdu-surface-architecture.md) for schema
ownership and parity validation.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
