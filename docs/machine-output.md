# Machine Output and Directory Inventories

All reports use `fdu.report/12`, including metadata-only and content-analyzed reports.
Cache status uses `fdu.cache/3`, and raw watch changes use `fdu.stream/3`. Check the
schema before decoding.
The [schema rule](project/guides/release-process.md) requires a new version when a
published shape changes.
Content analysis does not select another schema.
`request.analyze` lists the analyzers the request enabled, whether `--analyze` named
them or a content view implied them, so `--view=code` and `--analyze=code` serialize one
request; `analysis.analyze` and the content sidecar identity follow it.
`request.views` and `request.omitted_views` are the views shown and the ones `full` had
to drop. `request.time` names the one clock every time in the report reads: `changed`,
the default, the newer of modification and status change, or `modified`. `selected` and
`scanned` are on every report, each `{files, dirs, bytes, allocated}` over every root:
`selected` is what a summary of the same selection reports, the files the selection
admits or covers, and `scanned` is everything the scan retained.
They are equal when the selection filters nothing.
Under `--ignored=exclude` or `--ignored=only` the scan itself keeps only that part of
the tree, so `scanned` is what that scan retained, not the whole tree, and the note
names the scan:
`note: selected 12 of the 4,296 files an --ignored=exclude scan retained, …`. A
selection by the same ignored state the scan applied narrows nothing it retained.
A filtered tree’s root row is 100% of `selected`, not of the tree; text reports say so
in `note: selected 2,951 of 27,060 files, 704 MiB of 1.8 GiB` when the two differ.

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
fdu ~/projects --kind dir --include node_modules --before 30d --format json
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
| `time_ns` | The entry’s time on the report’s clock (`request.time`), in signed epoch nanoseconds; a directory uses the newest eligible root/descendant time |
| `files`, `dirs` | Directory descendant counts, excluding the matching root; null for other kinds |
| `complete` | Whether a directory’s eligible subtree was listed in full; false makes `bytes`, `allocated`, `files`, `dirs`, and its times lower bounds and `age_ns` null; null for other kinds |
| `age_ns` | Signed age of `time_ns` at `age_reference_ns`; negative for a future time, null if the reference cannot be represented or the subtree is incomplete |
| `time_at` | `time_ns` as an RFC 3339 UTC instant to the nanosecond, or null when the subtree is incomplete and `time_ns` only a lower bound |
| `mtime_ns` | Modification time in signed epoch nanoseconds, whatever the report’s clock; a directory uses the newest of the entries `time_ns` counts |
| `changed_ns` | Change time, the newer of modification and status change, in signed epoch nanoseconds, whatever the report’s clock; a directory uses the newest of the entries `time_ns` counts; null for a regular file whose status change was not observed |
| `ignored` | Boolean classification, or null when rules were unobserved or governing classification is unknown |
| `sort_value` | Value of the requested content sort metric, or null when absent/unavailable |
| `classification` | For a regular file: stable file type, family, detection source/confidence, and generated/vendor/documentation flags; null for other kinds |

The request’s nullable `sort_metric` identifies the content metric used to rank rows.
Unavailable values sort last in either direction, with deterministic path ties.
`--sort time` orders rows by `time_ns`, newest first.
On the change clock, a file whose status change was not observed is timed by its
modification time, a lower bound of its change time, while its `changed_ns` stays null.
With both raw clocks, one listing finds what no window can: an arrival that kept old
content times is a row whose `changed_ns` is recent and whose `mtime_ns` is old.

The envelope’s `age_reference_ns` is the request instant every age is measured from, in
epoch nanoseconds, and `age_reference_at` is the same instant as RFC 3339 UTC; both are
null when it cannot be represented.
When representable, `age_ns = age_reference_ns - time_ns`. Re-rendering a Report never
samples another clock.
A one-shot report samples the instant before its walk, so a file written during the walk
can show a small negative age.
A watch measures each repaint from that repaint’s own instant, so ages stay current over
a long session, while the time windows it selects by stay the absolute bounds its
request resolved when it was built.
`provenance.generated_at` describes report generation and `provenance.scan_started_at`
is a conservative lower bound for a later `--since` selection, not a complete change
log: deletions are not selected, and under `--time modified` neither are files copied or
restored with older modification times.
Neither should be substituted for the age reference.
Signed age can exceed an i64 even though each timestamp fits one.
Use an integer-preserving parser; nanosecond values exceed JavaScript’s exact binary64
integer range.
Each RFC 3339 field is the readable twin of an exact `*_ns` field, never a
replacement for it.

