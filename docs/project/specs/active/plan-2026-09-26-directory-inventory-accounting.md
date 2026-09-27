# Plan: Directory Inventory and Accounting Examples

**Date:** 2026-09-26 (last updated 2026-09-26)

**Status:** Implemented assessment and examples above PR #133

**Tracking:** fdu-qvu3 (issue assessment), fdu-qqd2 (public examples), fdu-6u68
(hard-link audit)

## Decision

[Issue #93](https://github.com/jlevy/fdu/issues/93) is implemented by merged
[PR #117](https://github.com/jlevy/fdu/pull/117), which carried the directory query work
from superseded drafts #96 and #103. This cycle needs a documentation and validation
layer above PR #133, with no new query or cache behavior.
Explain two ways to list matching build directories and tally their usage, and show code
analysis on the fdu repository.
Issue #93 can be closed as implemented after review.
Keep hard-link accounting as its own design question.

## Current Contract and Evidence

A matched directory is a query result with apparent and allocated subtree bytes,
descendant file and directory counts, and newest modification activity.
Size and time predicates test these eligible subtree values.
Excludes apply before measurement.
Nested matching roots each appear in a flat list, while Summary counts their covered
path union once. An incomplete subtree reports lower-bound sizes and an unknown age; it
cannot satisfy a time bound.

This is implemented in `query_subtrees::measure` and `query_report::walk`. Focused tests
in `query_report.rs` cover overlapping directory matches
(`matching_a_directory_selects_its_subtree_once`), excluded contents and ignored
selection (`exclusions_apply_before_subtree_bounds_and_selected_ancestor_coverage`), and
unknown ages at scan-depth boundaries.
The `cli-axes` golden exercises `.venv`, `node_modules`, and `target` paths and long
output. Reuse the existing directory-query plan for the detailed semantics rather than
restating it here.

A local macOS fixture at revision `6731aad9` had four regular files under `.venv`,
`node_modules`, and `target`, including a nested `node_modules`. Its flat matching-root
rows summed to 36 KiB; the Summary was 32 KiB across four files and five directories.
The difference is the nested matched root counted twice in the row sum, once in Summary.
These are fixture observations, not a performance measurement.

## User-Facing Workflows

For a human-readable inventory with size and age per matching directory, then one
deduplicated total from the same snapshot:

```shell
fdu ROOT --cache refresh --kind dir --include .venv \
  --include node_modules --include target --sort size --long
fdu ROOT --cache only --kind dir --include .venv \
  --include node_modules --include target --view summary
```

Use the same root, cache directory, scope, size choice, and ignored population in both
requests.
The first command scans and publishes the snapshot; the second reads it without
another filesystem walk.
Label its result as cache-only and not revalidated.
Default size is allocated; add `--size apparent` to both commands when logical lengths
answer the question.

For one invocation with complete flat directory details and a deduplicated total, use
the existing machine contract:

```shell
fdu ROOT --kind dir --include .venv --include node_modules \
  --include target --view files,summary --sort size --format json
```

The `files` rows expose path, both byte measures, counts, completeness, and age;
`summary` is the selected path union.
`files` text intentionally emits one path per line, and `long` currently requires one
list section.
A text `tree,summary` report can display size-bearing rows and a total, but
its hierarchy and display bounds are a different presentation.

The README example was captured at revision `6731aad9` with
`fdu . --analyze=code --ignored=exclude --limit=5`. It records 119,153 code lines, with
five displayed rows and explicit coverage.
The performance footer is omitted; no benchmark claim is made.
Shares include all measured languages, including those outside the displayed rows.

## Accounting Decision

**Directory-root deduplication is not hard-link deduplication.** The current Summary
sums eligible file *paths* once.
Two paths with the same device and inode contribute twice.
On a current macOS fixture, two hard links under separate `.venv` roots each reported
16,384 allocated bytes and Summary reported 32,768 bytes, while both paths had the same
device and inode. State this explicitly near any claim of a “deduplicated” inventory
total.

Hard links, copy-on-write clones, and reclaimable space are different measures.
[uv’s link-mode reference](https://docs.astral.sh/uv/reference/settings/#link-mode)
currently documents clone as the default on macOS and Linux, hardlink on Windows, with
configurable copy and symlink modes.
Its [cache guide](https://docs.astral.sh/uv/concepts/cache/#cache-directory) says
cross-filesystem placement can force a copy fallback.
Thus neither directory-root union nor a future device/inode union proves bytes freed by
deleting a directory; clones can share extents under different inodes, and links outside
the selected root can retain blocks.
On Windows, the current scanner reports apparent bytes as allocated and does not query
allocated blocks (`scan/windows_metadata.rs`); label that platform limitation.

Do not alter ordinary rollups in this documentation layer.
The existing fdu-579b hard-link attribution design gate and fdu-8ybz durable checkpoint
work own any unique-identity metric.
The [disk-usage checkpoint plan](plan-2026-09-13-fdu-disk-usage-checkpoints.md) already
separates per-path allocated, unique allocated, and free-space observations.
A future capability must retain sound link identity, preserve incremental attribution
under add/remove/rename, disclose that clone/shared-extent allocation remains
unobserved, and return unknown where identity or physical allocation is unavailable.
It must not label unique-inode bytes as reclaimable bytes.

## Implementation Checklist

- [x] In README, add a measured fdu code-analysis example with commit/scope, output,
  code-line share denominator, coverage note, and usage link
- [x] In README or the usage guide, add the three-name inventory examples: long then
  cache-only Summary, and one-command JSON `files,summary`
- [x] Explain nested matching-root overlap, path-union Summary, hard-link double
  counting, and cache-only freshness beside those examples
- [x] Check the examples against the candidate CLI; preserve the shared golden corpus
  and record final gate/CI evidence on the stacked PR
- [x] Record the hard-link audit under fdu-6u68 and leave unique accounting to
  fdu-579b/fdu-8ybz

## Optional Later Presentation Slice

Only if users require *flat human size/age rows and Summary in one command*, allow
`long` with exactly one flat list section plus Summary.
The smallest source map is shared request validation in `query_request.rs`, projection
checking and flat rendering in `report_format.rs`, then one full-output golden and
CLI/Python parity. Keep `paths` single-list for pipes and leave `files` text as path
enumeration. Test either section order, empty rows, nested overlap, incomplete ages,
escaped paths, and invalid grouped mixes.
This is not a prerequisite for closing #93.

## References

- [Issue #93](https://github.com/jlevy/fdu/issues/93) and merged
  [PR #117](https://github.com/jlevy/fdu/pull/117)
- [Directory-query design](plan-2026-09-20-directory-query-formats.md)
- [Disk-usage checkpoint plan](plan-2026-09-13-fdu-disk-usage-checkpoints.md)
- [uv link modes](https://docs.astral.sh/uv/reference/settings/#link-mode) and
  [uv cache placement](https://docs.astral.sh/uv/concepts/cache/#cache-directory)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
