# Feature: Disk-Usage Checkpoints and Daily Deltas

**Date:** 2026-09-13

**Status:** Proposed.
Research and implementation plan; no new commands or persistence format ship with this
document.

## Objective

Save an inventory of a home folder or selected roots, return a day later, and identify
which directories gained or lost bytes without enumerating millions of unchanged files.
Comparing the same two checkpoints, identified by their immutable ids, must return the
same net changes or refuse explicitly; it must never return a different answer.

The first inventory requires a scan.
Subsequent refreshes should verify changed scopes, update their ancestors, and read or
write only the persisted state they need.
The performance target covers the whole command, including loading and saving state.

## Terminology

Three different change records appear in this plan:

- **Index journal:** the bounded, process-local commit history that an opened root keeps
  for `since(clock)` and change polling
  ([`opened/journal.rs`](../../../../crates/fdu-core/src/opened/journal.rs)). It exists
  on `main` and starts empty whenever a snapshot is loaded.
- **FSEvents history:** the per-volume event store that macOS `fseventsd` keeps on disk
  across process exits and reboots.
- **History replay:** the planned mechanism that reads FSEvents history from a stored
  per-volume **replay cursor** to nominate scopes for fresh observation.

`Source::JournalScoped` is the engine’s provenance for values that history replay scoped
and nothing re-verified.

## What Exists Today

| Capability | Behavior on `main` | Consequence for daily comparison |
| --- | --- | --- |
| Metadata inventory | Each entry retains apparent and allocated bytes, mtime, ctime, inode, and device (`Attrs`); the index maintains directory roll-ups. No link count or clone identity is retained | Reuse these facts and reducers; hard links are detectable only by grouping `(dev, inode)` across an inventory |
| Default one-shot metadata report | `plan_report` reads the snapshot under `auto` and `read-only` only when content analysis is requested; a summary-only query retains no index and writes no snapshot | A cache does not currently avoid the next traversal |
| `open()` with a usable snapshot | Loads the image and reconciles every entry before returning | Reuse is still proportional to tree size |
| `--cache only` | Loads saved facts without filesystem verification and labels them stale | Useful for viewing an old inventory, not for discovering changes |
| Snapshot persistence | One replaceable flat image per root, at the current format version (`snapshot::FORMAT_VERSION`). `engine_fingerprint` mixes the crate version, format version, and classification version; a mismatch, or a stored scan scope that cannot serve the request, is a miss, and the next complete indexed scan replaces the image | No baseline survives an upgrade, a rules change, or a scope change; loading materializes the full index |
| Opened roots | Bounded change polling, verified multi-path refresh, and `since(clock)` over the index journal | The history is process-local; it does not recover a day of changes after process exit |
| FSEvents history replay | Planned in the FSEvents plan, with an uncommitted exploratory spike | No replay module and no replay cursor in the snapshot format |
| Partial scans | Report errors; `snapshot::save` refuses an incomplete index | `--allow-partial` changes exit acceptance only; a denied home-folder scan is not cacheable |

Source: [execution planning](../../../../crates/fdu-core/src/execution.rs),
[open and cache policy](../../../../crates/fdu-core/src/lib.rs),
[snapshot format](../../../../crates/fdu-core/src/snapshot.rs),
[engine contract](../../../../crates/fdu-core/src/engine_contract.rs), and
[cache guide](../../guides/cache-design.md).

Build this feature through the engine and mirror it in CLI and Python; do not make a
Python inventory replica or persist an opened handle’s session identity.

## User Workflow

The operations below define behavior, not committed command syntax.

1. **Capture a checkpoint.** Scan the selected scope, show coverage and filesystem free
   space, and publish checkpoint A with a newly minted immutable id.
   Optionally attach a label such as `before-upgrade`.
2. **Refresh on the next visit.** Resume the working inventory from its replay cursor,
   reconcile changed scopes, then publish checkpoint B with its own id.
   If replay is unavailable or insufficient, run the ordinary scan.
   A is unaffected either way.
3. **Compare A with B.** Show the largest positive and negative directory deltas,
   before/after bytes, file-count changes, and coverage.
   Expand a directory for the files or child directories responsible.
4. **Reuse or advance deliberately.** Re-reading A→B performs no refresh and produces
   the same answer. A separate refresh creates C; comparing A→C leaves A unchanged.
   Moving a label, for example `yesterday` from A to C, is an explicit operation.

### Checkpoint Identity

A checkpoint id is minted when the checkpoint is published and is never reused.
It is independent of `Clock`, `EngineVersion`, and session identity, whose reset rules
belong to a live process.

A label is a movable name for an id:

- A comparison may be requested by label or by id.
  The engine resolves labels to ids at request time, and every result records both
  resolved ids.
- Reading the same pair of ids again returns the same rows, or refuses with a typed
  error if either checkpoint is no longer retained.
  It never substitutes another checkpoint.