Directory metrics apply exclusions throughout the subtree before positive name/kind,
size, and time predicates.
They include directory and symlink activity but no directory inode or symlink bytes.
Nested matching roots may overlap; summary/grouped totals count the covered union once.
These are activity times on the report’s clock and counted bytes, not last-use or
uniquely reclaimable-space claims.
Native entry/lookup APIs retain inode metadata.

Summary, tree, and extension rows report `ignored` as null when their contributing scope
includes an entry whose governing `.gitignore` rules could not be verified.
Known sibling tree rows retain their exact ignored subtotals.
The report notes this condition; null never means zero ignored entries.

## Tree Nodes

A tree section’s `tree` is its root node, and every node lists its `children` in the
section’s sort order.

| Field | Meaning |
| --- | --- |
| `name` | Final path component, or `.` for the root |
| `path`, optional `path_raw` | Relative path, empty for the root, and lossless platform-tagged identity for non-Unicode paths |
| `kind` | `dir` or `file` |
| `entry_ignored` | This entry’s own `.gitignore` classification, or null when its governing controls were not observed or could not be verified |
| `bytes`, `allocated` | Apparent and allocated bytes of the regular files the row counts |
| `files`, `dirs` | Regular files and directories the row counts beneath it; a file counts itself |
| `ignored` | The part of those tallies `.gitignore` rules ignore, or null when governing rules were unobserved or unverified |
| `newest_file_time_ns` | Newest time, on the report’s clock, among the regular files the row counts, or null when it counts none |
| `time_ns` | Newest time, on the report’s clock, among every entry the row counts, of any kind; null when it counts none |
| `complete` | Whether a directory’s subtree was listed in full; false makes its tallies and `time_ns` lower bounds; null for a file |
| `age_ns` | Signed age of `time_ns` at `age_reference_ns`; null when the row counts nothing, its subtree is incomplete, or the reference cannot be represented |
| `time_at` | `time_ns` as an RFC 3339 UTC instant to the nanosecond; null when `time_ns` is null or only a lower bound |
| `truncated` | Whether display bounds withheld any child |
| `omissions` | First-exclusion records for the children display bounds withheld ([below](#tree-bounds)) |
| `children` | The rows displayed beneath this one |

A row counts its own entry when the selection admits it, and every entry beneath it that
its tallies count. The report root is a traversal boundary, never an admitted entry, so
its own time never counts.
`time_ns` includes directory and symlink activity: deleting or renaming a file changes
no surviving file’s time but does change its directory, and an install that preserves
archive times, as npm and `tar` do, is dated by its directories even under
`--time modified`. Under a selection that admits everything, a directory’s `time_ns` is
its list row’s, so `--long` and the tree agree; under a filter each follows its own
row’s population. `newest_file_time_ns` keeps its files-only meaning, which a summary
reports too. `--sort time` orders tree rows by `time_ns`, the age the text column shows.
An omission or remainder carries no time.

## Tree Bounds

Each tree section states `limits`: `depth`, `min_share`, `breadth`, and `rows`. A null
integer bound means unlimited; `min_share` is an exact percentage string.
The default is depth 5, share `1%`, and unlimited breadth and rows.
`tree` is null when no data row is admitted, including `--limit=0`. A node’s
`entry_ignored` remains independent of its selected-subtree `ignored` tally, including
for empty directories and zero-byte files.

Sections and nodes carry `omissions`. Each item names `reason` (`share`, `breadth`,
`depth`, or `rows`), the number of direct child roots omitted in `entries`, the
recursive regular-file count in `files`, and the `bytes`/`allocated` remainder.
Counts and sizes are null when unknown.
Each omission belongs to its first excluding bound, so these disjoint child subtrees can
be counted without double counting.
The root denominator and aggregate totals are unchanged by display bounds.

## Several Roots

A report over several roots (`fdu docs src`, or `fdu.report([...])` in Python) has the
same schema, with these differences; a report over one root has none of them.

- The envelope’s `root` is null, and `roots` lists each root in the caller’s order:
  `label`, the path as given and normalized by its components (with `label_raw` when it
  is not Unicode), and `path`, the canonical directory (with `path_raw`). Exactly one of
  `root` and `roots` is set.
- List rows, `status.errors` items, and `ignore_rules.refusals` items begin with `root`,
  the position of their root in `roots`. Their `path` stays relative to that root, so
  `roots[root].path` joined with `path` names the entry, and an absolute label never
  makes a relative path absolute.
- A tree section has `tree: null`, then `total`, then `trees`. `total` is the merged
  root row: the tree node fields from `bytes` through `time_at`, with no `name`, `path`,
  or `kind`, since no entry is the total; it is null when the row limit is zero.
  Its `time_ns` is the newest of the roots’ and `complete` holds when every root is.
  `trees` holds, in display order, `{root, tree, remainder}` for each root the row limit
  kept: `tree` is that root’s ordinary tree, its root node named by the label with an
  empty `path`, and `remainder` what its own bounds hid.
  The section’s `omissions` and `remainder` describe the root rows the row limit cut.
- Every share and bound is over the union: a row’s share is of `total`, and the row
  limit counts the total as the first row.

A root node never counts its root’s own time, as a single report’s root does not, so its
sizes, counts, `time_ns`, and `complete` equal those of the root node of that root’s own
report. Its `name` is the root’s label, and its children are those whose share of the
combined total reaches the threshold, so it can show fewer rows than that report does.

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

## Age Profiles

An `ages` section has `bands` and `total`, by age on the clock `request.time` names.
`bands` always holds all nine bands, newest first, empty ones included: `future`,
`under 1h`, `1h to 1d`, `1d to 7d`, `7d to 30d`, `30d to 90d`, `90d to 1y`, `1y to 3y`,
and `over 3y`.

| Field | Meaning |
| --- | --- |
| `band` | The band’s label |
| `since_age_ns`, `before_age_ns` | The band’s window as ages at `age_reference_ns`: the files `--kind file --since <since_age> --before <before_age>` selects; null where the window is open |
| `since_ns`, `before_ns`, `since_at`, `before_at` | The same edges as epoch nanoseconds and RFC 3339 instants |
| `files`, `bytes`, `allocated` | The regular files in the band |
| `ignored` | The band’s gitignored files, bytes, and allocated bytes, or null when rules were unobserved or a counted file’s classification is unknown |
| `cumulative` | The same tallies for this band and every newer one |

`total` holds the same tallies over every band: what `summary` reports for the same
selection.

## Volumes

A `volumes` section has `observed_at_ns` and `observed_at`, when capacity was read, and
`volumes`, one row per filesystem the report touches, ordered by its path.

| Field | Meaning |
| --- | --- |
| `device` | The device the walk identified the filesystem by |
| `root`, `path`, optional `path_raw` | Where the report entered the filesystem, relative to its root; the root itself is an empty path |
| `other_paths` | How many other paths the report entered it at |
| `filesystem` | The type by name, or on Linux any type but `btrfs` and `zfs` as its number |
| `capacity_bytes`, `available_bytes` | Size and the bytes an unprivileged writer can still use; null when unknown |
| `shares_pool` | Whether available space is shared with other volumes of an APFS container, btrfs filesystem, or ZFS pool |
| `unknown_reason` | Why capacity is unknown, or null |
| `counted` | The regular files this report counted on the filesystem: `files`, `bytes`, `allocated` |

Capacity is observed when the report is read and never stored.
No row adds to another, and `counted` never sums to what is in use.

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
Such a row’s sizes and counts are lower bounds, its `age_ns` is null, and no `since` or
`before` bound matches it on either clock, because a lower-bound maximum is not an age;
`min_size` still can, since a lower bound at or above the minimum proves the true size
is too. A cache-only result carries staleness in report provenance and preserves the
snapshot’s directory completeness.
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
A Files watch first writes an ordinary report (`fdu.report/12` in machine formats), then
streams raw changes (`fdu.stream/3`) in Text or machine formats; explicit Tree, Paths,
and Long repaint snapshots instead.
Aggregate views also repaint reports.
An `upsert` gives both raw clocks, `mtime_ns` and, where the entry’s status change was
observed, `changed_ns`, the time a window on the default clock admitted it by; a
`remove` or `invalidate` gives neither.
An `invalidate` change means incremental state is no longer sufficient: rebuild or
resynchronize the inventory before relying on subsequent changes.
A consumer must handle the initial report and the change-record schema separately.
Creating, removing, or renaming an entry also moves its directory’s own modification
time, so a Files watch follows that change with an `upsert` record for the directory,
unless it is the scan root or the selection leaves directories out.
It is an ordinary `fdu.stream/3` record.

See [the usage guide](usage.md) for filter grammar and examples, and
[the surface architecture](project/architecture/fdu-surface-architecture.md) for schema
ownership and parity validation.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
