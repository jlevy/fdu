---
type: is
id: is-01kzqn502680awzhvddzntq32d
title: "P3: watch scope validation errors"
kind: task
status: closed
priority: 1
version: 8
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
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-11T05:36:29.253Z
updated_at: 2026-08-25T13:03:42.925Z
closed_at: 2026-08-25T13:03:42.925Z
close_reason: null
resolution: null
duplicate_of: null
---
Constraint carried from the engine: watch requires full scope, so --watch combined with --scan-depth or --one-filesystem is a usage error (exit 2) with a message naming the conflict, until validate_for_watch_scope learns otherwise. Selection-axis flags (--depth, --include, --min-size, --modified-since) remain fully legal with --watch, since they filter the retained index rather than narrowing what is observed - that distinction is exactly the scope-versus-selection split and the error message should make it legible. tryscript coverage for each rejected combination and at least one accepted selection-plus-watch combination.

## Notes

Closed at ce8d78b. The bounded scope a consumer opens -- a positive max_depth and
a positive max_files -- is now watchable, without a second watcher, an uncapped
index, or adapter-side mirror state.

The refusal was one rule over three axes, and the axes turn out to differ:

- max_depth and one_filesystem are properties of the entry an event names. A
  path and one stat decide both, so the boundary the walk drew is redrawn around
  each event. scan::within_scope is that predicate, beside admits (by name,
  mid-listing) and retains (by kind, after the stat). Depth counts components,
  which agrees with should_descend by construction: it admits a child at
  parent_depth + 1 < max, so the deepest entry a walk records has exactly max
  components.
- max_files is a property of the whole inventory, so no per-event predicate can
  decide it. The index keeps it instead: upsert_beneath refuses a new file row
  once the root roll-up reaches the cap, where the previous state of the path is
  already in hand. That also closed the fdu-97dd remainder -- reconciliation
  walks from the index and never consulted the walk's budget, so one refresh
  turned a bounded inventory into an unbounded one.

An out-of-scope upsert becomes a removal rather than a dropped op: a directory
moved deeper, or a filesystem mounted over one, is one event on a path that
never goes absent, so anything less leaves the old row standing. An out-of-scope
invalidation is dropped -- it asks for a subtree that is not in scope.

Three decisions to know before touching it:

- Directories are not counted against the cap. A directory carries no bytes of
  its own and admitting one keeps the tree navigable to what is there; counting
  them would lose the shape as well as the contents.
- The refusal and the coverage loss are one commit (AppliedDelta::of_both). At a
  cursor between them the index would have dropped an entry and still claimed to
  cover everything.
- Which files a long-lived capped index holds depends on the order events
  arrived, as which files a capped walk holds depends on the order it reached
  them. No rule bounds the retained set and is history-independent at once; this
  one at least keeps the bound.

With nothing left to refuse, validate_for_watch_scope reduces to
validate_for_scope, and WATCH_SCOPE_GUIDANCE plus the CLI flag-name translation
built to render it are gone. Reconciliation needed no change: should_descend
returning false already reads as "out of scope, remove what is below".

Divergence to carry into fdu-kl7r's agreement fixture: the consumer's own
reference walker gives each subtree rewalk a fresh max_files budget, so its
retained set is bounded per walk rather than in total. One side has to move.

Tests: three live bounded-watch cases plus three unit cases on the admission
funnel; a capped-refresh pair in walk_budget.rs; the golden now pins what the
cap does rather than the refusal. Every predicate mutation-checked -- depth
always-true and off-by-one, the funnel skipping the boundary, the fast path
forgetting an axis, dropping instead of removing, keeping an out-of-scope
invalidation, the cap removed, the cap off by one, directories counted, the
refusal not marking coverage. Each fails a named test.

Also extends check-admission-sites.mjs to the kind rule by pinning the set of
functions that ask it, and fixes two things the corpus found: the parity shim
knew neither --max-files nor --special, and returned 1 for partial coverage
where the command line returns 2.

make check and make cross-lint pass; parity holds at 21 recorded deviations,
one fewer than before because the watch-scope message no longer exists.
