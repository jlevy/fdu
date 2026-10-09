# Plan: Several Paths in One Report, and an Age Column in the Tree

**Date:** 2026-10-09 (last updated 2026-10-09, after design review and review A on #191)

**Author:** fdu project, with Claude Opus 5.5

**Status:** Approved for implementation as the 0.5.0 release, in two stacked pull
requests: the age column, then several roots.
The report schema moves to `fdu.report/11`, and the engine’s public report types change,
which the 0.x minor version allows.

## Overview

Two additions, each small on the surface, each starting in the engine:

1. **An age column in the tree.** Every tree row shows how long ago what it counts was
   last modified: `4d`, `2mo`, `3y` in text; the exact nanoseconds, a signed age, and an
   RFC 3339 instant in JSON, JSON Lines, and YAML.
2. **Several paths.** `fdu docs src` reports on both trees together, as if fdu had run
   on each and the results were added: sizes, counts, rows, shares, and bounds are all
   computed over the union, once.

```console
$ fdu docs src
██████████   100%      40 MiB     2m  (total) 1,212 files
██████░░░░    62%      25 MiB     2m    docs/ 496 files
███░░░░░░░    28%      11 MiB     4d      experiments/ 312 files
██░░░░░░░░    22%     8.6 MiB    21m      reports/ 76 files
█░░░░░░░░░    14%     5.4 MiB             … and 108 more files
████░░░░░░    38%      15 MiB     2m    src/ 716 files
███░░░░░░░    31%      12 MiB     2m      core/ 402 files
░░░░░░░░░░     7%     2.9 MiB             … and 314 more files
```

## Goals

- The text tree shows an age on every entry row, and machine formats carry it exactly.
- Where a tree row and a list row count the same entries, they report the same age;
  `--long` and the tree never disagree over one directory in an unfiltered report.
- One request can name one or more disjoint roots, on every surface: the engine, the
  command line (`fdu PATH...`), and Python (`fdu.report([...])`).
- A report over several roots equals the merge of its single-root reports: additive
  values sum, maxima take the maximum, and every display bound is applied once, after
  the merge, against the combined total.
- A report over one root keeps its shape; only the age column and the new age fields
  change it.
- The default one-shot report gets no measurably slower on macOS. That is a goal, not a
  result: the loaded host it was measured on could not resolve the default report’s
  change either way ([below](#computing-it)). On Linux it is expected to give back about
  3% of the default tree on a directory-dense tree, because every route must read
  directory and symlink times ([below](#every-route-reads-directory-and-symlink-times));
  H193 measures that before release (`fdu-088k`).
- A retained report pays the column’s per-row work and no work per entry, so its cost
  grows with the rows it shows, never with the tree beneath them.
  Measured on macOS at the shipped head, that is no resolved change over a retained
  `Index` and 17 µs (+11.5%) of the second report over an opened root, recorded as the
  column’s cost, not as a speed decision
  ([exp-212](../../experiments/exp-212-macos-the-age-column-re-measured-at-the-shipped-head-per-row.md);
  the first build’s figures, 7.5 to 23 µs, are in
  [exp-210](../../experiments/exp-210-macos-h192-maintained-activity-leaves-the-age-column-per-row.md)
  and
  [exp-211](../../experiments/exp-211-macos-h192-replicated-over-an-opened-root-23-microseconds-a-.md)).
  Linux is unmeasured.

## Non-Goals

- **Watching several roots.** `--watch` keeps one root and refuses more, naming the
  limit (`fdu-ijpo` records the design it would need).
- **Cache lifecycle over several roots.** `--cache-status` and `--cache-clear` keep at
  most one PATH and refuse more.
- **Opened or scanned indexes over several roots.** `fdu.open` and `fdu.scan` return one
  `Index` for one root; several roots compose at report time.
- **Overlapping roots.** A root equal to or inside another is refused
  ([below](#overlapping-roots-are-refused)).
- **An age on remainder rows** (`… and 108 more files`). See
  [Remainder rows](#remainder-rows-carry-no-age).
- **A switch to hide the column.** There is no columns axis, and the six-axes rule rules
  out a one-off flag; `--bar-size 0` remains the only column control.

## Background

### Ages

Every walker reads each entry’s modification time on every platform, and the index keeps
two recency values:

- `RollUp.newest_mtime_ns`, maintained incrementally: the newest **regular file** under
  a directory, `0` in the roll-up and `null` in `RollUpSummary` when it has none.
  Summary reports it, tree nodes carry it, and `--sort mtime` orders tree rows by it.
- `SubtreeValues.mtime_ns`, computed per report by `query_subtrees::measure`: the newest
  activity of a directory **and every eligible descendant**, directories and symlinks
  included, with a `complete` flag.
  List rows carry it with a signed `age_ns`; `--long` prints it; `--modified-since` and
  `--modified-before` test directories against it.

[The directory query formats plan](plan-2026-09-20-directory-query-formats.md) kept them
apart: summary keeps its released files-only meaning, a row’s age is subtree activity.
`RollUp` documents why it excludes directories: their times change on every child add or
remove, so a files-only “what changed recently” is not dominated by directories.
The text tree shows neither value today.

### Several roots

fdu takes exactly one PATH. `du`, `dust`, `fd`, `find`, and pdu take several; pdu joins
them under a synthetic `(total)` root and removes overlapping ones
([the pdu research](../../research/research-2026-09-28-pdu-and-the-linux-peer-gap.md)).
Two open beads asked for this: `fdu-onoo` (several roots under a combined total) and
`fdu-khu8`, whose open question 2 noted that one index per root composes easily in the
library but that the command-line ergonomics and the cache story were undesigned.

Everything below one report assumes one root: `Basis.root`, one `Index`, one snapshot
file in `Delivery.cache_path`, one `Report.root`, root-relative row paths, and shares
and bounds against that root’s total.
Logical-word estimates are pooled from statistics that are never serialized, so two
rendered reports cannot simply be added; a merge has to happen inside the engine, before
bounds.

## Design

### Part 1: The Age Column

#### What a row’s age measures

A tree row’s age is the newest modification time among the entries it counts: its own
entry, when the selection admits it, and every entry beneath it that its tallies count,
of any kind.
The report root is a traversal boundary, never an admitted entry, so its own
time never counts, as no list row shows the root either.
A row that counts no entries has no age.

For an admit-all selection this is exactly the list row’s definition, `SubtreeValues`,
so `fdu . --long --kind dir` and `fdu .` show the same age for each directory.
Under a filter, the tree row and the list row already count different populations — a
list row measures the eligible subtree before positive predicates, a tree row only what
the selection admits — and each age follows its row’s population, so
`fdu . --kind file --modified-before 1y` never shows a row aged `3d`.

Counting directory and symlink activity, rather than files alone, is deliberate:

- **It answers the column’s question.** Deleting or renaming a file changes no surviving
  file’s time, but it is a modification of the subtree.
- **Files-only lies about installed trees.** npm writes package files with a fixed 1985
  time and `tar` preserves archive times, so a `node_modules` installed today reads
  about `40y` by files alone and shows its install time by activity.
- **It agrees with `--long`.** A third definition of age would make two views report two
  ages for one directory.

The cost, which is why `RollUp` excludes directories, is that churn (a lock file created
and removed) makes a directory look recent.
For a column labelled age, recent activity is the truthful answer, and the files-only
value stays available as `newest_mtime_ns`.

#### Completeness

A lower-bound maximum is not an age.
When a row’s subtree was not listed in full — a scan-depth boundary, a failed subtree, a
directory an opened root has not listed — its age is unknown, exactly as for list rows.
Over a complete index with no scan depth every subtree is complete, so this costs
nothing there; otherwise the reader already computes `measure` for the partial-tree
proof and takes `complete` from it.

#### Computing it

- **Unfiltered selections over a complete index with no scan depth** read each row’s
  activity from a maximum the index maintains per directory beside `newest_mtime_ns`:
  the newest time among every entry beneath the directory, of any kind, including the
  files a folded index counted without keeping.
  A row adds its own time unless it is the root.
  Every subtree is complete there, so the tree reads only the rows it shows.
  The maximum absorbs an addition or a later time in O(depth), and the stale-max repair
  that `recompute_newest_upward` already ran for `newest_mtime_ns` rebuilds it after a
  removal, an earlier file or symlink time, or an earlier directory time.
  It is kept for the `all` partition alone: an unfiltered selection admits ignored
  entries, and any other selection folds its activity in `walk`.
- **Unfiltered selections over an index that may hold an unlisted subtree** (a partial
  walk, an opened root mid-discovery, a scan depth) take a pass in `query_subtrees`,
  beside `measure`: one post-order traversal by entry id and depth, no paths, which also
  supplies the completeness the partial-tree share proof needs.
  A directory’s activity is the maximum of its own time (unless it is the root), its
  non-directory children’s times, its subdirectories’ activity, and its roll-up’s newest
  file, read only when the roll-up counts files.
  The roll-up includes files the folded one-shot tier counted without keeping as
  entries, which is why this pass, not `measure`, is right there.
- **Filtered selections** fold the same maximum in `walk`, which already folds the
  newest admitted file: it extends to admitted and covered directories and symlinks.
  No extra pass.

Tests pin the definition across routes: the fast pass equals `measure` (with the root
boundary) on every directory of a full index, and the fast pass on a folded index equals
`measure` on the full index of the same tree, through the path-independence harness;
fixtures include empty directories, symlinks, a scan-depth boundary, a failed subtree,
and a directory whose own time is its newest activity.
The maintained maximum equals the pass, and `measure` wherever the index keeps every
file, on every directory: after a detached and a scanner cold walk, on a folded index,
after a snapshot load, on an opened root once discovery settles and after its refreshes,
and after each kind of incremental change (a file or symlink time moved either way, the
newest file removed, a directory time raised and lowered, a kind replaced, a subtree
removed, a rename). The reference model compares every tree row’s activity with its
from-scratch definition after every generated step.

The pass came first and served every unfiltered tree, adding a traversal to reports that
had read only pre-computed roll-ups.
Paired against the pre-age build on `~/.rustup` (77,355 entries; macOS, M1 Pro, an
uncontrolled and heavily loaded host), the one-shot default report’s change was not
resolved (−2.1%, 95% interval −12.2% to +9.6%), but the retained regime did regress
clearly: a second tree report over a retained `Index` went from 0.13 ms to 0.79 ms, and
over a settled opened root from 0.25 ms to 4.9 ms.
An opened root keeps each directory’s children in a name-keyed map whose order does not
follow the arena, so its pass touched scattered entries and cost two to three times the
detached index’s in isolation; the paired figure adds that host’s noise.
That design is recorded as rejected, H191 in
[exp-209](../../experiments/exp-209-macos-h191-a-per-report-activity-pass-makes-a-retained-tree-.md).
So the maintained maximum replaced the pass wherever every subtree is complete (H192).

The bar the maintained maximum is held to is that a retained report’s cost grows with
the rows it shows and not with the entries beneath them; a few microseconds of per-row
age work is the column’s price, not a regression to remove.
Measured the same way afterwards
([exp-210](../../experiments/exp-210-macos-h192-maintained-activity-leaves-the-age-column-per-row.md)),
the second report is 0.134 ms over a retained `Index` (control 0.129 ms), +5.8% (95%
interval +2.7% to +7.5%), and 0.153 ms over an opened root (control 0.135 ms), +13.7%
(+11.7% to +14.7%); a recheck
([exp-211](../../experiments/exp-211-macos-h192-replicated-over-an-opened-root-23-microseconds-a-.md))
put the opened root at +17.8% (+15.1% to +21.4%), 23 µs.
That is 7 to 23 µs a report, against the 0.66 ms and 4.7 ms the pass added.
A cold walk into an index and a snapshot load were non-inferior at +3% on wall; the
default report’s change was not resolved (+3.4%, 95% interval −11.8% to +13.1%). A
snapshot load’s component time moved +2.2% in one run (95% interval +0.5% to +2.7%), and
+1.7% in the recheck, whose interval (−5.3% to +14.0%) is too wide to bound it.
Those two runs measured a binary built before its named commit and before two later
changes on measured paths, so the shipped head was measured again, with each binary tied
to its commit
([exp-212](../../experiments/exp-212-macos-the-age-column-re-measured-at-the-shipped-head-per-row.md)):
the second report over a retained `Index` showed no resolved change (+0.2%, −2.4% to
+1.6%) once each age cell was formatted once, and over an opened root it cost 17 µs
(+11.5%, +8.4% to +14.2%); a cold walk and a snapshot load were non-inferior on wall,
and the snapshot load’s component did not move (−0.4%, −2.1% to +0.1%). The default
report’s wall change was again not resolved (+2.3%, −3.0% to +4.6%), and its peak RSS
moved +4.9% (+0.3% to +10.0%) for a reason the run cannot name.
The one resolved cost is in machine formats: every row gains `modified_at`, and a full
JSON or YAML render of the tree and a file list got 4% to 6% slower, about a quarter of
a microsecond per row.
Writing each instant into a fixed buffer instead of a `String` (`b2968074`,
[exp-213](../../experiments/exp-213-macos-h194-fixed-buffer-instants-halve-the-age-column-machin.md))
removed about half of that: 1% to 2% of the render’s wall remains, about 0.12 µs a row,
the cost of writing the new fields themselves.
These are records of the column’s price, not speed decisions, and every figure is from
one loaded macOS host; Linux is unmeasured (H193).

No snapshot fingerprint changes.
A snapshot stores each entry’s record and no roll-up, and a load rebuilds every roll-up,
this maximum included, by merging each record into its ancestors, so a snapshot written
before the change loads to the value a fresh walk computes.

#### Every route reads directory and symlink times

A row’s age counts each directory’s and symlink’s own time, so every route has to read
it. The folded one-shot index, which answers the default tree, did not: since 0.3.0 it
took H72’s listing policy (H185), taking a directory’s or symlink’s kind from the
listing’s `d_type` with default attributes, because no tree row read their attributes.
That applied on Linux, on any other Unix, and on macOS wherever a directory fell back
from bulk listing to the portable reader.
There a directory would be aged by its files alone on the default route and by its
activity on every full-index route, which makes the route part of the answer.

So the folded index stats every child, as the full index does, on the native readers and
the portable fallback alike.
The transient summary keeps H72, since its `newest_mtime_ns` is files-only and it reads
no directory’s or symlink’s attributes.
The price is expected, not yet measured.
It is one `statx` per directory and per symlink: +7,886 on the directory-dense
`node-modules-dense`, +5,180 on `linux-v6.12`, and about +125k on the million-entry
generated tree.
At the rate H185 saved them that predicts about 3% of the default tree on
`node-modules-dense`, 0 to 2% on `linux-v6.12`, and about 4.5% on the generated tree.
H185 measured −3.55% on `node-modules-dense` (95% interval −7.85% to −2.57%;
[exp-197](../../experiments/exp-197-linux-h185-describes-each-directory-once-on-the-folded-tree-.md)),
and the policy that shipped kept 83% of those `statx` calls saved, so the give-back
should be that size or a little less.
These predictions are pre-registered as H193, and `fdu-088k` measures them on Linux
before 0.5.0 is released.

One metadata read per directory and per symlink is the interface floor route-equal ages
need: [the floor report](../../reports/report-2026-08-23-metadata-walk-floor.md)’s “per
entry, exactly one metadata call”, which H185 had taken the tree tier below.
Taking a directory’s attributes from its own opened descriptor (H179) would not win it
back: since H169’s first phase the native reader makes no call on that descriptor, so
H179 would swap one call for another.
If the give-back has to be won back, the mechanism whose saving scales with the
directory count is H169’s third phase, opening each directory relative to its parent’s
descriptor.

#### Tree nodes and sorting

`TreeNode` gains `mtime_ns` (newest counted activity), `complete` (null for a file), and
`age_ns` (signed, null when incomplete, empty, or unrepresentable).
`newest_mtime_ns` keeps its files-only meaning.
`--sort mtime` on the tree orders by `mtime_ns`, so the order matches the column it
sorts and the order a list sorted by mtime already uses.

#### The age reference

`age_ns` is measured from the request’s `now`, the instant its relative time windows
resolved against. Windows are resolved to absolute bounds when the request is built, so
after that `now` serves only as the age reference.

A watch session builds its request once, so a file modified after the session starts
shows a negative age on every later repaint.
Each repaint, and `Session::report` (which Python’s `Watch.report` calls), now reads
with a copy of the request whose `now` is that repaint’s instant; the windows do not
slide, because they are already absolute.
The skip-identical-repaints check compares the exact `mtime_ns` and `complete` of every
rendered row, plus what it compares today, with the age reference held fixed, so an idle
tree repaints nothing while its ages roll over and any change in a row’s activity
repaints. The `Request.now` doc comment, the machine-output reference (“fixed request
instant”), the Python README, the skill, and the design principles’ watch section (which
calls the default tree size-only) are amended.

A one-shot report samples `now` before the walk, so a write during the walk can show a
small negative age (`-2s`). It is printed as measured, as `--long` already does.

#### Remainder rows carry no age

`… and 108 more files` stands for rows the bounds hid.
The folded one-shot tier counts most files in a tally without keeping their times, so
today only the full index could give the remainder an age; showing one on one route and
not the other would make the route part of the answer.
The cell is blank, which keeps columns aligned, and the parent row’s age covers
everything beneath it.
Machine-format omissions carry no time.
Keeping a maximum in the folded tally would make it possible on both routes and is
recorded as a follow-up.

#### Text rendering

The column sits between size and name, as in `--long` (size, age, path), right-aligned
to the section’s widest age cell, with two-space gutters.
An unknown age is the gray word `unknown`, as `--long` prints it; a row with no age is a
gray `—`, as a missing percentage already is.

`human_age` remains the one formatter for the tree and `--long`, and gains two units so
an old tree does not read as `4,382d`:

| Age | Shown as |
| --- | --- |
| under 60 s | seconds, `42s` |
| under 60 min | minutes, `59m` |
| under 24 h | hours, `23h` |
| under 30.44 d | days, `30d` |
| under 365.25 d | months of 30.44 days, `11mo` |
| otherwise | years of 365.25 days, `3y` |

Units switch at their own length, so no cell reads `0mo` or `0y`. Values are floored
toward zero and signed: `-5m` for a future time, `-0s` under a second ahead, as today.
A cell is at most four characters plus a sign below a century.
`mo` and `y` are display units only.
`parse_age` refuses them because a month or year has no fixed length to resolve a window
against (the reasoning at `query_values.rs` stands); its refusal now names the day
equivalent (`use 60d`). The `--long` column changes with the ladder: the unit tests that
pin `30d` and `1,000d`, and the `cli-axes` golden that expects days for its 2000-01-01
fixture (now `26y`), move with it.

#### Machine formats

- Tree nodes gain `mtime_ns`, `complete`, `age_ns`, and `modified_at`.
- List rows gain `modified_at`.
- The envelope gains `age_reference_at` beside `age_reference_ns`.

`modified_at` is the RFC 3339 UTC rendering of `mtime_ns` from the existing
`format_rfc3339_nanos` (`2026-10-09T07:26:50.123456789Z`), and it is `null` whenever the
age is null for incompleteness: a lower bound rendered as an instant would present it as
a modification time.
The integer fields stay; they are exact and already read.
The schema becomes `fdu.report/11`.

Python’s tree-node and list-row models gain the same fields.
`modified_at` is a timezone-aware UTC `datetime` derived from `mtime_ns` (floored to the
microsecond, correctly before the epoch), not parsed from the string.

### Part 2: Several Roots

#### The request names its roots

A new engine type beside `Basis` models the roots of one report: a non-empty, ordered
list, each with a **label** and its canonical path.
The label is the path as the caller gave it, normalized by its components
(`PathBuf::from_iter(path.components())`), which drops repeated and trailing separators
without turning `/` into an empty string or
`C:\` into a drive-relative `C:`. A label that is not valid UTF-8 serializes with a `label_raw`,
as paths already do.

Constructing it validates every root before anything is scanned: each must exist and be
a directory, with the errors and exit codes one root already has, and no two may
overlap. The rest of the request is shared: the engine derives one per-root `Request`
from one spec, each with today’s single-root identity — basis, snapshot, content
sidecar, cache policy — so a root answered from cache and a root walked cold can sit in
one report.

#### Overlapping roots are refused

`fdu a a/b` cannot be the sum of `fdu a` and `fdu a/b`: every path under `a/b` would
count twice, and fdu counts each path once ([usage](../../../usage.md)). Collapsing the
inner root would be a guess about intent and would change `.gitignore` results, since
rules come only from inside the scanned root.
So a root equal to or inside another is refused, naming both labels
(`fdu: src/core is inside src; name one or the other`). Refusing is the reversible
choice: accepting overlap later is additive, while changing what a total means after
release is not.

Canonical paths alone miss aliases: on macOS, `/Users` and `/System/Volumes/Data/Users`
are the same directory through a firmlink, and Linux bind mounts behave the same way.
On Unix the check compares each root’s device and inode against every other root’s and
against the chain of every other root’s ancestors; elsewhere, and in addition, canonical
paths are compared component by component.
The check is conservative: `fdu / /mnt/usb --one-filesystem` is refused even though the
scope would not have descended into the second root.

What no path or ancestor shows is an inner root that is itself an alias into the outer
root’s tree: a bind mount, or `/usr/local` beside `/System/Volumes/Data` on macOS, whose
own ancestors never pass through the outer root (review A3 on #192). On Unix each root’s
walk therefore compares the directories it enters with the other roots’ identities, on
every tier: the full and folded indexes keep every directory they entered, a cache-only
load reads the snapshot’s, and the summary fold, which keeps none, states every
directory over several roots and checks each as it arrives.
A walk that enters another root refuses the report as overlapping, naming both labels
and where it met it (`RequestError::RootReachedInside`); a walk that only lists it, at
the scan depth or across a filesystem boundary, counted nothing twice and is not
refused.

#### Caches

`Delivery.cache_path` names one snapshot file, derived today from the one root.
A report over several roots takes a cache directory instead, as a delivery field,
`Delivery.cache_dir`, and the engine derives each root’s snapshot path with
`default_cache_path_in`, the function the command line uses for one root.
`None` in either field means no cache there, never a default: a surface that caches by
default passes its resolved default directory (review A4 on #192), and every route
resolves a directory to its root’s file before it reads or writes
(`Delivery::for_root`). A delivery that names a single explicit snapshot file is refused
with several roots, and one that names both a file and a directory is refused
everywhere. No root’s snapshot is written until every root has been walked, since the
cache directory can lie inside a later root, whose walk would then count a write in
progress (review B3); the writes then run together on at most as many threads as the
machine runs at once, and the caller joins them in one handle.

#### Execution

With one root, every surface takes today’s path unchanged.
With several, each root runs its own plan in argument order and keeps what the reader
needs instead of reading immediately: a summary row on the summary tier, a folded index
on the tree tier, or a full index.
The folded tier’s retention stays sound: a file that reaches the share threshold of the
combined total reaches it of its own root, so each root keeps a superset of what the
merged tree can show.

One progress handle spans the run, so counters are cumulative; the progress frame names
the root being walked and its position (`src (2/3)`). Any root that fails fails the run;
under `--allow-partial` a partial root makes the report partial.
The `perf:` line sums walked files and bytes and names each cache tier when the roots
differ.

#### Reading several indexes into one report

Each section builder splits into an accumulation over one index and a finalization that
applies bounds. One root finalizes its own accumulation; several finalize their merge.
The single-root path is the multi-root path with one input, so each section keeps one
definition.

| Section | Merge |
| --- | --- |
| Summary | Counts and sizes sum; newest file time is the maximum; the ignored share is unknown if any root’s is |
| Extensions, types, families, languages, documents | Per-bucket accumulators merge before shares and limits; pooled word statistics merge as sufficient statistics; `observed` holds only if it holds for every root |
| Code | Per-language tallies and the total add |
| List, files, largest, recent | Each root’s rows are classified and bounded by the row limit on their own (exact for a sorted top-k), then the existing `sort_rows` and `truncate` run once over the concatenation; `total` sums |
| Tree | Per-root trees under a section-level total (below) |

**The tree.** With several roots, the tree section has no single root node.
It carries a `total` row (the merged tallies, the newest activity across roots, complete
when every root is) and one ordinary tree per root, built as follows:

1. Each root’s tree is built by the existing builder with the combined total passed in
   as the share denominator (`expand` takes it as a parameter instead of deriving it
   from the node), uncapped.
2. Every root is a row whatever its share, a zero-byte root included, because the caller
   named it; `--breadth` does not apply to the roots, and `--depth` counts below each
   root, as for that root alone.
3. Roots are ordered by the existing tree sorter, with their labels as names, so ties
   and `--reverse` behave as for any rows; under a metric sort a root has no value and
   sorts last.
4. The row cap (`--limit`) and `--limit 0` are applied once, over the assembled
   pre-order with the total row first, using each root’s own completeness.
   Rows past the cap, root rows included, are omitted with the usual records.
5. Each root keeps its own remainder: in text, `… and N more files` appears under that
   root; in machine output, each root tree carries its omissions as one root does today.

**Paths.** Machine formats keep every path relative to its root and add a `root` index
into the envelope’s `roots`, on list rows, status errors, and `.gitignore` refusals, so
`roots[i].path` joined with `path` resolves exactly and an absolute label never makes a
“relative” path absolute.
Text, `paths`, and `long` print `label/path`, as `find docs src` does.
Everything that reads a root-relative path keeps reading it: `.gitignore` matching,
globs containing `/`, and documentation and vendored classification.
So `fdu docs src --include 'project/*'` selects exactly what the two single-root runs
select, and, by the same sum rule, a file at the top of `docs` is not classified as
documentation, as it is not when `docs` is scanned alone.

**Status, provenance, notes.**

- `complete` holds when every root is complete; the coverage reason is the first
  incomplete root’s.
- Errors and refusals are ordered by root, then by path, and bounded by the existing
  retention limits.
- Provenance takes the weakest source and least-fresh tier, as it already does across
  tiers, and the earliest scan start.
- `.gitignore` coverage sums its counts.
- Analysis metadata merges per analyzer.
- Notes and tips that depend only on the request are derived once; the four that read an
  index or its control coverage (the unverified-ignored note, the incomplete-subtree
  note, the text-only Markdown count, and the retained-refusals tip) are computed from
  the merged values.
- A display-limit note says `of total` where one root says `of root`.

#### Output shape

| Where | One root | Several roots |
| --- | --- | --- |
| Envelope | `root` as today; no `roots` | `root: null`; `roots: [{label, label_raw?, path, path_raw?}]` in argument order |
| Tree section | `tree` as today | `tree: null`; `total`; `trees`, one root tree each, root node named by its label |
| List rows, errors, refusals | unchanged | gain `root` |
| Text | unchanged apart from the age column | a `(total)` row, root rows named by label, labelled flat paths |

Exactly one of `root` and `roots` is non-null, so a one-root report needs no label and
looks the same on every route: `fdu.report`, `fdu.open(...).report()`, and watch.
The total is not a tree node and has no entry kind: `EntryKind` is the snapshot format’s
and `--kind`’s vocabulary and gains nothing.

#### Surfaces

- **Command line.** `PATH...` takes one or more (usage `fdu [OPTIONS] <PATH>...`).
  `--watch`, `--cache-status`, and `--cache-clear` refuse a second PATH with a usage
  error naming the limit.
  Help, `--docs`, and the skill describe several roots.
- **Python.** `fdu.report` takes one path or a sequence of paths in its first argument,
  testing for `str` and `os.PathLike` before treating it as a sequence; the `root=`
  keyword keeps working.
  `Report.roots` is a tuple of labelled roots or `None`; `Report.root` is `None` for
  several; rows gain `root`. `fdu.open` and `fdu.scan` stay single-root.
  The parity shim accepts several positionals, so the golden corpus replays on both
  surfaces.

### Components

- `query/query_subtrees.rs`: the unfiltered pass beside `measure`, the root boundary.
- `query/query_report.rs`: tree-node fields and sort, the filtered fold in `walk`, the
  per-section accumulate/finalize split, the multi-index reader, the tree assembly,
  status, provenance, and note merges.
- `query/query_request.rs`: the named-roots type, validation, per-root requests, the
  `now` doc comment.
- `execution.rs`, `lib.rs`: several plans retaining state for one read; per-root cache
  paths; pending saves.
- `watch_session.rs`: the per-repaint request instant and the repaint check.
- `report_format.rs`: the age column, the total and root rows, the ladder, machine
  fields, `fdu.report/11`.
- `crates/fdu/src/cli.rs`, `progress_line.rs`: `PATH...`, refusals, progress, `perf:`.
- `crates/fdu-py`: `report` over several roots, models, stubs, the parity shim.
- Tests: `tests/path_independence` gains several-root requests and folded-versus-full
  age cases; its known-violations registry stays empty.
- Docs: usage, machine output, output design, the design principles (scope axis, watch,
  views are readers), the engine architecture’s request model, the surface
  architecture’s schema table, the correctness runbook’s age check, README, CHANGELOG,
  `--docs`, the skill, the Python README.

### API Changes

For the changelog and `make semver-check`:

- `TreeNode` gains `mtime_ns`, `complete`, and `age_ns` (breaks struct literals).
- `Report.root` becomes optional and `Report` gains `roots`; the tree section gains a
  total and per-root trees; rows and issues gain a root index.
- A named-roots type and a multi-root report entry point; a cache-directory delivery.
- Python: `fdu.report(paths)`, `Report.roots`, `Report.root: Path | None`, the new node
  and row fields.
- Machine output: `fdu.report/11`.

## Implementation Plan

Two stacked pull requests on gh-stack, the age column first.

### Phase 1: The Age Column (first pull request)

- [ ] Root boundary in `measure`; the unfiltered pass; tests pinning it to `measure` on
  full and folded indexes
- [ ] The filtered fold in `walk`; tree nodes carry `mtime_ns`, `complete`, `age_ns`;
  `--sort mtime` uses them
- [ ] `human_age` ladder with `mo` and `y`; `parse_age` refusal names the day equivalent
- [ ] Text column, blank remainder cell, `unknown` and `—` in gray
- [ ] `modified_at`, `age_reference_at`, `fdu.report/11`, Python models
- [ ] Per-repaint request instant in watch and `Session::report`; repaint check over
  exact activity
- [ ] Goldens, docs, and the two performance measurements

### Phase 2: Several Roots (second pull request)

- [x] Named-roots type, labels, validation, overlap refusal by identity
- [x] Per-root requests and cache paths; execution retains per-root state; pending
  saves; one progress handle; combined outcome
- [x] Accumulate/finalize split for each section and the multi-index reader
- [x] Tree assembly with a section total, per-root remainders, and bounds applied once
- [x] Root indexes on rows and issues; status, provenance, and note merges
- [x] Command line `PATH...` and refusals; Python `report(paths)`; parity shim
- [x] Goldens over two fixtures, the path-independence cases, docs
- [x] The performance guard for one root and the cost of many roots (below)

**Performance.** Two measurements on macOS, both on an uncontrolled host at a load
average near three times its ten cores, so both are exploratory.
One root first, since every surface now reads it through the several-roots reader and
door (review C1 on #192): H195, registered before the run, predicted wall non-inferior
at +3% on every job the refactor reaches.
[exp-214](../../experiments/exp-214-macos-h195-one-root-through-the-several-roots-reader-non-inf.md)
holds it on five jobs, `cold-scan-index` (the placebo), `default-tree`,
`index-second-report`, `content-query`, and `render-json`, with both retained reads flat
in component, and cannot resolve `aggregate-summary`, `opened-second-report`, or
`render-yaml` either way; those three rerun on a quiet host with H193 (`fdu-088k`)
before release. An earlier run at the pre-review head, before the registration, resolved
nothing
([exp-215](../../experiments/exp-215-macos-an-early-look-at-one-root-through-the-several-roots-re.md)).
Then many roots (review C2): H196 put 625 crate directories against their parent, both
through the several-roots door (`roots-default-tree --child-roots`).
[exp-216](../../experiments/exp-216-macos-h196-several-roots-pay-about-2-4-ms-a-root-625-small-r.md)
measured about 2.4 ms a root on the default tree, 1.72 s against 0.23 s for the parent,
and about 0.9 ms a root on the summary, about eight and three times the estimate.
So `fdu */` over many small directories is several times slower than `fdu .` over their
parent; usage says roots are walked one after another, and one walker pool across roots
is `fdu-ich9`.

Decided during implementation:

- Where the name decides an order (`--sort name`, and every tiebreak), flat rows of
  different roots rank by label first, then by path, so text that prints `label/path`
  reads as sorted; ordering by the relative path alone would interleave the roots.
  Labels that compare equal keep the caller’s order.
  Each root’s rows are ordered among themselves exactly as alone, which keeps the
  per-root bound exact for a sorted top-k.
- A root row keeps the single-root rule that the root’s own time never counts, so
  `ages/installed` in `fdu ages/installed ages/docs` shows what `fdu ages/installed`
  shows at its root, not what `fdu ages` shows for `installed/`; a golden pins it.
- Combined totals that no `u64` holds are refused (`UnrepresentableTotal`), as one
  root’s already are, so every sum across roots is exact.
- `TreeStatus::errors` holds `StatusIssue { root, issue }`, an explicit pairing, rather
  than a parallel list of root positions.
  A `.gitignore` refusal carries its root as a field instead (`RefusedControl::root`):
  `Issue` is the engine contract’s, shared with commits and opened roots, so its root
  goes on a report-level wrapper, while a refusal is the control table’s record inside
  `ControlCoverage`, which an index and a report share whole, and wrapping it would need
  a report-level copy of `ControlObservation` (review A9 on #192). Modeling “exactly one
  of `root` and `roots`” and the tree’s rows as enums is `fdu-couf`, with the one-shot
  API consolidation (review A10).
- One root through `prepare_roots_report` is `prepare_report` over the validated
  canonical path, with the snapshot path derived from the delivery’s cache directory.
  Validating the roots before the walk means `--stale-ok` over a file now fails as not a
  directory rather than as a missing snapshot.
  The refusals keep one root’s old order (review B1 on #192): a root that cannot be
  resolved, then a delivery no route can carry, then a root that is not a directory,
  then overlap (`RootsRequest::resolve`, used by the command line and Python).
  A malformed watch option, which the command line parses, now comes before a missing
  root, with the rest of the request’s parsing.
- Every root’s walk reads the canonical path validation resolved, and the read refuses a
  state built from any other directory (`RootMismatch`, review B2), so a symlink
  retargeted mid-run cannot put another directory into the report.
  `report_roots` refuses indexes of different scopes (`RootScopesDiffer`, review B4).
- A PATH that names a file is still refused, now in the command line’s words with what
  to type instead:
  `fdu: notes.txt is a file; fdu reports on directories (to name only the directories here: fdu */)`
  (review A5 on #192). `fdu *` stops at the first file; accepting a file as a root is
  `fdu-ejw5`.
- Roots run one after another, each with its own walker pool; the cost of many small
  roots against one walk of their parent, and one pool across roots, are `fdu-ich9`.

## Testing Strategy

- **Engine unit tests.** Ages are unknown for incomplete subtrees, absent for empty
  rows, and negative for future times.
  The ladder is pinned at every unit boundary, including `-0s`. Roots validation covers
  duplicates, nesting, a symlink alias, a firmlink alias where the host has one,
  trailing separators, `/`, and a non-UTF-8 label.
  Multi-root sections equal the merge of single-root ones for disjoint fixtures,
  including pooled word estimates and unknown ignored shares.
  In the tree, every root is a row and `--depth` counts per root; `--limit` applies
  once; per-root remainders appear; shares are of the total.
  Flat top-k is exact across roots.
- **Path independence.** Folded-versus-full ages; several-root requests against the
  merge of single-root cold answers.
- **Watch.** A file modified after the session starts shows a non-negative age on the
  next repaint; an idle tree still repaints nothing; a touch inside one age bucket still
  repaints. Removing a directory’s newest file, renaming across directories, and moving
  an old file in leave each directory aged as a cold walk ages it, on a watch and
  through `OpenedIndex::refresh`; a seeded property test holds a watched tree to a cold
  walk of the same tree after every generated step, because a test that compares the
  index only with itself cannot see a time no event carried (review B on #191).
- **Goldens.** Default tree sessions use the existing `AGE` pattern for the column,
  since their fixture times are fresh.
  One session stamps a fixture at fixed distances before the run, mid-unit, and pins the
  ages it reads (`3d`, `2mo`, `1y`) on the folded default tree, the full index
  `--cache on` builds, and `--long`, including a directory whose own time is its newest
  activity. A new session over two fixtures covers the text tree, a flat listing, JSON
  shape, and the overlap refusal.
- **Parity.** The Python shim replays every new session; no unclassified deviation.
- **Performance.** Paired one-shot comparison and a retained-index report timing, each
  recorded with its regime and each binary tied to the commit it was built from; the
  machine-format render jobs, since every machine row gains `modified_at`; and on Linux,
  H193 before release.

## Rollout Plan

Release 0.5.0 by the release checklist, once both pull requests have landed.
The release notes name the schema bump, the new column and its definition, the `--long`
ladder change, the `--sort mtime` change on trees, the Linux default tree’s return to
reading directory and symlink times, and the multi-root shape.

## Open Questions

- Should a remainder row carry the newest time among the rows it hides?
  It needs a maximum in the folded tally, one comparison per folded file.
- Should watch accept several roots (`fdu-ijpo`)?
- Should there be a way to hide the age column, as `--bar-size 0` hides the bar?
  Not without a columns axis.

## References

- [Design principles](../../architecture/fdu-design-principles.md)
- [Engine architecture](../../architecture/fdu-engine-architecture.md)
- [Output design](../../architecture/fdu-output-design.md)
- [Machine output](../../../machine-output.md)
- [Directory query formats](plan-2026-09-20-directory-query-formats.md)
- [CLI and skill follow-ups](plan-2026-09-27-cli-and-skill-followups.md) (`fdu-khu8`)
- [pdu and the Linux peer gap](../../research/research-2026-09-28-pdu-and-the-linux-peer-gap.md)
  (`fdu-onoo`)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
