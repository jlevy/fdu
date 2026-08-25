---
type: is
id: is-01kzqn502680awzhvddzntq32d
title: "P3: watch scope validation errors"
kind: task
status: open
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47
    at: 2026-08-25T09:54:14.887Z
  - kind: pr
    url: https://github.com/jlevy/metabrowser/pull/74
    at: 2026-08-25T09:54:14.888Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5017522830
    at: 2026-08-25T09:57:37.843Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019372007
    at: 2026-08-25T13:21:13.810Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-11T05:36:29.253Z
updated_at: 2026-08-25T14:49:51.814Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
Constraint carried from the engine: watch requires full scope, so --watch combined with --scan-depth or --one-filesystem is a usage error (exit 2) with a message naming the conflict, until validate_for_watch_scope learns otherwise. Selection-axis flags (--depth, --include, --min-size, --modified-since) remain fully legal with --watch, since they filter the retained index rather than narrowing what is observed - that distinction is exactly the scope-versus-selection split and the error message should make it legible. tryscript coverage for each rejected combination and at least one accepted selection-plus-watch combination.

## Notes

FDU47-E3 addressed at 353d48f: one_filesystem now agrees with the scan.

The disagreement was mine, and the doc claim that hid it was too. should_descend
gates *descent*, not retention: a directory on another filesystem is listed by
its parent and recorded, and nothing under it is ever read. My within_scope asked
the entry's own device, which rejected the mountpoint row the scan deliberately
keeps -- so a live event on it deleted that row and the next rescan put it back.
InvalidateSubtree was checked with zeroed attrs and passed, after which
resolve_subtree_root could reject reconciliation below the boundary.

The rule is "did the walk descend into this entry's parent", which is the
parent's device. scan::on_root_filesystem asks that, split out from within_scope
because it is not a property of the path: it needs a stat, and of a different
entry than the one being admitted. One stat, memoised per directory, since a
coalesced intent is usually one directory's worth of events. A parent that cannot
be stat'ed admits, because the walk already drew this boundary and refusing on a
transient error would remove rows the tree still holds.

Two design-doc claims from the previous commit are corrected with it: this axis
is *not* decided by "a path and one stat", and the split between the path-pure
axes (hidden, depth) and the one needing I/O is now stated in scan.rs, watch.rs
and the implementation spec.

Two devices cannot be fabricated in a unit test without a mount, so the test pins
the half that is checkable: whose device is consulted. Against a root_dev nothing
matches, the root is still admitted (no parent to disqualify it) and a child is
not. It also asserts the pairing directly -- should_descend stops at a mountpoint
on another device, and on_root_filesystem keeps the row for it. The mutation that
asks the entry's own device fails it.
