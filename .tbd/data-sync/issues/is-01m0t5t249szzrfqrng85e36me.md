---
type: is
id: is-01m0t5t249szzrfqrng85e36me
title: Hidden-path admission as scope, with an exact-name allowlist
kind: task
status: open
priority: 2
version: 10
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019372007
    at: 2026-08-25T13:21:14.537Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:56.893Z
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:47.400Z
updated_at: 2026-08-25T15:10:56.894Z
closed_at: 2026-08-25T06:35:17.988Z
close_reason: |
  Shipped as `crates/fdu-core/src/admission.rs` plus wiring across all three surfaces.

  Engine: `HiddenPolicy` with `keep_all`/`prune_hidden`/`admits`/`fingerprint`,
  `admission::parse_policy` and `AdmissionError`, `ScanConfig.hidden`,
  `ScanScope.hidden_fingerprint`, and `ScanReport.control_dirs` ->
  `Index::adopt_pruned_control_dirs`. Snapshot format 2 -> 3: the scope record is
  positional, so a field added to it moves every byte after.

  Command line: `--hidden keep|prune` and `--hidden-allow LIST` on the Scope axis, with an
  axis-table row. Python: `ScanOptions(hidden=, hidden_allow=)`. fdu's own default is
  untouched -- a du replacement counts what is there, so pruning is opt-in and fingerprints
  to zero.

  Four things worth recording.

  1. The admission rule and the `dotfile` tag share one predicate on purpose. They are
     distinguished by what they do with an entry, never by which entries they mean: a
     second definition of hidden would make `--hidden prune` and `--not-tag dotfile`
     disagree about one file, and the disagreement would read as a bug in whichever
     surface was consulted second. Windows' FILE_ATTRIBUTE_HIDDEN is deliberately not
     read, for the same reason.

  2. Admission is asked once, by name, before the stat, from all four listing loops the
     engine has. One predicate rather than four copies, because the copies are how a scan
     and a refresh come to disagree about which entries exist.

  3. The snapshot has to record where the pruned control files were. Binding a gitignore
     rule asks the index where the `.gitignore` files are, and pruning is exactly what
     removes them from it. The trap: under `CachePolicy::Auto` a revalidation re-walks and
     re-records them, so the section could be deleted and every assertion still passed. The
     warm-start test only says what it means under `Only`, which cannot touch the tree.

  4. Two copies of one rule is two messages. The CLI validated `--hidden` itself and the
     Python dataclass validated `hidden` again, giving `--hidden` against `hidden` and
     double quotes against single for one mistake; the parity harness recorded the pair as
     a difference between the surfaces. `admission::parse_policy` is the only judge now.

  Six new golden sessions, replayed by the parity harness as six exact matches. No new
  declared deviation.

  Not implemented, and no consumer in the engine asks for it: repository-root detection
  inside a pruned subtree. `.gitignore` is the only control file any enabled rule reads.

  Landed in 6a8ac6f on claude/fdu-interactive-client-implementation-map (PR #47).
resolution: null
duplicate_of: null
---
Hidden-path admission as scope, split out of the old fdu-mvt3 during the 2026-08-24
restructure so the tag model does not carry it. This is the reconciliation's settled
answer: hidden files PRUNE at scope with an exact-name allowlist. They are not a tag
and not a plane, because the only named consumer wants them absent, and a tag would
still walk the .git, cache, and virtualenv trees it exists to exclude.

Distinct from the dotfile TAG the model ships, and the distinction is the axis test:
the tag filters with both numbers visible; this prunes, and a pruned entry does not
exist in the index at all.

WHAT LANDS: prune hidden components during the walk except a configured exact-name
allowlist; keep governing control files readable without retaining them; fingerprint
the rule and its allowlist into snapshot identity. ScanScope is positional in the
snapshot header, so the new field means a FORMAT_VERSION bump -- deliberate this time,
unlike the leaf-count case where recompute-at-load avoided one.

No dependency on the tag model. P2: metabrowser Phase 2 wants it, nothing in the
critical plane path does.

## Notes

FDU47-E1 addressed at 353d48f: the live boundary now holds the hidden-path rule.

watch::admitted checked kind, depth and device and never HiddenPolicy, and its
fast path returned early when those three were default. So under hidden="prune"
a hidden file or directory created after boot entered the index and stayed there
until whatever reconciliation happened next -- the same "an index whose contents
depend on whether anyone was watching" failure as the special-object case, one
axis over.

The rule asks every path *component*, not the leaf. The walk never descends into
a pruned directory, so nothing beneath one is in the index either, and a backend
reports `.git/HEAD` as its own path with nothing in that name saying its parent
was pruned. Checking only the leaf would admit an entire pruned subtree one event
at a time.

Out-of-scope upserts become removals (a path that crosses the boundary without
going absent would otherwise keep its old row); out-of-scope invalidations are
dropped, since there is no subtree in scope to reconcile.

The fast path now asks ScanConfig::narrows_entries rather than naming the axes,
because a fast path that names them is one that forgets the next one added --
which is exactly how this happened, one axis after a mutation check that was
supposed to prevent it. every_narrowing_axis_is_visible_to_the_admission_fast_path
turns each axis on alone and requires it to be visible there; writing it found
the one_filesystem disagreement (FDU47-E3) independently.

Tests, all live: a hidden file, a hidden subtree (with the directory created
before the session so the child's own event actually arrives -- created live, the
child event vanishes and the test would pass for a leaf-only rule), an
allowlisted hidden path admitted, a visible-to-hidden rename removed, and the
default scope still recording a hidden path so the four above cannot pass for a
watcher that prunes always.

Mutations checked: hidden admission removed, leaf-only instead of every
component, the fast path forgetting the axis, and pruning regardless of the
policy. Each fails a named test.
