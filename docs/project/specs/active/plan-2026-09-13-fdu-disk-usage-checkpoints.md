# Feature: Disk-Usage Checkpoints and Daily Deltas

**Date:** 2026-09-13

**Status:** Proposed.
Research and implementation plan; no new commands or persistence format ship with this
document. This work follows `0.1.0`: nothing in it is part of that release.

## Objective

Save an inventory of a home folder or selected roots, return a day later, and identify
which directories gained or lost bytes without enumerating millions of unchanged files.
Comparing the same two checkpoints, identified by their immutable ids, must return the
same net changes or refuse explicitly; it must never return a different answer.

The first inventory requires a scan.
Subsequent refreshes should verify changed scopes, update their ancestors, and read or
write only the persisted state they need.
The performance target covers the whole command, including loading and saving state.

This is an additional, opt-in disk-history workflow.
Ordinary fast roll-ups of a known directory and cache-assisted content calculations,
such as source-line counts, retain their existing requests and behavior.
Neither starts checkpoint retention, scheduled capture, or journal-scoped verification
implicitly. Disk history uses the same engine’s filesystem facts and roll-ups, with a
separate comparison request and durable before-state.

fdu may exit after each capture or refresh.
macOS maintains FSEvents history while fdu is absent; the next process replays from the
inventory’s saved cursor.
A resident watcher is optional.
Replay availability and retained history determine whether that next visit can avoid a
sweep, as specified by the
[cross-process replay design](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md#cross-process-lifecycle).

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
| Default one-shot metadata report | `plan_report` reads the snapshot under `auto` and `read-only` only when content analysis is requested; a summary-only query that also turned `.gitignore` observation off (`--no-gitignore`) retains no index and writes no snapshot, while a default summary observes and retains the full index | A cache does not currently avoid the next traversal |
| `open()` with a usable snapshot | Loads the image and reconciles every entry before returning | Reuse is still proportional to tree size |
| `--cache only` | Loads saved facts without filesystem verification and labels them stale | Useful for viewing an old inventory, not for discovering changes |
| Snapshot persistence | One replaceable flat image per root, at the current format version (`snapshot::FORMAT_VERSION`). `engine_fingerprint` mixes the crate version, format version, and classification version; a mismatch, or a stored scan scope that cannot serve the request, is a miss, and the next complete indexed scan replaces the image | No baseline survives an upgrade, a rules change, or a scope change; loading materializes the full index |
| Opened roots | Bounded change polling, verified multi-path refresh, and `since(clock)` over the index journal | The history is process-local; it does not recover a day of changes after process exit |
| FSEvents history replay | Proposed integration with a standalone probe; controlled shallow refresh matches its metadata oracle, but busy-root replay has stable misses and day-old completion has a long variable tail | No engine replay module or replay cursor in the snapshot format; correctness and whole-command latency acceptance remain open |
| Partial scans | Report errors; `snapshot::save` refuses an incomplete index | `--allow-partial` changes exit acceptance only; a denied home-folder scan is not cacheable |

Source: [execution planning](../../../../crates/fdu-core/src/execution.rs),
[open and cache policy](../../../../crates/fdu-core/src/lib.rs),
[snapshot format](../../../../crates/fdu-core/src/snapshot.rs),
[engine contract](../../../../crates/fdu-core/src/engine_contract.rs), and
[cache guide](../../guides/cache-design.md).

Build this feature through the engine and mirror it in CLI and Python; do not make a
Python inventory replica or persist an opened handle’s session identity.

### September 27 spike decision

The
[replay continuation](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md#shallow-refresh-and-completion-continuation-2026-09-27)
demonstrates changed-scope observation on a controlled 20k-entry tree, but does not
validate this plan’s complete agent-coding workflow.
Two busy-root trials missed stable changes despite completed replay and no degradation
signal. A missed growing log was still open for writing.
An aged controlled file then reproduced the failure: append and `fsync` while open
yielded no change event despite completed replay; closing it and replaying the same
cursor yielded the event and exact agreement.

The workflow must account for active logs and databases, not merely files that have been
closed. Native active-writer discovery is a research candidate, with explicit
permission/coverage and cost limits, not an implemented fix.
A recent-file heuristic or skill-selected folder list cannot establish completeness.
Keep profile defaults in the skill, while the engine owns generic coverage,
verification, and fallback semantics.

Historical completion can also cost tens of seconds or exceed a diagnostic minute even
when the changed fixture is tiny.
Compare total refresh cost against fdu’s full scan; do not promise seconds-scale daily
results from the event callback latency alone.
These findings do not change immutable checkpoint comparisons or authorize background
monitoring. They gate the accelerated refresh route used to produce a new checkpoint.

The
[change-source review](../../research/research-2026-09-27-disk-growth-change-sources.md)
adds measurements that bear on sequencing:

- **Loading the flat snapshot costs about as much as a walk.** On a 452k-entry root it
  took 3.15 s, against a 5.7 s walk; at 1.5 M entries, 10.0 s against 18.6 s.
- **A delta-only roll-up log is enough to diff.** Across 14 minutes of agent activity it
  touched 4,618 of 95,500 directories, and the diff took 0.077 s.
- **The interim JSON workflow is heavy at this scale.** It costs 498 MB and 14–27 s per
  checkpoint at 452k entries.
- **Spotlight cannot nominate paths on this host.** Indexing is off on the Data volume.

The review recommends fixing the watcher first (one-sided rename scoping, persistence
cadence, and the open-writer list), shipping walk-captured checkpoints for small scopes
(slice 2), and prototyping a resident dirty-directory recorder for home scale.
Slice 4 follows only if the recorder’s flat load then dominates.
One-shot replay stays for gap recovery, and APFS directory statistics are parked.

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

### Scope Defaults Belong in a Brief Skill

The engine accepts explicit roots and coverage rules; it does not embed a list of
folders to monitor.
A brief disk-history skill should suggest a scope, resolve it for the
host, show the selected roots, and save the user’s choice as profile data.
These are workflow defaults, not automatic scanning or scheduling:

| User’s question | Suggested scope |
| --- | --- |
| What grew in this project? | The selected project; offer associated external build/cache locations separately |
| What grew in my home folder? | The user’s home, including hidden application and agent directories |
| What used my development disk space? | Home plus the host’s actual temporary locations, deduplicated |
| What else explains the volume’s space drop? | Offer additional explicit roots, such as applications or external build storage, with coverage gaps visible |

Resolve aliases before recording scope: on macOS `/tmp` and `/private/tmp` normally name
one location, while the per-user temporary directory may be elsewhere.
Do not count overlapping roots twice or follow symlinks into unselected storage.
For disk-pressure attribution, explain that excluding ignored build outputs can hide the
cause; any choice to include them is explicit scan scope, not a global change to
ordinary fdu defaults.
Persist roots, exclusions, volume boundaries, retention, and any opted-in schedule so
subsequent runs are reproducible without reinterpreting prose.
The skill supplies suggestions and best practices; the engine owns validation,
accounting, identity, and trust on every surface.
Until the checkpoint API ships, the skill must describe this as proposed behavior and
use only the documented interim report workflow, not invent checkpoint commands.

### Disk-pressure workflow: where did the space go?

The motivating question is: **“Available space is falling; which directories grew or
shrunk over the last hour or day?”** The optional disk-history profile (`fdu-vw9r`)
tracks a user-selected set of roots, for example Home, `/Applications`, and
`/private/tmp`. It does not scan those roots on every ordinary fdu invocation.

The primary workload is agent-assisted development: several GB accumulate within an hour
or day in Cargo targets, dependency environments, package caches, worktrees, agent
artifacts, or temporary build trees.
With an established baseline, the product target is seconds to useful current
attribution, not minutes spent rewalking the home folder.
This is a target for the complete diagnostic path, not a claim about today’s code or a
guarantee under arbitrary churn or missing journal history.

| Stage | Work | Result and guarantee |
| --- | --- | --- |
| Establish a baseline while space is healthy | Scan each selected root and retain its inventory, roll-ups, coverage, checkpoint, and pre-scan fence | Known before-state; the initial scan is unavoidable |
| Preserve useful time boundaries | On demand, or through an explicitly enabled capture schedule, refresh and retain hourly/daily checkpoints | Actual capture intervals, not timestamps inferred from file mtimes |
| Diagnose a space drop | Read volume availability, load baseline summaries, replay eligible histories, and observe dirty scopes | Ranked growth and shrinkage, with actual comparison interval and trust |
| Drill into a cause | Read retained child deltas; discover or verify missing scopes as needed | Details tied to the same checkpoint pair; new observations create a new checkpoint |
| Check the broader explanation | Compare directory changes with the volume-wide availability change | Named path changes plus an explicit unaccounted difference, not a promise that directory sums equal physical allocation |

For example, a current checkpoint captured around 15:00 can compare against the retained
14:00 checkpoint for an hourly view and yesterday’s 15:00 checkpoint for a daily view.
Capture is an interval, not an atomic filesystem snapshot: display the actual start and
completion times for both ends.
Resolve a relative window against one fixed request time, then select the closest
eligible checkpoint at or before the requested boundary and report its actual age.
Eligibility uses capture completion, not capture start or publication time.
A capture that straddles the boundary is ineligible for that before-state.
For a root-set manifest, every root used in the comparison must satisfy this rule;
missing or ineligible roots remain explicit gaps in that window.
If there is no suitable retained checkpoint, say that the requested window is
unavailable and offer an explicitly different available interval or a new baseline.
Do not silently label a three-day comparison “last hour,” interpolate old byte counts,
or substitute the current modification-time filter.
The permitted boundary tolerance and retention count are explicit profile settings;
their defaults require storage and latency measurements.

FSEvents can nominate paths changed while fdu was stopped, but it cannot reconstruct
their byte counts at an arbitrary past hour.
A profile used only today and last week supports that actual interval, not an exact
hourly or daily breakdown.
An optional scheduled invocation preserves the desired checkpoints without keeping an
fdu process running continuously.
Lightweight volume free-space samples may run more often, but a free-space sample alone
does not create the directory before-state needed for attribution.
Sleep, missed runs, expired history, and failed captures leave visible gaps.

The report ranks all comparable directory deltas before applying its display limit and
keeps both positive growth and negative shrinkage visible.
Show before/after allocated bytes, apparent bytes where useful, file-count changes,
coverage, verification mode, and the resolved checkpoint ids.
The initial layout should show a non-overlapping level of each root; expansion replaces
a parent row with its detail or labels the parent as a subtotal, never adds both.
A whole-scope ranking requires refreshing all relevant covered scopes and still carries
the delivered verification level.
Early rows from cached state are explicitly historical; rows from only some completed
scopes are provisional, not the globally largest current changes.

“Why” initially means filesystem attribution: which paths grew, shrank, appeared,
disappeared, or may have moved.
It does not identify the responsible process or prove the user action that caused it.
Events are not an accounting ledger: a temporary file created and deleted between both
checkpoints contributes zero net growth, even if it briefly exhausted space.
Explain that distinction when a user asks for intermediate churn rather than net change.

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
- A retained root-set manifest protects every referenced root checkpoint and its backing
  blocks from collection, whether or not each root is independently pinned.
  Retention removes an eligible manifest before reclaiming its otherwise unreferenced
  dependencies.

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

### Root sets, aliases, and shared volumes

Home, `/Applications`, and `/private/tmp` are proposed convenience selections, not an
implicit whole-machine scan or a change to the existing single-root API. Resolve root
aliases at profile admission, preserving the spelling the user supplied.
Other useful explicit selections are the resolved per-user `TMPDIR` and configured
external scratch or build roots: `/private/tmp` alone does not cover every tool’s
temporary storage, and Home does not cover an external build volume.
On macOS, `/tmp` normally resolves to `/private/tmp`; if both resolve to the same root,
keep one inventory and show the alias instead of counting it twice.
Confirm filesystem/root identity rather than relying solely on string spelling, and
revalidate it on each open.
A retargeted alias, replaced root, or different volume must not inherit the previous
root’s replay state.
This root admission does not imply following symlinks found inside a scanned tree.

Use one canonical owner for any overlapping subtree.
Selecting both Home and a cache below it provides a drill-down view of one inventory,
not two additive totals.
If independently retained roots overlap, the profile must project them into disjoint
coverage or refuse an aggregate; neither root’s checkpoint may be silently rewritten.

A root-set checkpoint is an immutable manifest of root checkpoint ids, scan scopes,
capture intervals, coverage, and volume samples.
Roots may finish at different times and the manifest must expose that fact.
If one root is denied, missing, cancelled, or offline, another root may still report its
completed comparison; the root set is partial, and an older value must not be presented
as newly refreshed. A failure to publish the manifest leaves its previous revision
intact. Publishing the manifest and protecting its dependencies must be ordered so
collection cannot race publication and leave a retained manifest with missing root
checkpoints.

Replay and inventory commit boundaries remain per root initially.
Roots sharing a volume may eventually share a replay read as an optimization, but
refreshing Home must never advance the applied cursor for an unrefreshed temporary root.
Different volumes always have independent journal identities and progress.

Sample available space once per distinct filesystem identity at each root-set capture,
with its own timestamp, not once per directory and then sum the samples.
APFS volumes can also share container capacity, so per-volume available figures are not
independent capacity to add together.
Per-root unique-allocated totals deduplicate links only inside that root.
Do not present their sum as globally unique unless the root-set reducer deduplicates
hard links across the union with a defined attribution rule.
Even that union cannot resolve APFS clone or snapshot sharing; the physical-space caveat
under Delta Accounting still applies.

### Entry coverage

Each checkpoint records the identities that decide whether two checkpoints can be
compared; see [Checkpoint Store and Compatibility](#checkpoint-store-and-compatibility).
Include hidden and Git-ignored build artifacts: those are often the growth being
investigated. Control-state observation classifies entries; it never removes them from
byte totals. Use one filesystem per inventory initially, with symlinks not followed.

Include Trash when measuring the whole home folder.
Moving a cache to Trash decreases its source directory and increases Trash; it does not
by itself free the blocks.
Exclude both the working cache and checkpoint store explicitly from the inventory and
report their bytes separately, so writing a checkpoint does not create an endless stream
of self-generated usage deltas.
This requires an actual engine scan-scope exclusion, not only a display filter.
The disk-history profile requests that exclusion explicitly and records it in scope
identity; ordinary directory requests do not acquire a new implicit exclusion.

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
file by `(dev, inode)`. Slice 2 therefore adds the link count to retained entry facts in
`fdu-core`, so only files with more than one link enter the group.
Its public representation needs compatibility review against released `Attrs`; a new
field must not silently break existing struct literals or exhaustive patterns.
The metadata calls the scanner already makes can supply it: `st_nlink` from `stat`, and
`ATTR_FILE_LINKCOUNT`, which `getattrlistbulk` can request.
The snapshot writes entry facts as fixed-width fields, so persisting the added fact
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
So does a request that turns observation off (`--no-gitignore`, `read_controls=False`)
or otherwise changes the scan scope, until `fdu-w3l5` keys snapshots by scope; a default
report, `open`, and `--watch` share one scope.

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
change, and every change of scan scope, such as turning `.gitignore` observation off
(`fdu-w3l5`), none of which a user would recognize as eviction.

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
below), `EntryScope`, `SemanticIdentity`, classification version, accounting version
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

- A different root, a different observed volume UUID, or a different `EntryScope`
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
- A checkpoint captured over its control limits has a partially observed ignored
  partition. For 0.1.0, `fdu-1onj` makes crossing the budget or the line limit, two
  independent settings, refuse the control source instead of ending the scan: sizes stay
  exact, the result is complete with exit status 0, and a coverage note names the
  directories whose control sources were refused.
  With `.gitignore` roll-ups on by default (`fdu-elnn`), a large home folder reaches the
  budget without any fault.
  Each checkpoint therefore records both control limits and every refused control source
  with the limit that refused it, read from the index, and a comparison applies these
  rules:
  - Byte and count deltas stay exact, whatever either checkpoint refused.
  - If either checkpoint refused a source, the ignored and unignored deltas are marked
    partial at every directory at or below a source refused in either checkpoint, and at
    each ancestor, with the refused sources named, the way a gap marks its ancestors.
    File rows shown below a refused source carry the same marker as their directory.
    Equal refused sets do not lift the marker: below a source both checkpoints refused,
    neither applied its rules, so a new file there is counted in whichever partition the
    loaded rules choose.
  - A checkpoint that retains fewer refused-source paths than its refused count, whether
    none or a truncated list (`Index::control_coverage` lists at most
    `MAX_RETAINED_ISSUES` of them beside the exact count), marks every classification
    delta of its comparisons partial: an unrecorded source could lie under any
    directory.
  - Both limits are mixed into `ignore_rules_fingerprint`, so checkpoints captured under
    different limits differ in `SemanticIdentity`, and their classification is not
    comparable under the `SemanticIdentity` rule above.
    The recorded limits let that reason name the limits that differ.
    Two complete checkpoints whose classifications agree are also reported as not
    comparable when their limits differ, which is conservative rather than wrong.
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
- **Library APIs:** Preserve the released 0.1.0 surface for compatible releases.
  `Attrs` is public with public fields and is not `#[non_exhaustive]`; adding a link
  count directly breaks external struct literals and exhaustive patterns.
  Prefer private retained facts or an additive observation API; if implementation needs
  a breaking shape, require a deliberate versioned API change and migration guidance.
  The first release is recorded in the [changelog](../../../../CHANGELOG.md).
  Checkpoint, comparison, and verification-policy APIs are proposed additions.
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
Its proposed verification policy keeps full scans as the default and makes accepting
journal-scoped evidence explicit.
Record each checkpoint’s coverage and trust, including the last full-verification time.
A comparison inherits the weaker trust of either baseline; repeatability of a stored
comparison does not make its source data verified.
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
The request’s verification policy is independent of cache policy and must agree across
Rust, CLI, and Python.
Persisted state must not promote a journal-scoped checkpoint or working inventory to
verified status on reload.
Every human and machine report exposes the guarantee actually delivered.
The proposal reuses the existing `watch` build feature for the macOS adapter; this does
not require fdu to stay running.
If a command’s work budget is exhausted, preserve its baseline and report incomplete
work; do not relabel the old answer as current.

### Low-space latency and bounded writes

The diagnostic should first obtain the cheap volume sample and retained summary, then
spend filesystem work on dirty scopes and requested drill-downs.
A retained summary is immediately useful context but must display its capture time.
Do not automatically run content analyzers such as source-line counting in this
workflow; metadata byte deltas answer the storage question without reading file
contents. Use fdu’s bulk scanning and adaptive parallelism when a scope needs discovery
or a full fallback. Missing before-state remains missing even after that discovery
succeeds.

The fast path still needs complete coverage or explicit gaps: a shallow scan of Home
cannot supply a full-home byte total, and filling only selected subtrees cannot prove
the global top-growth ranking.
Later progressive serving may prioritize requested or historically high-churn scopes,
but unvisited scopes retain an unknown/stale status until reconciled.
It must obey the engine’s mixed-provenance contract before becoming a public path.

Set explicit work and state-growth budgets for low-space operation.
Avoid rewriting a full inventory simply to report a small change; bounded block
publication is part of the product’s latency and disk-headroom requirements.
Keep a previously valid checkpoint if publication runs out of space, and never advance
its cursor independently.
A current in-memory result may be reported with a persistence-failed outcome, but it
must not be given a durable checkpoint id or made the baseline for future comparisons.
Refuse compaction when its temporary-copy headroom would violate the reserve.
Support a configured durable store on another volume; if unavailable, report the
limitation rather than silently falling back to the nearly full volume.
Durable checkpoint data must not be placed in disposable temporary storage.

Measure time to the first *current* useful attribution separately from time to display a
cached summary and from time to complete all roots.
Record volume sampling, replay, reconciliation, state reads/writes, and rendering
separately. A fallback may be slower than a fresh scan after paying replay overhead;
report that outcome rather than excluding it from the speed comparison.

## Delivery Slices and Acceptance

| Slice | Deliverable | Acceptance |
| --- | --- | --- |
| 1. Reproducible replay probe (`fdu-uwhl`) | Commit the probe, exact flags, cursor fences, and machine-readable results; compare with and without `FullHistory` | Deep append, deletion, move, restart, overlap, and controlled loss agree with an independent scan or produce a declared degraded result |
| 2. Checkpoint store and comparison contract (`fdu-8ybz`) | Engine-native store with ids, labels, pins, typed gaps, recorded identities, and the three accounting measures, including retained link counts (compatibility-reviewed API and snapshot `FORMAT_VERSION` bump); initially from scanned state | Repeated reads of one id pair are identical; moving a label or refreshing never changes a checkpoint; a removed checkpoint is refused, not substituted; an incompatible scope is refused; the hard-link, clone, and denied-subtree cases below produce their expected deltas; native, CLI, and Python surfaces agree |
| 3. Incremental macOS refresh | Cursor encoding and gates (`fdu-2cdv`), replay (`fdu-3tun`), scoped reconciliation (`fdu-rvje`), and the snapshot’s typed-gap section and save rule, at a new `FORMAT_VERSION` | Persist/restart/refresh tests and failure injection prove no lost cursor work or duplicate accounting; a persistently denied subtree does not stop the cursor advancing; after an engine fingerprint change the first refresh scans in full, and its checkpoint compares with one captured before |
| 4. Bounded persistent access | Block/lazy inventory and durable changes with checkpoint-aware compaction | Quiet refresh does not decode or rewrite all N entries; retained state stays within its declared budget |
| 5. Large-home workflow | Measure full capture, day-gap refresh, delta read, and fallback at realistic churn | Publish paired results against a fresh scan, with all work and coverage reported |
| 6. Multi-root disk-pressure profile (`fdu-vw9r`) | Optional selected-root capture and hour/day checkpoint selection, volume samples, and bounded low-space diagnostics | Root aliases/overlap do not double-count; mixed-age roots and missing time boundaries are explicit; rankings and free-space attribution obey the accounting contract; ordinary roll-ups and cached content analysis are unchanged |

Slice 2 can provide useful comparisons before the replay optimization, and a
complete-root replay prototype can be evaluated independently of the checkpoint store.
Production refresh on persistently inaccessible home-folder subtrees requires the
typed-gap publication contract in slice 3. Slice 4 is needed before claiming fast
whole-home refresh independent of inventory size.
The slices build on the opened-root lifecycle that the
[opened-root inventory engine plan](plan-2026-08-25-fdu-opened-root-inventory-engine.md)
delivered to `main`. Slice 2 also builds on three adjacent contracts, all now on `main`:

- the index journal is bounded in bytes (`OpenOptions::journal_capacity_bytes`,
  [#56](https://github.com/jlevy/fdu/pull/56));
- an index built without control state answers with `Error::ControlStateNotObserved`
  rather than as if nothing were ignored ([#57](https://github.com/jlevy/fdu/pull/57)),
  which is what a checkpoint captured without control observation records;
- a crossed control limit leaves the index, including one loaded from a snapshot, a
  typed record of both limits and every refused control source
  (`Index::control_coverage`, `fdu-1onj`, [#63](https://github.com/jlevy/fdu/pull/63)),
  which is what a checkpoint with partially observed classification records.

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
shown with the partial marker, not as an unqualified move between partitions; a
checkpoint whose recorded refused sources are fewer than its refused count marks every
classification delta partial; and checkpoints captured at different budgets report
classification as not comparable.
Accounting cases have stated expected deltas:

- **Hard link added to an existing in-scope file:** per-path allocated grows by the
  file’s allocated size at the new link’s directory; unique allocated is unchanged in
  total, and its attribution moves only if the new link sorts first.
- **Hard link added in scope to a file outside the scope:** for example, uv configured
  with `--link-mode hardlink`, installing from a cache outside the root into an
  environment inside it.
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

Disk-pressure acceptance adds Home/Applications/temporary-root fixtures, `/tmp` alias
deduplication, nested-root projection, cross-root hard links and moves, independent
volumes, one unavailable root, and an external checkpoint store that disconnects.
An hourly request without an hourly baseline must refuse that interval rather than
guessing; a skipped scheduled capture reports the actual longer interval.
Test a capture that straddles the requested boundary and a mixed-root manifest with only
some eligible roots.
Run root retention and block collection while publishing and retaining manifests; every
retained manifest must still resolve all its checkpoint dependencies.
Inject ENOSPC during checkpoint publication and verify the previous checkpoint and
applied cursor remain usable.
Include growth outside every selected root, Trash moves, clone writes, and transient
create/delete churn so a volume-space drop cannot be presented as fully explained by an
incompatible directory-byte sum.
Run ordinary roll-up and cached source-line requests alongside profile operations to
prove this additional workflow does not change their answers or create history stores
implicitly.

The performance subject must also reproduce agent-development growth: a few large
artifacts, many small compiler outputs, a new dependency environment, and a removed or
moved build tree, on top of an unchanged million-entry inventory.
Include a wide dirty directory and overlapping historical creation events, since either
can defeat an apparently small change set.
Use actual allocated growth for the disk-pressure case, not only sparse lengths or APFS
clones; record both when either mechanism is the subject.
Predeclare seconds-scale first-current-attribution and completion thresholds and the
allowed state-write budget before those trials, then report distributions and fallbacks
against a paired fresh scan.
Until those end-to-end targets pass, present the probe as mechanism evidence only.

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
