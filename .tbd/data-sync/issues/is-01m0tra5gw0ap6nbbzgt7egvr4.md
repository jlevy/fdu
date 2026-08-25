---
type: is
id: is-01m0tra5gw0ap6nbbzgt7egvr4
title: "[bug] Gitignore bind walks the whole tree at open, even cache-only"
kind: bug
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:50.764Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:12.297Z
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T20:45:09.518Z
updated_at: 2026-08-25T17:06:51.580Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
Found by the 2026-08-24 design review, reading `GitignoreSet::build` against the cache
policy contract. `TagRules::from_names(names, root)` binds the gitignore matcher at
argument-parse time (cli.rs and the Python config build), and `bind` runs
`ignore::WalkBuilder` over the whole root to discover control files. Three consequences:

1. **`--cache only` walks the tree.** The tier whose contract is "never touches the
   tree" performs a full directory traversal at open whenever gitignore rules are
   enabled, before cache policy is even consulted. The answer is still correct; the
   promise is broken silently, and the cost is the whole point of cache-only.
2. **A cold scan traverses twice.** The bind walk visits every directory, then the real
   metadata walk visits them again. With gitignore default-on this doubles directory
   I/O on the scan path for every fdu run on a git tree.
3. **Every `.gitignore` save re-walks.** `TagRules::rebound(&root)` rebuilds the set, so
   a control-file edit during watch costs a full-tree walk plus the full retag, on every
   save.

DESIGN. The index itself already knows where every `.gitignore` is -- control files are
entries. Replace discovery-by-walk with discovery-from-index:

- Scan path: gather control-file paths during the metadata walk itself (the walker
  passes every directory; note `.gitignore` basenames as they stream by), bind matchers
  after the walk delivers them, tag as a deferred pass or tag-on-insert once bound.
  Alternative: keep bind-first for the scan path only if measurement shows the deferred
  pass costs more than the double walk -- decide with `make perf-compare`, not by
  argument.
- Load path (fdu-ycyy's ordering): after entries materialize, `lookup` the `.gitignore`
  entries the snapshot already lists, read just those files (bounded reads, not a walk),
  bind, then evaluate tags. Cache-only then reads N control files and stats nothing
  else, which is an honest reading of "does not walk".
- Rebind path: the watch batch names the control file that changed; rebind by re-reading
  the governed files recorded in the set, not by re-walking the root.

TESTS. A structural counter (or a probe FS) proving `--cache only` with gitignore rules
opens without a directory traversal; a counter proving one scan visits each directory
once; a rebind test proving a `.gitignore` save reads control files only.

INTERACTION. fdu-ycyy's install-before-materialize ordering must land first or together:
bind-from-index requires the loaded structure to exist before matchers bind, which is
the reverse of today's bind-at-parse. `ignore` 0.4.30+'s `IncrementalIgnore`
(build_matchers) is the lazy per-directory alternative recorded on fdu-brt0 -- if the
deferred-bind design gets complicated, measure that instead.

## Notes

Answered at the branch head after the fourth review round.

Hidden pruning gives .gitignore no row, and the live admission rule rewrites its
upsert to a removal of a path that was never there, which commits nothing. A
session watching the delta alone therefore saw an idle tree while every gitignore
tag under it went on describing a file that had changed -- the one case where an
answer-affecting change leaves nothing in the delta to notice it by.

The row was never the obstacle: a tag rule reads its control file from disk by
path. What was missing was being told, in two places.

1. watch::admitted records every op naming a control file the scope excludes,
   before anything rewrites it, and WatchApplyReport::pruned_control_files carries
   it. Deliberately not folded into the `outside` decision that drives the rewrite:
   a *removal* is never outside -- removing what is not there is harmless and
   always allowed -- yet deleting a pruned control file is exactly as
   answer-affecting as creating one. Keying on the rewritten upsert covers create
   and edit and misses delete, which is what the first implementation did and what
   the test caught.
2. Session::rebind_tags_for deposits the *directory* through
   IndexHandle::adopt_pruned_control_dirs before rebinding. Binding looks in the
   index and in the directories a *walk* pruned a control file from, and a control
   file created after that walk is in neither -- so the live rule deposits the same
   fact the walk would have. A directory stays on the list once it is on it,
   including after the file is deleted: binding reads from disk by path, so a stale
   entry finds nothing, and the list is bounded by the directories that ever held
   one, which is the bound the walk's own list has.

Test: a_pruned_control_file_still_rebinds_the_tags_it_governs, over a pruning
scope with a promoted gitignore rule. All three lifecycle events, because they
fail differently -- create has no prior row, edit has no row but a real prior
effect, and delete is the one a rewritten-upsert rule misses. It asserts the file
is still outside the index throughout, so the fix is not accidentally admitting it.

Five mutations, all caught: never recording, recording only the rewritten upsert
(delete), the session ignoring the signal, never depositing the directory, and
depositing the path instead of its directory.
