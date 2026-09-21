# Machine Output and Directory Inventories

Reports use `fdu.report/7` for metadata-only sections and `fdu.report/8` when content
analysis or a generic metric summary is present.
Cache status remains `fdu.cache/2`, and raw watch changes remain `fdu.stream/1`. Check
the schema before decoding.

## List Rows

```shell
fdu ~/projects --kind dir --include node_modules --modified-before 30d --format json
```

The default List in JSON, JSONL, or YAML exposes complete matching rows unless an
explicit limit bounds them.
A flat section has `view: list`, a `files` array (the existing row collection name), and
`bound`, which is null when no rows were omitted.
A bounded section gives shown/total counts.
Full retains its bounded digest; the legacy `--view tree --format json` preset retains
the `tree` hierarchy and its per-node `truncated` flags.
A Python List requested in Tree format also serializes its stored tree projection as a
`tree` object, so inspect the payload key as well as `view`.

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
| `ignored` | Boolean classification, or null when `.gitignore` was not observed |

The envelope’s `age_reference_ns` is the fixed request instant in epoch nanoseconds.
When representable, `age_ns = age_reference_ns - mtime_ns`. Re-rendering a Report never
samples another clock.
`generated_at` describes report generation and `scan_started_at` is the conservative
incremental-sync watermark; neither should be substituted for the age reference.
Signed age can exceed an i64 even though each timestamp fits one.
Use an integer-preserving parser; nanosecond values exceed JavaScript’s exact binary64
integer range.

Directory metrics apply exclusions throughout the subtree before positive name/kind,
size, and time predicates.
They include directory and symlink activity but no directory inode or symlink bytes.
Nested matching roots may overlap; summary/grouped totals count the covered union once.
These are modification times and counted bytes, not last-use or uniquely
reclaimable-space claims.
Native entry/lookup APIs retain inode metadata.

## Coverage and Formats

Inspect `complete`, `errors`, `freshness`, `source`, ignore-rule coverage, and section
bounds independently.
Display folding is not incomplete scanning; an incomplete scan cannot establish a whole
subtree’s absence or size.
A directory row says so itself: `complete` is false at a `--scan-depth` boundary, for a
directory an opened root has not listed yet, and throughout a scan that finished with
errors, which records that the walk was partial but not where.
Such a row’s sizes and counts are lower bounds, its `age_ns` is null, and no
`modified_since`/`modified_before` bound matches it, because a lower-bound maximum is
not an age; `min_size` still can, since a lower bound at or above the minimum proves the
true size is too.
A cache-only result is stale rather than incomplete: a snapshot is only
written from a complete index, so its rows stay complete and `source`/`freshness` carry
the staleness. Content metric coverage remains separate from metadata completeness.

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
Cache-status human aliases render its existing table.
New List watch requests repaint snapshots.
Legacy Files watch requests retain raw change streams with Text or machine formats;
explicit Tree, Paths, and Long repaint snapshots instead.

See [the usage guide](usage.md) for filter grammar and examples, and
[the surface architecture](project/architecture/fdu-surface-architecture.md) for schema
ownership and parity validation.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