- Moving a label never modifies a checkpoint.
  The previous checkpoint stays addressable by id until retention removes it.
- A checkpoint is pinned while a label points to it or the caller has pinned it
  explicitly. Retention never removes a pinned checkpoint.

Refusing to reuse a label is the alternative; see [Open Questions](#open-questions).

### Delta Ranking

The default question is “which directories account for the largest storage changes?”
Rank by the absolute signed change in unique allocated bytes, defined under
[Delta Accounting](#delta-accounting), with a deterministic path tie-breaker.
Keep increases and decreases visible, offer per-path allocated and apparent bytes as
alternative rankings, name the measure in every output, and expose every output bound.
Compute deltas before selecting the largest rows; comparing yesterday’s top ten with
today’s top ten loses directories that newly became large.

### Interim Workflow

An immediately usable interim workflow is to save complete, dated directory reports:

```bash
fdu "$HOME/.cache" --one-filesystem --cache off \
  --view tree --depth all --limit all --format json > cache-before.json
```

Repeat to a different output path later and compare directory records by path, including
paths present in only one report.
Check `complete` and `errors` before subtraction.
Keep the reports outside the measured root.
Both sizes are already present in JSON. This supplies a baseline today, but both
captures still scan the selected tree, the sums count hard links and clones in full, and
fdu does not yet provide the comparison command.
`--modified-since` filters present files; it cannot recover old sizes or deleted files.
`--depth` and `--exclude` are report controls, not a promise to avoid scanning
descendants.

## Scope

Start with independently refreshable roots such as `$HOME/.cache`, `$HOME/.codex`,
`$HOME/.claude`, `$HOME/Library/Caches`, and the workspace directory.
Add the home folder once the same contract is measured at that scale.
A home inventory and a nested root may be alternative views, but their totals must not
be added together.

Each checkpoint records the identities that decide whether two checkpoints can be
compared; see [Checkpoint Store and Compatibility](#checkpoint-store-and-compatibility).
Include hidden and Git-ignored build artifacts: those are often the growth being
investigated. Control-state observation classifies entries; it never removes them from
byte totals. Use one filesystem per inventory initially, with symlinks not followed.

Include Trash when measuring the whole home folder.
Moving a cache to Trash decreases its source directory and increases Trash; it does not
by itself free the blocks.
Exclude the checkpoint store explicitly from the inventory and report its bytes
separately, so writing a checkpoint does not create an endless stream of self-generated
usage deltas. This requires an actual engine scan-scope exclusion, not only a display
filter.

## Delta Accounting

Each measure is computed per checkpoint and then subtracted:

| Measure | Counts | Hard links | APFS clones |
| --- | --- | --- | --- |
| Apparent bytes | `size` of every path entry | each path in full | each clone in full |
| Per-path allocated bytes | `allocated` of every path entry, as today’s roll-ups do | each path in full | each clone in full |
| Unique allocated bytes | `allocated` once per `(dev, inode)` within one checkpoint | once per inode | each clone in full |

**Hard links.** Within one checkpoint, regular-file entries that share `(dev, inode)`
are one file, so unique allocated bytes count it once.
A package manager that links an existing file into a new environment therefore does not
grow the checkpoint total.
Attributing that file to one directory needs a deterministic rule.
Slice 2 attributes it to the in-scope entry whose root-relative path sorts first in
bytewise path order (not by size), and marks every directory row whose subtree holds a
shared inode, including one whose other links lie outside the scope.
That rule is stable across two complete captures, but a new link that sorts earlier
moves the attribution: the delta shows a decrease at the old directory and an increase
at the new one, with net zero at their common ancestor.
Renaming one link of a multi-link file can move it with nothing physical changing,
because attribution follows whichever in-scope link sorts first after the rename.
If the attributed link is renamed so it no longer sorts first, the file’s full allocated
size arrives at another link’s directory, where no entry changed.
If another link is renamed so it now sorts first, the size leaves the previously
attributed link’s directory, where no entry changed either.
The durable rule, which must also survive incremental updates, is `fdu-579b`.
`(dev, inode)` is compared only within one checkpoint: device numbers are not stable
across reboots, and inode numbers are reused.

The engine retains no link count, so grouping would otherwise have to hash every regular
file by `(dev, inode)`. Slice 2 therefore adds the link count to the retained entry
facts in `fdu-core`, as a new field of `Attrs`, so only files with more than one link
enter the group. The metadata calls the scanner already makes can supply it: `st_nlink`
from `stat`, and `ATTR_FILE_LINKCOUNT`, which `getattrlistbulk` can request.
The snapshot writes each entry’s `Attrs` as fixed-width fields, so the same change
alters the record layout and bumps the snapshot `FORMAT_VERSION`; existing snapshots
become clean misses.

**Where file identity is not observed.** Unique allocated bytes need a real
`(dev, inode)` and a link count.
On Windows the scanner records `inode` and `dev` as zero, and `std::fs::Metadata` has no
stable link count there, so no file would enter the group and a “unique” total would
silently equal per-path allocated bytes.
A checkpoint captured where either fact is unavailable records unique allocated bytes as
not observed in its accounting version.
Its comparisons then rank by per-path allocated bytes and name that measure, and an
explicit request for unique allocated bytes is refused with the reason, so a per-path
figure is never presented as unique.

**APFS clones.** A clone is a distinct inode that shares blocks with its source, and
each reports its full allocated size.
The engine cannot see that sharing today, so for clone creation and deletion both
allocated measures are an upper bound on the physical change.
uv, which clones from its cache into environments by default on macOS, can show an
environment’s full size as growth while few blocks were written.
The bound runs one way only: a write into a cloned file can allocate new blocks without
changing its reported allocated size.
The macOS attribute API documents extended attributes that bear on this:
`ATTR_CMNEXT_PRIVATESIZE` (bytes not shared with a clone or snapshot, freed immediately
if the file were deleted), `ATTR_CMNEXT_CLONEID`, `ATTR_CMNEXT_CLONE_REFCNT`, and
`EF_MAY_SHARE_BLOCKS` in `ATTR_CMNEXT_EXT_FLAGS`. Whether `getattrlistbulk` returns them
on the target volumes, and at what cost, is unmeasured; using them is outside slice 2.

The default ranking is honest about these limits:

- Every comparison shows the volume free-space change recorded with each checkpoint
  (`statvfs`), labeled volume-wide.
- Rows whose subtree contains a shared inode are marked shared.
- On APFS, the output states that clone sharing is not visible and that allocated deltas
  can overstate physical change.
- A directory sum is not a promise of reclaimable space.
  Snapshots, purgeable data, open deleted files, and changes outside the scope can make
  it differ from the free-space change.

## Coverage and Denied Subtrees

Permission loss is unknown state, not removal.

- A subtree that a capture or refresh cannot read is recorded as a typed gap: its path,
  the error kind, and when it was last observed.
  A gap is unknown, not empty.
- A capture with gaps publishes a partial checkpoint marked with its gap list.
  The command’s exit status follows the existing partial-result acceptance
  (`--allow-partial`).
- A crossed control budget is not a gap.
  Sizes stay exact, so the checkpoint is complete and the exit status is unaffected;
  only its ignore classification is partial, which comparisons mark as described under
  [Checkpoint Store and Compatibility](#checkpoint-store-and-compatibility).
- A comparison shows a gap in either checkpoint as unknown for that subtree and marks
  every ancestor’s delta partial.
- A disappearance is established only by a successful reconciliation of its containing
  scope. A gap is cleared only by a complete scan of its subtree, never by replay,
  because events delivered while it was unreadable were not applied.

Typed gaps belong to the checkpoint format from slice 2. The working inventory, which
lives in the snapshot (see
[Persistent State and Idempotence](#persistent-state-and-idempotence)), needs them
before slice 3 can advance a replay cursor past a gap.
The snapshot’s complete-only rule keeps its meaning once they exist:

- `Coverage::Complete` still means every in-scope directory has a complete listing.
- Slice 3 adds a typed-gap section to the snapshot at a new `FORMAT_VERSION`.
  `snapshot::save` then persists a `Coverage::Partial(Inaccessible)` index only when
  every incomplete directory is recorded as a gap in that section.
- An index with an unrecorded gap, or one that is partial for any other reason (still
  building, budget, cancellation, failure), is still refused.
- Loading a gap-marked image restores it as partial, never as complete.

## Persistent State and Idempotence

Keep three kinds of state separate, each in a container that matches whether it can be
rebuilt:

| State | Lifetime and meaning | Container |
| --- | --- | --- |
| Working inventory | Latest reconciled entry facts, typed gaps, and persisted directory aggregates; replaced by each refresh | The root’s snapshot image in the cache directory; discarded by a release upgrade, `--cache-clear`, or a scope change |
| Replay cursor | Per-volume FSEvents progress used to discover work since the last refresh | The same snapshot image as the inventory it was fenced against; discarded with it |
| Checkpoint | Immutable comparison baseline with its id, recorded identities, coverage, capture interval, and free space; labels point to it | The checkpoint store under the user data directory; kept across upgrades |

The working inventory and replay cursor are derived state.
A full scan rebuilds both, so losing them costs time but no answer, and the snapshot
cache’s VERSION + FAIL FAST contract already serves that case.
Keeping the cursor in the image it describes gives the two one publication boundary,
where the FSEvents plan already places it.
A checkpoint cannot be rebuilt once the filesystem has moved on, so it lives in the
store.

**After a release upgrade,** `engine_fingerprint` no longer matches, so the snapshot is
a miss and the first refresh is a full scan.
That scan publishes a new inventory with a pre-scan cursor (the FSEvents plan’s G2).
Checkpoints captured before the upgrade stay retained, and they remain comparable with
the checkpoint the refresh publishes, because neither the crate version nor the snapshot
`FORMAT_VERSION` affects comparability.
`--cache-clear` has the same effect.
So does alternating between report and opened-root scopes, until `fdu-w3l5` keys
snapshots by scope.

Keeping the inventory in the checkpoint store instead would let replay survive an
upgrade. It would also make derived state user-owned data under the store’s SUPPORT BOTH
contract, so every inventory format change would carry a reader for as long as that
format is supported.
One full scan per release costs less.
Slice 4 changes the inventory’s encoding, not its container.

A week-old checkpoint may be compared using a recently refreshed cursor.
The checkpoint’s age must not force replay from a week ago.

Replay events nominate work; they are not byte increments.
Coalesce overlapping scopes, observe current filesystem facts, and apply conditional
replacements or removals through the existing engine reducer path.
Duplicate observations must be no-ops.
When a file grows from 100 to 160 bytes, its contribution changes by 60, whether the
event is delivered once or three times.
A created-then-deleted file absent at both checkpoints contributes zero net change;
intermediate churn is a different feature.

Rename detection is optional attribution.
A move contributes a decrease at the old path and an increase at the new one, with net
zero at their shared ancestor when allocation is unchanged.
Inode matches alone do not prove a rename across long gaps because of reuse and hard
links. Compute ancestor deltas from the same facts without adding both a parent’s
recursive total and its children’s totals to a grand total.

Capture a filesystem-event fence before a full scan.
Publish the resulting inventory and that conservative cursor together.
For a scoped refresh, advance the cursor only through work reconciled and committed;
overlapping historical events remain safe to reobserve.
Cancellation, an exhausted work budget, and failed persistence must not publish a cursor
past unapplied work.
A denied subtree is applied work once the committed inventory records it as a typed gap,
so the cursor advances past events inside it.
An age-forced sweep that meets the same denial still publishes its gap-marked inventory
and pre-scan cursor, and the refresh still publishes its partial checkpoint.
A persistently denied subtree therefore cannot hold the cursor back indefinitely.
Preserve hints arriving during refresh for the next pass.

Each committed working revision, its cursor, checkpoint references, and aggregate
changes need one recoverable publication boundary.
Define crash durability explicitly; atomic rename alone is only atomic visibility.
Concurrent refresh writers for the same root must serialize or conflict by revision.
Checkpoint reads remain immutable.

Share unchanged blocks between checkpoints or retain a compact change log; avoid one
full million-file copy per day.
Compaction preserves pinned checkpoints and atomically publishes its replacement.
Bound unpinned history by a stated retention policy; refuse an over-budget save rather
than silently removing a pinned checkpoint.
Final policy values require measurements.

## Checkpoint Store and Compatibility

The snapshot cache is built to be discarded.
`engine_fingerprint` includes the crate version, so every release misses every existing
snapshot, and a scope mismatch cold-scans and replaces the root’s single image.
A checkpoint keyed the same way would be lost on every upgrade, every classification
change, and every alternation between report and opened-root scopes (`fdu-w3l5`), none
of which a user would recognize as eviction.

**Separate store.** Checkpoints live in a store under the user data directory, not the
cache directory that users and cleanup tools treat as disposable.
The cache path, `engine_fingerprint`, and `--cache-clear` never address it.
A checkpoint may be captured from any complete or gap-marked index, but once published
it depends on no cache image.
Blocks shared with the working inventory (slice 4) follow the store’s lifecycle, not the
cache’s.

**Own format version.** The store has a checkpoint format version, independent of the
snapshot `FORMAT_VERSION`, the crate version, and `CLASSIFICATION_VERSION`. Each
checkpoint records its format version, id, root path and volume identity (defined
below), `ScopeIdentity`, `SemanticIdentity`, classification version, accounting version
(which measures it recorded), capture interval, coverage, and free space.
It also records the writing engine version, as information only.

**Volume identity.** A checkpoint’s volume identity is the stable UUID of the filesystem
volume that holds its root, never its device number.
`st_dev` is renumbered across reboots and remounts.
The engine’s `Attrs::dev` holds that number, or zero on Windows, and `Fingerprint::dev`
copies it; `fdu-4qtk` corrects the rustdoc that calls the latter a volume identity
component. The FSEvents plan follows the same rule for its replay cursor.

- **macOS:** the volume UUID the root reports through `getattrlist` (`ATTR_VOL_UUID`).
  The FSEvents plan’s cursor UUID is a different value: `FSEventsCopyUUIDForDevice`
  identifies the volume’s FSEvents database, which changes when that history is
  discarded or event IDs wrap, while the volume’s contents do not.
- **Linux:** no portable call returns one.
  A filesystem UUID is reachable only through the block device behind the mount, and
  `statfs`’s `f_fsid` is not stable on every filesystem.
- **Windows:** fdu observes none today.
  `dev` and `inode` are zero, and the volume serial number or GUID needs platform calls
  the scanner does not make.
- **Network and FUSE volumes:** may report no UUID on any platform.

Where no UUID is observed, the checkpoint records the volume identity as not observed.
A comparison in which either checkpoint lacks one skips the volume check and is decided
by root path and the other recorded identities.
Its result states that the volume was not verified, so a different volume mounted at the
same path is never presented as a verified comparison.

A remount that renumbers `st_dev` leaves A→B comparable: deltas are computed by path and
byte facts, and `(dev, inode)` pairs are grouped only within one checkpoint.
It does change every retained entry’s `Fingerprint`. A refresh that finds the root’s
device number changed therefore re-observes every entry before it publishes a
checkpoint, so hard-link grouping never meets one file under two device numbers.

**Upgrades.** Before any release writes checkpoints, a development build refuses an
older development format and says so.
Once a release writes checkpoints, they are user-owned data:

- The reader keeps every released checkpoint format readable for the comparison facts
  until that format’s support window closes, and never rewrites a checkpoint in place.
- Compaction, which already republishes blocks atomically, may re-encode what it copies
  in the current format, provided the A→B result of every retained id pair is unchanged.
  Nothing depends on it doing so: a pinned checkpoint that compaction never copies keeps
  the format it was written in.
- A checkpoint in a format the reader cannot read is refused, never skipped, with an
  error naming its id, its format version, and the remedy.
  For a newer format, the remedy is a release that reads it.
  For a retired older format, it is the range of releases that still read that format,
  or explicit removal.
- A format’s reader is retired only after a support window, a number of releases
  documented when the first release writes checkpoints, has shipped since the last
  release that wrote that format.
  The release that drops the reader names the last release that reads it in its release
  note. Retirement requires no re-encoding, so a store can still hold the format
  afterwards, for example in a pinned checkpoint that compaction never copied.
  That checkpoint meets the refusal above, so neither direction has a silent path.

**Scope and classification changes.** Whether a comparison is valid is decided per
measure, from the recorded identities:

- A different root, a different observed volume UUID, or a different `ScopeIdentity`
  (depth, symlink policy, filesystem boundary, hidden-entry policy, special files),
  refuses the comparison with a typed error naming the differing field.
  The two checkpoints admitted different entries, so no byte delta between them means
  anything. A volume identity missing from one or both checkpoints is not a difference;
  the result is marked volume-unverified instead.
- A different `SemanticIdentity` or classification version leaves the filesystem facts
  comparable: path, kind, apparent bytes, allocated bytes, and counts.
  Measures derived from classification, such as ignored and unignored partitions and
  type tallies, are reported as not comparable, with the reason, rather than as zero.
  For example, observing `.gitignore` control state by default on every surface
  (`fdu-elnn`) changes a report’s ignore-rules fingerprint without changing any byte
  count.
- A checkpoint captured without control observation has no ignored partition, and
  comparisons report that measure as not observed.
- A checkpoint captured over its control budget has a partially observed ignored
  partition. For 0.1.0, `fdu-1onj` makes crossing that budget, or the per-line guard the
  same setting raises, refuse the control source instead of ending the scan: sizes stay
  exact, the result is complete with exit status 0, and a coverage note names the
  directories whose control sources were refused.
  With `.gitignore` roll-ups on by default (`fdu-elnn`), a large home folder reaches it
  without any fault. Each checkpoint therefore records its control budget and every
  refused control source, read from the index, and a comparison applies these rules:
  - Byte and count deltas stay exact, whatever either checkpoint refused.
  - If either checkpoint refused a source, the ignored and unignored deltas are marked
    partial at every directory at or below a source refused in either checkpoint, and at
    each ancestor, with the refused sources named, the way a gap marks its ancestors.
    Equal refused sets do not lift the marker: below a source both checkpoints refused,
    neither applied its rules, so a new file there is counted in whichever partition the
    loaded rules choose.
  - A checkpoint that retains only a count of refused sources, not their paths, marks
    every classification delta of its comparisons partial.
  - The budget is mixed into `ignore_rules_fingerprint`, so checkpoints captured at
    different budgets differ in `SemanticIdentity`, and their classification is not
    comparable under the `SemanticIdentity` rule above.
    The recorded budget lets that reason name the two budgets.
    Two complete checkpoints whose classifications agree are also reported as not
    comparable when their budgets differ, which is conservative rather than wrong.
- A measure not recorded in both checkpoints is unavailable for that pair: for example,
  unique allocated bytes from before link counts were retained, or from a platform that
  observes no file identity.
  An explicit ranking by it is refused with a remedy, the default ranking falls back to
  per-path allocated bytes and names that measure, and the other measures remain
  available.
- The crate version and the snapshot `FORMAT_VERSION` never affect comparability.

Marking a partially observed classification partial, rather than refusing it, keeps the
comparison consistent with the coverage note a single report gives for the same state.
Case against: below a refused source the classification is not exact in either
direction, because a refused file can hold negations as well as ignore rules.
A partial number there can be wrong, not merely incomplete, and a reader who skips the
marker is misled. Over budget, the marker always reaches the root, so the top rows of a
home-folder comparison carry it.
Alternative: refuse classification-derived measures at the affected directories and
their ancestors. No number that may be wrong is shown, at the cost of withholding the
root’s ignored delta whenever any source below it was refused.

**Backward compatibility requirements:**

- **Internal code:** DO NOT MAINTAIN. Nothing outside the repository depends on
  checkpoint internals.
- **Library APIs:** DO NOT MAINTAIN. Slice 2 adds a link count to `Attrs`, a public
  struct with public fields that is not `#[non_exhaustive]` and is re-exported from the
  crate root, so a struct literal or exhaustive pattern outside the engine stops
  compiling. Neither `fdu-core` nor the Python package has been released (the changelog
  holds only unreleased changes), so every consumer is in this repository and updates in
  the same change. Revisit at the first release.
  The plan’s other engine, CLI, and Python APIs are additions.
- **Server APIs:** N/A.
- **Plugin and extension APIs:** N/A.
- **File formats:** The snapshot cache stays VERSION + FAIL FAST, where a mismatch is a
  clean miss; slice 2’s link count and slice 3’s typed gaps and replay cursor each bump
  its `FORMAT_VERSION`. The checkpoint store is VERSION + FAIL FAST until a release
  writes it, then SUPPORT BOTH at the reader for released versions.
  The protected data is retained checkpoints.
  Tests keep a stored checkpoint pair in each released format and assert its recorded
  A→B result. Support for a format ends after its documented support window, with a
  release note naming the last release that reads it, and a store still holding that
  format is refused with that remedy.
- **Persisted client state:** Labels and pins follow the checkpoint store.
- **Database schemas:** N/A.

## Refresh Mechanism and Cost

Use the [FSEvents plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) for macOS
change discovery and the existing bulk metadata reconciler for verification.
Spotlight may nominate additional paths or prioritize a preview.
Its asynchronous, selective index cannot establish complete cross-run additions,
deletions, or allocated usage.
The
[research update](../../research/research-2026-08-10-performance-frontier.md#daily-disk-usage-comparison-2026-09-13)
records the source and local evidence behind that decision.

```text
working inventory + replay cursor
             | FSEvents history replay
             v
      normalized dirty scopes
             | fresh metadata observations
             v
  engine updates facts and ancestor totals
             | atomic durable publication
             v
working inventory + replay cursor + checkpoint B
             |
             + checkpoint A --> signed directory/file deltas
```

Let N be retained entries, E replayed events, D entries enumerated in dirty scopes, and
A affected ancestors.
Whole-command latency includes:

```text
open needed state + replay(E) + reconcile(D) + update(A)
    + persist changed state + produce requested delta rows
```

The current flat loader and writer still cost O(N). History replay alone therefore
cannot deliver an end-to-end O(changes) claim.
Reuse the planned block format and durable mutation storage (`fdu-xihx`, `fdu-pdra`,
`fdu-3dtq`), including persisted roll-ups and lazy access.
A flat-format replay prototype is useful for measuring avoided traversal, but must
report its remaining load and rewrite cost.

Changed-directory enumeration can itself be large: one changed file in a directory of
100,000 siblings may require that directory to be relisted under the initial policy.
Replay cost also depends on cursor age and volume-wide event traffic, not only files
changed under the root.
Coalescing and bounds must expose these costs and choose the scan fallback when it is
cheaper. Queries over arbitrary old checkpoints may have to scan their retained change
range; fast stored comparisons and bounded top-change queries need their own measured
read path.

Replay-scoped results carry `Source::JournalScoped` provenance.
A successful `HistoryDone`, matching UUID, or age bound does not prove complete history.
A requested fully verified result uses the scan path.
If a command’s work budget is exhausted, preserve its baseline and report incomplete
work; do not relabel the old answer as current.

## Delivery Slices and Acceptance

| Slice | Deliverable | Acceptance |
| --- | --- | --- |
| 1. Reproducible replay probe (`fdu-uwhl`) | Commit the probe, exact flags, cursor fences, and machine-readable results; compare with and without `FullHistory` | Deep append, deletion, move, restart, overlap, and controlled loss agree with an independent scan or produce a declared degraded result |
| 2. Checkpoint store and comparison contract (`fdu-8ybz`) | Engine-native store with ids, labels, pins, typed gaps, recorded identities, and the three accounting measures, including retained link counts (a new `Attrs` field and a snapshot `FORMAT_VERSION` bump); initially from scanned state | Repeated reads of one id pair are identical; moving a label or refreshing never changes a checkpoint; a removed checkpoint is refused, not substituted; an incompatible scope is refused; the hard-link, clone, and denied-subtree cases below produce their expected deltas; native, CLI, and Python surfaces agree |
| 3. Incremental macOS refresh | Cursor encoding and gates (`fdu-2cdv`), replay (`fdu-3tun`), scoped reconciliation (`fdu-rvje`), and the snapshot’s typed-gap section and save rule, at a new `FORMAT_VERSION` | Persist/restart/refresh tests and failure injection prove no lost cursor work or duplicate accounting; a persistently denied subtree does not stop the cursor advancing; after an engine fingerprint change the first refresh scans in full, and its checkpoint compares with one captured before |
| 4. Bounded persistent access | Block/lazy inventory and durable changes with checkpoint-aware compaction | Quiet refresh does not decode or rewrite all N entries; retained state stays within its declared budget |
| 5. Large-home workflow | Measure full capture, day-gap refresh, delta read, and fallback at realistic churn | Publish paired results against a fresh scan, with all work and coverage reported |

Slice 2 can provide useful comparisons before the replay optimization.
Slice 4 is needed before claiming fast whole-home refresh independent of inventory size.
The slices build on the opened-root lifecycle that the
[opened-root inventory engine plan](plan-2026-08-25-fdu-opened-root-inventory-engine.md)
delivered to `main`. Slice 2 also depends on three adjacent contracts.
Before slice 2 fixes its API, confirm on `main` that:

- the index journal is bounded in bytes (`journal_capacity_bytes`), as proposed in
  [#56](https://github.com/jlevy/fdu/pull/56);
- an index built without control state answers with a typed not-observed value rather
  than as if nothing were ignored, as proposed in
  [#57](https://github.com/jlevy/fdu/pull/57), which is what a checkpoint captured
  without control observation records;
- a crossed control budget leaves the index, including one loaded from a snapshot, a
  typed record of its budget and refused control sources (`fdu-1onj`), which is what a
  checkpoint with partially observed classification records.

Use the existing [performance loop](../../guides/performance-loop.md) and predeclare
accept rules before trials.
Measure 100k, 1M, and multi-million-entry trees, including real package caches and a
home-folder corpus, with quiet, ordinary, and install/build churn.
Test gaps of 1 hour, 24 hours, 48 hours, and 7 days; repeat near the age-policy
boundary. A next-day check must not be described as incremental when the policy simply
forces it to scan.

Record first-result and completion latency, snapshot bytes read/written, entries
enumerated and verified, replayed events, dirty scopes, peak RSS, retained-store growth,
coverage, and fallback reason.
Seconds-scale day-gap completion is the product target, not an existing benchmark
result. Record the full-scan oracle separately from the timed candidate path, and use
immutable or quiesced subjects for exact equality; a live home scan is an operational
observation rather than a simultaneous filesystem snapshot.

Correctness cases include in-place append with unchanged directory mtimes,
allocated-only changes, file and subtree deletion, rename across sibling and monitored
roots, duplicate/overlapping events, sparse files, ignored and hidden paths, permission
loss and restoration, event loss, concurrent writers, and crashes before and after
cursor publication. Volume identity cases: a remount or reboot that renumbers `st_dev`
stays comparable and re-observes every entry; a different volume UUID at the same root
path is refused; and a volume that reports no UUID compares as volume-unverified.
Classification cases: a control budget crossed in either checkpoint leaves every byte
delta exact and marks the ignored and unignored deltas partial at each directory at or
below a refused source and at its ancestors; a source refused at A and loaded at B is
marked partial, never shown as bytes moving between partitions; and checkpoints captured
at different budgets report classification as not comparable.
Accounting cases have stated expected deltas:

- **Hard link added to an existing in-scope file:** per-path allocated grows by the
  file’s allocated size at the new link’s directory; unique allocated is unchanged in
  total, and its attribution moves only if the new link sorts first.
- **Hard link added in scope to a file outside the scope:** for example, uv on Linux,
  which hard-links from its cache by default, installing from a cache outside the root
  into an environment inside it.
  Both allocated measures grow by the file’s allocated size at the new link’s directory,
  and the row is marked shared.
- **One link of a multi-link file renamed:** apparent and per-path allocated bytes move
  from the old link’s directory to the new one.
  Each measure nets to zero at the common ancestor of the directories it moves between.
  Unique allocated is unchanged in total, and its attribution follows whichever in-scope
  link sorts first after the rename, which has four outcomes:
  - The attributed link is renamed and still sorts first: attribution moves with it, as
    per-path bytes do.
  - The attributed link is renamed and no longer sorts first: the file’s full allocated
    size moves to the directory of the link that now sorts first, where no entry
    changed.
  - Another link is renamed and still sorts after the attributed one: attribution does
    not move.
  - Another link is renamed and now sorts first: the size moves to its new directory
    from the previously attributed link’s directory, where no entry changed.
- **Last remaining link removed:** both allocated measures shrink by the file’s
  allocated size.
- **Clone of an existing file:** both allocated measures grow by the clone’s allocated
  size; the recorded free-space change can be near zero.
- **Write into a cloned file without changing its size:** allocated measures are
  unchanged; free space can decrease.
- **Subtree denied at B but readable at A:** the subtree and its ancestors report
  unknown or partial deltas, not removal.
- **Capture on a platform without file identity (Windows today):** unique allocated
  bytes are not observed, and the comparison ranks by per-path allocated bytes under
  that name.

Compaction and retention must preserve every pinned checkpoint and the A→B result of
every retained id pair.
Exercise failure without materializing cloud-only files.
A resident observer is an optional later optimization; it does not replace the
cross-process, next-day acceptance test.

## Follow-Ups

- `fdu-b9d5`: the `Source::JournalScoped` rustdoc still states that FSEvents silently
  drops history; align it with the interpretation the FSEvents plan’s Phase 0 findings
  support, using slice 1’s evidence.
- `fdu-579b`: the durable hard-link attribution rule that survives incremental updates.
- `fdu-4qtk`: the `Fingerprint::dev` rustdoc calls the device number a volume identity
  component; say it is not stable across reboots or remounts.
- `fdu-w3l5`: scope-keyed snapshots, which reduce cache churn but do not make the cache
  a checkpoint store.

## Open Questions

These need a decision from the user.
The plan above follows each recommendation.

1. **Default delta measure.** Recommendation: unique allocated bytes, with per-path
   allocated and apparent bytes selectable and the free-space change always shown.
   Case against: it needs retained link counts, which change the engine’s entry facts
   and snapshot format.
   Its directory attribution can move a file’s full size into or out of a directory
   where nothing changed, when a link is added or one link of a multi-link file is
   renamed. On clone-populated macOS caches neither allocated measure removes the
   overstatement. Alternative: per-path allocated bytes, which the roll-ups already
   report at no new cost.
2. **Label reuse.** Recommendation: labels move and comparisons bind to ids.
   Case against: the same command, such as `yesterday` against `today`, returns
   different answers on different days, and nothing in the invocation shows it.
   The resolved ids appear only in the output, so a saved command is not a saved
   question. Alternative: refusing to reuse a name makes a name an id and removes the
   resolution step, but forces dated names on the rolling `yesterday` workflow and still
   needs a separate notion of the latest checkpoint.
3. **Released checkpoint formats.** Recommendation: keep each released format readable
   for a documented support window, never rewrite a checkpoint in place, and refuse a
   retired format with the releases that read it.
   Case against: every retained reader is a compounding cost.
   Each later format change has to be tested against a stored checkpoint pair in every
   released format for as long as that format is supported, which the backward
   compatibility requirements commit to.
   And once the window closes, a pinned baseline that compaction never copied becomes
   unreadable to current releases, however deliberately it was kept.
   Alternative: migrating on upgrade keeps one reader, at the cost of rewriting
   user-owned data, which then has to be atomic and verified against earlier comparison
   results.
4. **Partial checkpoints.** Recommendation: publish a gap-marked partial checkpoint and
   let `--allow-partial` decide the exit status.
   Case against: two partial checkpoints with different gap sets mark every shared
   ancestor’s delta partial.
   A rolling label such as `yesterday` can land on a partial checkpoint, so the daily
   workflow degrades without anyone opting in, and the label then pins that partial
   checkpoint against retention.
   Alternative: requiring an explicit opt-in keeps every stored checkpoint complete, at
   the cost of no baseline at all for a home folder with one protected subtree.

## References

- [Cache design](../../guides/cache-design.md)
- [Opened-root inventory engine](plan-2026-08-25-fdu-opened-root-inventory-engine.md)
- [FSEvents-scoped revalidation](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
- [Performance frontier](../../research/research-2026-08-10-performance-frontier.md)
- [Campaign 2, Phase D](plan-2026-08-23-fdu-performance-campaign-2.md#phase-d-the-warm-end-state-after-b-because-the-representation-decides-the-format)
- [Future persistence roadmap](../future/plan-2026-08-09-fdu-post-phase-1-roadmap.md)
- [Surface architecture](../../architecture/fdu-surface-architecture.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
