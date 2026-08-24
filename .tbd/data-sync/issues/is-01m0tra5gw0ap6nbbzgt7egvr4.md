---
type: is
id: is-01m0tra5gw0ap6nbbzgt7egvr4
title: "[bug] Gitignore bind walks the whole tree at open, even cache-only"
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T20:45:09.518Z
updated_at: 2026-08-24T21:28:38.314Z
closed_at: 2026-08-24T21:28:38.313Z
close_reason: |
  Shipped. `make check` green.

  THE DEFECT. `GitignoreSet::build(root)` discovered its own control files by walking the
  tree with `ignore`'s walker, and `TagRules::from_names(names, root)` called it at
  argument-parse time. So a Path-tier rule cost a full filesystem traversal before the
  engine had done anything: `--cache only` opened by walking the tree its entire contract
  says it does not touch, a cold scan visited every directory twice, and every `.gitignore`
  save during watch re-walked the root.

  THE FIX. Naming and binding are separate steps, because they answer to different things.
  `from_names(names)` validates what a user typed and takes no root at all. Binding needs
  to know where the control files are, and that is a question the *index* answers -- a
  `.gitignore` is an ordinary entry, so the set of them is already in hand.
  `Index::control_file_directories()` collects them by testing basenames during one
  in-memory traversal, paying for a path only at the tens of hits.
  `GitignoreSet::from_directories(root, dirs)` then reads exactly those files. The walker is
  gone from the crate: `ignore::WalkBuilder` no longer appears anywhere in the source.

  THE WINDOW, and why it is safe. A scan tags each entry as it lands, which is necessarily
  before a Path-tier rule can bind -- the control files are not all known until the walk
  ends. `open_for_report` closes the window at one point on each of its three paths, and the
  `adopt_tag_rules` re-tag makes every earlier bit right. So writing under unbound rules is
  a step in a correct sequence; reading under them is the bug, and that is where the
  `debug_assert` lives -- `Index::tags_of` and `Index::tag_bits_of`, not `evaluate`. Putting
  it on `evaluate` first was wrong and the scan path said so immediately.
  `GitignoreSet::unbound()` is distinct from a bound set that found nothing: both ignore
  nothing, but one is an answer and the other is the absence of one.

  TESTS.
  - `a_cache_only_open_with_gitignore_rules_does_not_walk_the_tree`: zero `dir_opens` across
    a cache-only open with two `.gitignore` files in play, and the same tags as the cold
    scan -- so the zero is a saving rather than a regression. Mutation-checked: making
    `bind_path_tags` a no-op fails it.
  - `a_cold_scan_with_gitignore_rules_opens_each_directory_once`: opens bounded below by the
    directory count and strictly below twice it.

  A MEASUREMENT LESSON worth keeping. The first version of the cold-scan test compared two
  scans inside one counter window and read 75 opens against 4. The counters are
  process-global, `test_serial` only serializes the tests that take it, and `enable(true)`
  is process-wide -- so a wide window collects every concurrent walk. The repo's own scan
  counter test documents this and uses deltas with `>=`. Rewritten to one scan, a narrow
  window, single-threaded so the walk's counts land on the measuring thread, and a bound
  rather than an equality: concurrency inflates and never deflates, so "fewer than two
  traversals" is a claim it cannot break.
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
