# Plan: Recent Growth, Age Profiles, and Volume Capacity

**Date:** 2026-11-08

**Author:** fdu project, with Claude Opus 5.5

**Status:** Approved for implementation (2026-11-08). Phase 0 (documentation of what
works today) shipped with this plan in #208. Phases 1–4 are implemented in #209, with
the [open questions](#open-questions) decided as recommended and the implementation
amended by its reviews where each section below says so.
Phase 4 (2026-11-09) replaces phase 1’s second time basis with one time that every
report reasons in, before any of it is released; [the design](#one-time) below is phase
4’s, and phase 1’s two-window design is recorded under
[alternatives](#alternatives-considered).

## Overview

A common disk question has no history attached to it: free space dropped by 10 GB since
yesterday, so where did it go?
fdu keeps no record of past sizes unless someone captured a baseline earlier, which is
what the [checkpoint plan](plan-2026-09-13-fdu-disk-usage-checkpoints.md) is for.
But every file already records when it was last written.
One metadata walk can therefore say where everything written since a given instant
lives, by folder, without any earlier state.

fdu can already answer that question today, but an agent has to assemble the answer from
several flags and knows none of their pitfalls.
Two of those pitfalls produce wrong answers: a directory-admitting filter that reports
whole active trees as growth, and modification times that miss copied and extracted
data. This plan:

1. Documents the recipe that works now, with its limits, in the skill, `--docs`, the
   README, and the usage guide (phase 0).
2. Makes **change time**, which copies and extractions made through ordinary file APIs
   do not hide from, the time every report reasons in: windows, sort, the tree’s age
   column, `recent`, and `ages`, with modification time one flag away (phases 1 and 4).
3. Adds an **`ages` view**: the selected bytes partitioned by when they were last
   written, every window from one walk (phase 2).
4. Adds a **`volumes` view**: the capacity of each filesystem the report touches, beside
   the bytes the report counted there, as a convenience rather than a `df` replacement
   (phase 3).

The design adds three concepts and three flags (`--since`, `--before`, and `--time`).
Everything else is composition with the axes fdu already has.

## The Questions

| Question | Today | After this plan |
| --- | --- | --- |
| Where are the files written in the last day, by folder? | `fdu PATH --kind file --modified-since 24h` (misses copies that kept older times) | `fdu PATH --kind file --since 24h` |
| How much was written in the last day? | Add `--view summary` | The same, or one row of `--view ages` |
| Which files are they? | `fdu PATH --view largest --modified-since 24h` | `--view largest --since 24h` |
| How is this tree’s space spread over time? | One run per window, or one Python index queried per window | `fdu PATH --view ages` |
| Which folders account for one window, and what does that window hold? | Two runs | `fdu PATH --kind file --since 7d --view tree,ages` |
| How full is the volume, and how much of it does this tree account for? | `df` beside fdu | `fdu PATH --view summary,volumes` |
| How full is the volume, quickly? | `df PATH` | `fdu PATH --view volumes --scan-depth 0` |
| What changed between two moments, deletions included? | Two saved JSON reports, diffed by hand | Unchanged: checkpoints (`fdu-8ybz`) |

## Goals

- An agent asked where disk space went over a window answers it with one command, one
  walk, and no prior state, and states exactly what the answer covers.
- The answer is not fooled by copies and extractions that keep an older modification
  time, on the filesystems and tools
  [the soundness limits](#where-change-time-is-weaker) do not name.
- One walk answers every window at once, so no question is answered by repeated scans.
- Every new value is a projection of facts the walkers already read.
  Phases 1 and 2 add no system call on Linux or macOS.
- Each new number equals a composition of existing queries, and tests prove it.
- The Rust engine, the command line, and Python give the same answer, per the
  [surface architecture](../../architecture/fdu-surface-architecture.md).

## Non-Goals

- **Net change and deletions.** A walk sees what exists.
  Bytes deleted or files shrunk since the window began are invisible to it, and the
  [checkpoint plan](plan-2026-09-13-fdu-disk-usage-checkpoints.md) owns exact net
  comparison. This plan never presents written bytes as net growth.
- **Free-space history.** Comparing today’s free space with yesterday’s needs a sample
  taken yesterday, which is retained state; the disk-pressure profile (`fdu-vw9r`) owns
  it. The volumes view observes capacity now and stores nothing.
- **A `df` or `duf` clone.** No mount enumeration, filesystem-type filters, inode
  counts, or volumes outside the report’s roots.
  `fdu-eas0` records why.
- **A folder-by-band matrix.** A tree with a column per age band needs per-directory
  band counters. Composition covers the questions in [the table above](#the-questions);
  revisit with evidence that it does not.
- **Birth time.** Splitting changed files into created-in-window and existed-before
  would tighten the upper bound, but birth time is not observed on every platform or
  filesystem. See [Open Questions](#open-questions).

## Background

### What one walk can know without history

Every entry carries a modification time (`mtime`) and an inode change time (`ctime`). On
Unix the kernel sets `ctime` to the current time whenever the inode changes: a write, a
truncation, a new link, or a change to its permissions, ownership, timestamps, or
extended attributes.
POSIX leaves it open whether a rename updates the renamed file’s `ctime`; Linux
filesystems do, so a file moved into the scope may or may not read as changed.
A move within one filesystem allocates nothing, so this cannot hide a loss of free
space. A move from another filesystem is a copy: it gets a new inode and change time but
keeps the old modification time, which is exactly the case change time catches.
No Unix file API sets `ctime` to a chosen value; [below](#where-change-time-is-weaker)
are the cases that bypass the file API or the clock.

Define an entry’s **change time** as the newer of the two.
The consequence is the whole design:

> An entry whose change time is before T has not been written since T, so it contributed
> nothing to growth since T.

Every byte written since T into a file that still exists is in a file whose change time
is at or after T. One walk therefore gives:

- **Where:** the complete set of present files written in the window, by folder.
- **An upper bound on how much:** a file written in the window counts at its full size,
  so appending 1 MB to a 10 GB database counts 10 GB.
- **No shrinkage:** deleted files and the bytes a shrinking file gave back are not in
  the walk. Written bytes are gross arrivals, not net change.
- **Over-inclusion, not omission:** a metadata-only change, such as `chmod -R`, an
  extended attribute that a desktop indexer sets, or a new hard link, makes a file count
  as changed without growing it.
  Omission is possible only in the cases [below](#where-change-time-is-weaker).

Both errors in the last two points are conservative for both common questions.
For “what grew?”, over-inclusion can only add candidates, never hide one.
For “what is stale?”, a file counts as untouched only if nothing at all has happened to
it, so a file someone just copied in with an old modification time is never reported as
stale.

Modification time alone does not have this property.
Userspace sets it freely, and many tools set it into the past on purpose: `cp -p`,
`rsync -a`, Finder and Explorer copies, `mv` from another filesystem, `tar -x`, `unzip`,
downloads that take the server’s time (wget does by default), and package managers that
stamp files with a fixed time so builds are reproducible.

### Where change time is weaker

The guarantee holds for data written through the file API on a filesystem that stores a
real status-change time.
It weakens in these cases, which the documentation must name and the plan does not try
to detect:

- **The clock moved.** A system clock set back makes new writes look old, and on a
  network filesystem the server’s clock, not the client’s, stamps change times.
- **Below the file API.** Replicating or restoring a whole filesystem, such as
  `zfs receive` or restoring a disk image, keeps the source’s change times, so restored
  data reads as old even though its space was allocated now.
- **Settable change time.** Windows lets a caller set `ChangeTime` with
  `SetFileInformationByHandle`; ordinary copies do not, but a tool can.
- **No stored change time.** Filesystems that do not store a status-change time, such as
  the FAT family, report a substitute, so change time there is no stronger than the
  timestamp it is derived from.
  Behavior per driver is unverified and belongs in phase 1’s platform tests.

### What modification time misses, measured

All figures below come from one uncontrolled, virtualized Linux container (about 182,600
regular files, 9.7 GiB), on 2026-11-08, counted with `find -mmin` and `-cmin` and a
one-pass `lstat` script except where an fdu command is named.
Their file totals differ by one (182,639 from the script, 182,638 from fdu); the cause
was not investigated.

- **Cargo:** a build in this session extracted 2,460 crate source files (31.2 MiB) into
  `~/.cargo/registry/src`. Cargo stamps them with a fixed 2006 modification time, so a
  two-hour modification-time window found only Cargo’s empty `.cargo-ok` markers (39
  files, 0.0 MiB). A two-hour change-time window found all 2,460 sources.
- **The whole filesystem:** allocated bytes written within the last hour were 1.4 GiB in
  9,801 files by change time, and 1.2 GiB in 7,370 files by modification time.
  The 2,431-file difference arrived with older modification times.
- **A synthetic tree** with 8 MB of new build output, a 5 MB log append, a 50 MB `cp -p`
  copy, and a 12 MB `tar -x` extract (about 75 MB written):
  `fdu --kind file --modified-since 24h` reported 13 MiB.

Not every package manager does this: the npm packages installed on the same container
carry their install time.
Which tools keep archive times changes between releases, so the design must not depend
on knowing them.

### What exists today, and its traps

The selection axis already has `--modified-since` and `--modified-before`, and the tree
already shows each row’s age.
Two behaviors make the obvious command wrong for this question:

1. **Without `--kind file`, a time window admits whole directories.** Size and time
   bounds test a directory’s eligible subtree, and a matching directory covers its
   contents in aggregate views.
   On the container above, `fdu / --modified-since 1h` admitted all of `~/.rustup` (1.2
   GiB) because a toolchain had been installed in it minutes earlier.
   The new toolchain was 623 MiB; the other 623 MiB was an older one, untouched.
   At home-folder scale nearly every top-level directory has some activity in a day, so
   the report is close to the whole tree.
   `--kind file` asks for the files, and their folders roll up only what was admitted.
2. **A filtered report does not say what it filtered from.** The root row of a filtered
   tree reads `100%` of the admitted total, and a filtered summary prints only the
   admitted size. Nothing on the screen tells a reader whether 704 MiB is a day’s growth
   or the tree itself.

### One walk is the floor; repeated walks are not needed

Nothing short of reading every entry’s attributes answers this question correctly
without a baseline:

- An in-place write changes no directory’s times, not even its parent’s
  ([performance frontier](../../research/research-2026-08-10-performance-frontier.md#change-propagation-what-filesystems-refuse-to-tell-you)),
  so no subtree can be skipped by its directory times.
- FSEvents does not report a write until the last close of the file, so it misses logs
  and databases that stay open, and replay cost scales with the volume’s journal, not
  the root
  ([change sources](../../research/research-2026-09-27-disk-growth-change-sources.md)).
- Loading a flat snapshot costs about as much as the walk it would replace.

The walk is cheap where most of this growth happens.
On the container above, warm, one run each:

| Command over `/ --one-filesystem` | Time |
| --- | --- |
| `fdu` (the default tree) | 143 ms |
| `--view summary` | 138 ms |
| `--kind file --modified-since 1h` | 272 ms |
| `--modified-since 1h --depth 2`, which admits directories | 670 ms |
| `--view largest --modified-since 1h --limit 5` | 708 ms |

The change-source review measured walks of 0.1 s at 12,000 entries, 5.7 s at 452,000,
and 18.6 s at 1.5 million.
A whole home folder of several million entries takes minutes cold, so scope still
matters: the checkpoint plan’s
[scope table](plan-2026-09-13-fdu-disk-usage-checkpoints.md#scope-defaults-belong-in-a-brief-skill)
lists the usual roots.

What this plan removes is the second and later walks.
Today the command line needs one run per window; the `ages` view answers every window
from one, and several views share one walk as they already do.
Python’s `fdu.scan` and `fdu.open` already answer any number of windows from one
retained index.

## Design

### Concepts added

| Concept | Model | Command line | Python |
| --- | --- | --- | --- |
| One time | `TimeBasis` (`Changed` by default, or `Modified`) on `Selection`, with one `Selection::window`; an entry’s `Times` holds both clocks (`Attrs::times`), and the index maintains both | `--since`, `--before`, `--time changed\|modified`, `--sort time` | `Selection(since=, before=, time=TimeBasis.MODIFIED)`, `SortKey.TIME` |
| Age profile | `ViewSpec::Ages`: a partition of selected regular files by age band on the report’s clock | `--view ages` | `View.AGES` |
| Volume capacity | `ViewSpec::Volumes`: a join of the bytes counted per filesystem with capacity observed at report time | `--view volumes` | `View.VOLUMES` |

One further change is not a concept: every filtered report says how much of the scanned
tree it admitted ([Selection coverage](#selection-coverage)).

#### Alternatives considered

- **A second window beside the first (phase 1’s design).** `--changed-since` and
  `--changed-before` beside `--modified-*`, `--sort changed` beside `--sort mtime`, and
  `ages` by change time while the tree’s age column and `recent` stayed modification
  time. It was chosen to keep released flags meaning what their names say, and replaced
  before release (phase 4), because the cost was a report in two clocks:
  `--view tree,ages` showed a copied Cargo crate as `20y` in the tree and `under 1h` in
  `ages`; `--sort changed` had to be refused wherever rows render as a tree; four window
  flags and two sort keys named near-synonyms, and fd’s `--changed-within` already means
  modification time. What it offered alone, composing the two windows to list arrivals
  carrying old content, remains in every list row, which carries both clocks.
- **Modification time as the one time.** The released meaning, but it misses exactly the
  arrivals measured above, and its errors point the unsafe way for both questions this
  plan answers ([one time](#one-time)).
- **One time with no opt-out.** A bulk `chmod -R`, `chown -R`, or SELinux relabel, or a
  sync client writing extended attributes, moves every status-change time at once, and
  then only modification time still tells content age; `--time modified` is that escape,
  and the same axis takes birth or access time later as a value, not a flag family.
- **ctime alone.** On Unix, `max(mtime, ctime)` equals `ctime` except for an entry whose
  modification time was set into the future.
  The maximum keeps that entry visible rather than letting a future modification time
  hide behind an older `ctime`.
- **Modification time for `ages`.** That would match the tree’s age column, but the
  view’s question is when space arrived, and modification time misses exactly the
  arrivals measured above.
  See [Open Questions](#open-questions).

### Change time

`Attrs` already carries `ctime_ns`, the walkers already read it, and snapshots already
retain it in the validity fingerprint:

| Platform | Source | Extra cost |
| --- | --- | --- |
| Linux | `statx` with `STATX_BASIC_STATS`, which includes `stx_ctime` | None |
| macOS | `getattrlistbulk` with `ATTR_CMN_CHGTIME`, in `REQUIRED_COMMON` | None today; if `fdu-f6n7` narrows the attribute set, the plan must keep it whenever change time is requested |
| Windows | `FILE_BASIC_INFO.ChangeTime` on the handle each entry is opened with | None; an entry that cannot be opened (`observe_listed`) has no change time |

An entry whose status-change time was not observed (on Windows, one fdu could list but
not open, such as `pagefile.sys`) is timed by its modification time on the change-time
clock, which is a lower bound of its change time, never a zero.
A note counts the files so timed whenever a window or `ages` counted one, so the
substitution is never silent.
Treating them as unknown instead, as phase 1 did, would blank the age of every ancestor
of such a file, the root of `C:\` included, while a directory’s time is the newest of
everything beneath it, which one lower bound almost never moves.

A directory’s time on either clock is the newest of that clock in its eligible subtree,
its own included unless it is the report root, as the tree’s age column has always been
defined.

### One time

Every report reasons in one time: an entry’s **change time** by default, or its
**modification time** under `--time modified`. Windows, `--sort time`, the tree’s age
column, `recent`, `ages`, and every row’s `age_ns` read the same clock, so one report
never mixes two.

|  | Change time (default) | Modification time (`--time modified`) |
| --- | --- | --- |
| Is | `max(mtime, ctime)`: when the entry last changed in any way, its content or its metadata, arriving by copy or extraction included | `mtime`: when its content was last written, as `ls -l` shows |
| “What is new since yesterday?” | Can over-include: a `chmod` looks new | Omits copies, extractions, and restores that keep old times |
| “What is old enough to delete?” | Errs safe: a just-restored backup is not called stale | Calls a just-arrived copy of old content stale |
| Fails when | A bulk `chmod`/`chown`/relabel moved every status change | Content arrived carrying old times |

Change time is the default because its errors point the safe way for both questions: it
over-includes rather than omits when asked what is new, and it holds back rather than
condemns when asked what is old.

`--since T` and `--before T` select by the report’s time, with the value grammar and
half-open interval the `--modified-*` flags had.
`--sort time` orders rows newest first wherever a row carries a time, trees included,
since each carries the same clock its column shows: tree, list, files, largest, recent,
and code rows. Type, extension, family, language, and document rows carry no time.
Beside a view the sort orders, as in `--view full`, they keep size order and a note says
so; a request in which the sort would order nothing is refused rather than falling back
to name order. `--time` takes `changed` or `modified`.

The index maintains a directory’s newest time on both clocks, for its regular files and
for every entry beneath it, beside its other roll-ups, so either value of `--time` reads
the same fast paths; a directory grows by 24 bytes.
Both clocks come from the same `stat` result, so maintaining the second adds no system
call to any walk or mutation.

Machine output names the request’s `time`, and every row and node gives its time as
`time_ns`, `time_at`, and `age_ns`; a summary or tree row’s newest regular file is
`newest_file_time_ns`. List rows also carry both raw clocks, `mtime_ns` and
`changed_ns`, so a caller can still find arrivals that carry old content (`changed_ns`
recent, `mtime_ns` old) in one listing.

The name `changed` follows POSIX, where `stat` labels `ctime` “Change”; the value names
the clock, not the query, and `--since` reads the same under either.

### The `ages` view

`--view ages` partitions the selected regular files by their age in the report’s time,
newest first. Edges are hours and days, and a year is 365.25 days, as in the tree’s age
column. The time grammar takes no calendar units, so each edge is spelled the way a
window flag accepts it, and every row is one window query over the same selection and
instant:

| Band | Age | Window that selects exactly this row |
| --- | --- | --- |
| `future` | Zero or below: clock skew or a timestamp set ahead | `--since now` |
| `under 1h` | Over 0, to 1 hour | `--since 1h --before now` |
| `1h to 1d` | Over 1 hour, to 1 day | `--since 24h --before 1h` |
| `1d to 7d` | Over 1 day, to 7 days | `--since 7d --before 24h` |
| `7d to 30d` | Over 7 days, to 30 days | `--since 30d --before 7d` |
| `30d to 90d` | Over 30 days, to 90 days | `--since 90d --before 30d` |
| `90d to 1y` | Over 90 days, to 365.25 days | `--since 365d6h --before 90d` |
| `1y to 3y` | Over 1 year, to 3 years | `--since 1095d18h --before 365d6h` |
| `over 3y` | Over 3 years | `--before 1095d18h` |

Windows are half-open, `[since, before)`, so an entry exactly on an edge falls in the
newer band: `--since` is inclusive.
That includes the zero edge, so an entry stamped at the request instant itself counts as
`future`.

Empty bands are omitted in text and kept in machine output.
Each text row is the standard grouped-row layout (size, share, band, file count), so
nothing in the output design changes.
Cumulative tallies (this band and every newer one, which answer “how much changed within
30 days?”) are in machine output; whether text shows them too is
[open question 7](#open-questions).
This is the shape the measured container would print (illustrative, not a golden; counts
from the `lstat` script):

```text
   4.0 KiB    0.0%  future           1 file
   1.4 GiB   14.8%  under 1h         9,801 files
   3.9 GiB   39.8%  7d to 30d        61,113 files
   985 MiB    9.9%  30d to 90d       12,810 files
   552 MiB    5.6%  90d to 1y        7,147 files
   2.1 GiB   21.4%  1y to 3y         54,225 files
   848 MiB    8.5%  over 3y          37,542 files
```

The rules it obeys:

- **It partitions.** The rows sum to the files and bytes `--view summary` reports for
  the same selection
  ([a roll-up partitions](../../architecture/fdu-design-principles.md#a-roll-up-partitions-what-it-reports-on)).
- **Each row is a query.** The window in the table above, with `--kind file` and
  `--view summary`, over the same selection and instant, gives the row’s tallies, and a
  test pins that identity for every band.
- **It composes.** Selection narrows what is profiled (`--include '*.log'`,
  `--ignored=only`, `--kind dir --include node_modules`, which covers each directory’s
  contents), several roots are profiled as their union, and `--view tree,ages` shares
  one walk.
- **It is metadata-only.** Like `largest` and `recent`, it implies no analyzer.
- **Bands are relative to the request instant.** A `--watch` repaint resolves them from
  its own instant, as its ages do, so a file can move to an older band at a repaint that
  another change caused; the repaint rule for the tree’s ages
  ([multiple paths and tree age](plan-2026-10-09-fdu-multiple-paths-and-tree-age.md))
  extends to band edges.

Machine output gives each band its exact edges as ages and as instants, its `files`,
`bytes`, `allocated`, and `ignored` tallies, and a `cumulative` object with the same
tallies; the request’s `time` names the clock.

### The `volumes` view

`--view volumes` answers “how full is the filesystem this report is about, and how much
of it did the report count?”
It is the thin capacity join `fdu-eas0` describes, promoted to a plan:

- **Rows.** One per filesystem that holds a scanned root or that the walk entered below
  one, identified by device, which already merges bind mounts of one filesystem.
  Without `--one-filesystem` a walk can enter mounted filesystems below the root, so a
  report can have several rows.
- **Capacity, observed by the serving layer, read by the reader.** The engine gains one
  observation step, `observe_capacity(volumes) -> CapacityObservation`, given each
  filesystem’s device and a path on it, with one call per filesystem and an
  `observed_at`: `statfs` on Linux and macOS, which returns capacity and the filesystem
  type together, and `GetDiskFreeSpaceExW` with `GetVolumeInformationW` on Windows.
  The pure readers take the observation as an argument, so they never touch the
  filesystem
  ([views are readers](../../architecture/fdu-design-principles.md#views-are-readers)):
  `report` and `report_roots` take it as their last argument, `None` where nothing was
  observed. Every serving route observes before it reads when the request names
  `volumes`, through `query::observe_volumes`: `prepare_report` for one-shot reports,
  `OpenedIndex::read` (before it takes its index guard), each watch repaint, and
  Python’s `Index.report`, inside the native call with the GIL released.
  A capacity call is made only on a root or on a directory the walk listed, since on an
  unmounted automount trigger it mounts as a listing would (review F1 on #209). The
  observation carries each root’s device beside the capacity, since an index does not
  record the root’s own attributes.
  `--stale-ok` observes now, so its capacity is current while its tree is stale, and the
  section’s `observed_at` says when capacity was read.
  A caller of the pure reader that passes no observation gets every capacity field as
  unknown (null), with a note naming `observe_volumes`; it is never zero.
  A request without `volumes` observes nothing.
  Capacity is never stored in a snapshot or served from the cache.
- **Columns.** Capacity, the share in use, the path where the report entered the
  filesystem (a root, or the directory where the walk crossed onto it), available bytes,
  and the bytes this report counted there.
  No mount table is read: the walk already knows where the device changes.
  When the walk reaches one filesystem at several paths, the row is labeled by the
  bytewise-first of them, and a note says how many others there are.
  The type comes from the same call: by name on macOS and Windows, and on Linux as
  `f_type`, a number, which a short table maps to names for the filesystems the notes
  need (btrfs, ZFS); any other number is shown as itself.
  The share in use is `(capacity − available) / capacity`: the figure that matters when
  space runs out, and what `duf` shows.
  It is not the volume’s own `used` blocks, which `df` shows and which differ on APFS.
- **No sums across rows, and no implied identity.** Available space can be shared
  between volumes of one pool (an APFS container, btrfs subvolumes, ZFS datasets), and a
  note says so on those filesystems.
  The bytes counted and the bytes in use never add to the capacity: snapshots, other
  volumes, purgeable space, and deleted files still open all use space no walk sees.
- **Fast form.** `--scan-depth 0` walks nothing below the root (160 µs on the container
  above), so `fdu PATH --view volumes --scan-depth 0` is a capacity check that composes
  existing scope rather than adding a mode.
  Its counted bytes are zero because the report counted nothing.

Illustrative layout, with invented values:

```text
   460 GiB   97% in use  /  (apfs)
                           14 GiB available
                           182 GiB counted by this report
note: available space on an apfs volume is shared with the other volumes in its container
```

`ages` joins `--view full`, which shows every view but List and Files.
`volumes` does not: it describes the host, not the tree, so two full reports of an
unchanged tree would differ, and a digest would make capacity calls a tree of the same
request does not. A first version included it; the maintainer left it out on review F13
of #209.

### Selection coverage

Every report whose selection predicates (kind, patterns, size or time bounds, ignored
state) narrow the scanned set ends with one note saying what they admitted.
Display bounds already have their own note and do not trigger this one.
The note is omitted when the selection admitted everything scanned, and `--quiet`
suppresses it with the other notes; the machine fields are always present.

```text
note: selected 2,951 of 27,060 files, 704 MiB of 1.8 GiB
```

The scanned totals are the unfiltered root roll-up, already maintained, so the note
costs nothing. Machine output carries the same two tallies.
This follows the output rule that a note is added wherever the displayed facts can be
misread
([output design](../../architecture/fdu-output-design.md#streams-and-categories)), and
it consolidates with the existing notes.

### Relationship to checkpoints

The two answers have different contracts and must never be presented as each other:

|  | Written-since (this plan) | Checkpoint comparison |
| --- | --- | --- |
| Prior state | None | A checkpoint captured at the window’s start |
| Answers | Where bytes written in the window are now | Net growth and shrinkage between two captures |
| Deletions | Invisible | Shown |
| A file appended in the window | Counted at full size (upper bound) | Counted by its delta |
| Cost | One walk | A walk or replay per capture, and retained state |

The checkpoint plan forbids answering a checkpoint window with a modification-time
filter. This plan keeps that rule: the skill and every note call this answer “written
since”, never “grew by”.
The written-since answer is the zero-setup first response to a disk-pressure question,
and the right answer whenever no baseline exists.
When both exist, a checkpoint comparison can use the written-since tree to confirm that
its growth lies where new data was written.

## Implementation Plan

The epic is `fdu-tjsi`, and the open questions are `fdu-dvub`. Each phase is one pull
request; phases 2 and 3 are independent of each other.
The report schema moves once per release, so phases that ship together share one bump
from `fdu.report/11`.

### Phase 0: Document what works today (`fdu-mncb`)

- [x] This plan.
- [x] The skill, `fdu --help`, `fdu --docs`, both READMEs, and the usage guide gain a
  “what was written recently” recipe with `--kind file --modified-since`, the reason
  `--kind file` is required, and the modification-time blind spot.
- [x] Pointers from the checkpoint plan, the change-source review, and `fdu-eas0`.

### Phase 1: Change time (`fdu-bo9q`, `fdu-xk3c`)

- [x] `Selection` gains a change-time window beside the modification window, one
  `TimeWindow` type for both (`ModifiedWindow` remains as an alias); `SortKey::Changed`.
  No separate `TimeBasis` type: the selection already names its basis per window.
- [x] `SubtreeValues` gains the newest change time and whether every counted entry had
  one; directory matching follows the existing modification-time rule.
- [x] Unknown change time is typed through selection and rows (`Option<i64>`); the note
  is below.
- [x] `--changed-since`, `--changed-before`, `--sort changed`; Python `changed_since`,
  `changed_before`, and `SortKey.CHANGED`.
- [x] `changed_ns` and `changed_at` on list rows; tree nodes carry none (see phase 1’s
  change-time windows, superseded by [one time](#one-time)).
- [x] The selection-coverage note and its machine fields (`selected` and `scanned` on
  every envelope, each `{files, dirs, bytes, allocated}`).
- [x] The skill and docs switch the recipe to `--changed-since`.
- [x] Once “changed” names change time, reword the places that use it for modification
  activity, such as the tree’s age (“how long ago anything it counts last changed”) in
  the skill, `--docs`, the usage guide, and the README.

Phase 4 replaces this phase’s second window, its sort key, and its unknown change time
before release; the items above record what was built and then superseded.

### Phase 2: The `ages` view (`fdu-5079`, `fdu-alw7`)

- [x] `ViewSpec::Ages` over the index: band assignment from change time and the request
  instant, partition and cumulative tallies, the ignored split, several roots
  (`query_ages.rs`). It profiles the files a summary of the same selection counts, so a
  directory match profiles what it covers; the window identity holds for selections that
  compose with `--kind file`, and the partition for every selection.
- [x] Text renderer on the grouped-row layout; JSON, JSONL, and YAML records.
  A watch’s repaint identity measures band instants from its fixed reference, so an
  answer whose bands moved differs and one whose clock alone moved does not.
  A watch asks for an answer when its tree changes, so it re-bands at each repaint, not
  at each edge: in an idle tree a file keeps its band until something else changes
  (review F7 on #209).
- [x] Inclusion in `--view full`.
- [ ] A performance-loop experiment for a derived route: an unfiltered `--view ages`
  (alone or with `summary`) can count bands as the walk reports entries, with no index,
  as unfiltered `summary` already does.
  Predeclare its accept rule before trials; until accepted, `ages` uses the index.

### Phase 3: The `volumes` view (`fdu-eas0`)

- [x] `observe_capacity` per distinct device in the report’s scope, called by every
  serving route listed under [the `volumes` view](#the-volumes-view), through
  `query::observe_volumes`; unknown capacity with a note when none is passed.
  Amended in implementation:
  - `report` and `report_roots` take the observation as their last argument, `None`
    where nothing was observed, so no caller silently gains filesystem reads.
    A first version kept their signatures beside two `_with_capacity` readers; review F9
    on #209 collapsed each pair, since the project owes no caller of the older shape.
  - Capacity is asked only of the roots and of directories the walk listed, and an
    opened read observes before it takes its guard (review F1 and F2 on #209).
  - The observation carries each root’s device beside capacity (`VolumesObservation`),
    because an index records every entry’s attributes but the root’s own; a reader given
    none infers it from the root’s directories.
  - When capacity was observed is the section’s `observed_at`, not a provenance tier: it
    labels the capacity beside a stale tree just as well and leaves every other envelope
    unchanged.
- [x] Each row labeled by the bytewise-first path where the report entered the
  filesystem, from the walk’s own device changes, with a note when there are others; the
  type from the same call, with Linux `f_type` mapped for btrfs and ZFS. No mount-table
  parsing. Only a directory starts a filesystem: overlayfs, which containers run on,
  reports each file’s underlying layer, which split one filesystem into a row per layer
  until files were counted on their directory’s device.
  A one-filesystem walk touches only its root’s filesystem, since it never enters the
  mount points it retains.
- [x] Rows, pooled-filesystem wording, text and machine renderers.
  Each row is three lines on every platform, the pooled-space caveat on its available
  line rather than a separate note, so goldens pin one layout with named patterns.
- [x] `make cross-lint`, since every source is behind a platform gate.

### Phase 4: One time basis (`fdu-14k4`)

Decided 2026-11-09 by the maintainer, after phases 1–3 were built and before any of it
was released: one time that every report reasons in, replacing phase 1’s second window.
The release is a minor version, so names change where the design is better for it, not
where compatibility alone would keep them.
The design is [one time](#one-time).

**Engine: both clocks, one maintained shape (`fdu-hw1m`).**

- [x] `TimeBasis` (`Changed`, the default, and `Modified`) and `Times` (`modified`,
  `changed`, nanoseconds) in `engine_contract`, re-exported from `query`.
  `Attrs::times()` times an unobserved status change by modification, and
  `Attrs::change_time_ns` keeps reporting whether one was observed.
- [x] `InternedRollUp`’s newest regular file and `InternedPartitionRollUp`’s newest
  activity become `Times`; `merge`, `recompute_newest_upward`, `raise_activity_upward`,
  the folded scan, the summary fold in `execution.rs`, and every snapshot load maintain
  both. Roll-ups are rebuilt from entry attributes on load, so the snapshot format is
  unchanged.
- [x] Public `RollUp` and `RollUpSummary` gain `newest_changed_ns` beside
  `newest_mtime_ns`; the index-level API keeps raw clocks, since it answers no request.
- [x] The size tests record the larger directory roll-up.

**Query: one window, one basis (`fdu-h3t9`).**

- [x] `Selection::window` and `Selection::time` replace `modified` and `changed`;
  `ModifiedWindow` goes.
  `Candidate`, `SubtreeValues`, the `ActivityTable`, and the maintained activity carry
  `Times`, and every predicate and row reads the request’s basis.
- [x] `SortKey::Time` replaces `Mtime` and `Changed` and is valid in every view, so the
  tree refusal goes; with no unknown time, `--reverse` reverses it as it does every
  other key.
- [x] `recent`, `ages`, the tree, grouped rows, list rows, and the summary read the
  basis. `ages` loses its `unknown` band and its basis note.
- [x] The note for files timed by modification replaces the unknown-change-time note:
  counted where a window or `ages` counted such a file, and only under `changed`.
- [x] `ReadSpec::since`, `before`, and `time`; `AxisNames` names them per surface;
  `parse_time_basis`; `parse_sort` takes `time` and no longer `mtime` or `changed`.

**Report schema (`fdu-wtoc`).**

- [x] Every row and node gives `time_ns`, `time_at`, and `age_ns`; summary and tree rows
  give `newest_file_time_ns`; list rows also give raw `mtime_ns` and `changed_ns`.
  `modified_at`, `changed_at`, and a node’s `mtime_ns` and `newest_mtime_ns` go.
- [x] The request echo gives `time`, and `Report` carries it for renderers.
- [x] The `ages` section drops `basis`; it has nine bands.
- [x] The schema stays `fdu.report/12`, which is unreleased.

**Surfaces (`fdu-0ace`).**

- [x] Command line: `--since`, `--before`, `--time changed|modified`, `--sort time`.
- [x] Python: `Selection(since=, before=, time=)`, `TimeBasis`, `SortKey.TIME`, model
  fields renamed with the schema, and `RollUp.newest_changed_ns`.

**Docs (`fdu-khwo`).**

- [x] The skill, `--help`, `--docs`, both READMEs, `docs/usage.md`,
  `docs/machine-output.md`, the architecture docs that name the age column or the old
  flags, and the CHANGELOG, which states every rename and the age column’s new default.

**Tests (`fdu-xff5`).**

- [x] Goldens move to `--since`, `--before`, and `--sort time`. A session over a
  backdated fixture passes `--time modified`, since `touch -d` cannot backdate a change
  time.
- [x] Engine tests: both clocks in maintained and walked roll-ups agree; the fallback
  timing and its note; `--sort time` over trees and grouped rows; `--time modified`
  reproduces the released modification-time answers.
- [x] Path independence compares a change-time answer by each row’s age residual and
  masks the clocks a fixture copy cannot carry, and adds `--time modified` requests that
  compare exactly.
- [x] Parity re-recorded; the `--time` and window flags are surface-label deviations.

Decided in implementation:

- Type, extension, family, language, and document rows carry no time, so under
  `--sort time` they keep size order beside a view the sort orders, with a note, and a
  request of only those views is refused rather than silently keeping name order; tree,
  list, and code rows sort by it.
  Review E found that refusing them outright refused `--view full --sort time`.
- Rust `FileRow::status_observed` is public, with `FileRow::changed_ns` beside it, so a
  library caller can tell an observed change time from the modification-time stand-in as
  machine output and Python can.
- Watch upserts carry `changed_ns` beside `mtime_ns`, so a stream under the default
  clock shows the time that admitted each entry.
- `ages` is refused for a request whose instant cannot be represented, where no age can
  be measured, rather than keeping an `unknown` band for that case alone.
- A list row’s raw `changed_ns` is null for an entry other than a directory whose status
  change was not observed, though its time on the change clock is its modification time,
  so the substitution is visible per row as well as in the note, and a symlink’s row
  agrees with its watch record (review F8 on #209).
- `FileRow` stays at 192 bytes: `times` and a public `status_observed` flag (review E4)
  replace `mtime_ns` and `changed_ns`.

## Testing Strategy

A test cannot give a file an old change time: anything it writes, `touch -d` included,
gets a change time of now.
Old change times come from moving the request’s instant instead.
`Request::build(spec, now, axes)` takes the instant that every relative window and age
resolves against (`request.now`); `generated_at` only stamps provenance.
A fixture written at T0, with its request built at T0 + 400 days, reads 400 days old by
both clocks, while a file given `touch -d` reads old by modification time and new by
change time. Python’s `Index.report` takes no instant, so these are Rust engine tests,
and Python tests cover windows relative to now.
Exact nanosecond edges and unknown change times use synthetic entry facts.

- **Soundness:** on a fixture with an in-place append, a `cp -p` copy, a `tar -x`
  extract, and a `touch -d` into the past, a change-time window admits every file
  written in the window, and a modification-time window demonstrably does not.
  A `chmod` is admitted though nothing grew.
  A CLI golden pins both against now; it asserts nothing about a rename, which POSIX
  lets a filesystem leave out of `ctime`.
- **Aging by instant:** engine tests build requests for the same fixture at several
  later instants and check every band and window, including entries that are old by both
  clocks, which neither the CLI nor Python can produce.
- **Identities:** the `ages` rows sum to `summary`; each row equals its window query in
  the band table; cumulative tallies equal `--since <edge> --view summary`; a window
  over an index equals a cold run (path independence).
- **Unobserved status change:** an entry with none is timed by modification on the
  change-time clock, and the note counts it when a window or `ages` counted it.
- **Clock edges:** future timestamps fall in `future`; an entry exactly on a band edge
  falls in the newer band, because `--since` is inclusive.
- **Volumes:** a root on one filesystem gives one row; a fixture with a mount below the
  root gives two (Linux CI, where mounting a tmpfs is possible), and `--one-filesystem`
  gives one; capacity never appears in a snapshot.
- **Parity and goldens:** every new flag, view, and field goes through the shared golden
  corpus and Python parity.
  Capacity and band contents vary by host, so goldens pin layout with named patterns,
  not values.
- **Selection coverage:** every existing filtered golden gains the note; review the
  diffs, not just the regeneration.

## Rollout

Phase 0 changes only discovery text: the skill, `--help`, and `--docs`. Phases 1–3 add
flags, views, and machine fields under one schema bump.
Phases 1–3 changed no existing value; filtered reports gained one note on stderr.
Phase 4 does change existing values: every time is change time unless `--time modified`
is given, so the tree’s age column, `recent`, and every age read change time.
The skill describes only what the installed executable supports, so it changes with each
phase, not before.

## Open Questions

**Decided 2026-11-08:** after #208 merged, the maintainer asked for the plan to be
implemented as written, so each question below takes its recommendation, which the
design above already follows.
Any of them can be reopened in the implementation pull request’s review.
**Reopened 2026-11-09:** phase 4 settles questions 1, 3, and 4 differently: one time,
change time by default, named `--since`, `--before`, `--time`, and `--sort time`, and
the tree’s age column follows it.

1. **Basis of `ages`.** Recommendation: change time, because the view’s question is when
   space arrived, and change time is conservative for both growth and staleness.
   Case against: the tree’s age column is modification activity, so `--view tree,ages`
   shows two bases in one report, and the Cargo sources above would read `20y` in the
   tree but `under 1h` in `ages`. Each is labeled, and the difference is the point.
   Alternatives: modification time (consistent, but blind to the arrivals measured
   above), or both as columns (no partition per row, and wider rows).
2. **Fixed or configurable bands.** Recommendation: fixed, as in the band table, because
   fixed edges keep every report comparable and any other edge is one window query away.
   Alternative: a view option such as `--age-bands 1h,6h,1d`, which needs a place on the
   view axis like `--words-per-page`.
3. **Names.** Recommendation: `--changed-since`, `--changed-before`, and
   `--sort changed`, per POSIX. Case against: an fd user expects `--changed-within` to
   mean modification time, and `--sort changed` beside `--sort mtime` names one key by
   meaning and the other by field.
   Alternative: `--written-since` and `--sort written`, which name the question but have
   no precedent.
4. **The tree’s age column.** Recommendation: keep modification activity, the meaning
   0.5.0 released. Alternative: move it to change time so one report has one basis, which
   changes released output.
5. **Birth time.** Recommendation: defer.
   Birth time would split changed files into those created in the window, all of whose
   bytes are new, and those that existed before it, whose growth is at most their size.
   It is `ATTR_CMN_CRTIME` on macOS, `STATX_BTIME` on Linux where the filesystem
   provides it, and creation time on Windows, so it would be unknown on some Linux
   filesystems. Revisit if the full-size count of appended files is shown to mislead.
6. **Selection coverage in machine output.** Recommendation: two tallies on the
   envelope, `selected` and `scanned`, because a parser should not need the note.
   Alternative: notes only.
7. **Cumulative tallies in text `ages`.** Recommendation: machine output only, so text
   keeps the grouped-row layout unchanged; “how much changed within 30 days?”
   is one `--since 30d --view summary` away.
   Case against: that is the question the view exists for, and a reader has to add rows
   to answer it. Alternative: a second share column of cumulative share, which needs the
   output design’s grouped-row rule amended first, since a continuation line may only
   break down its own row.

## References

- [Disk-usage checkpoints and daily deltas](plan-2026-09-13-fdu-disk-usage-checkpoints.md)
- [Finding disk growth fast: change sources compared](../../research/research-2026-09-27-disk-growth-change-sources.md)
- [Multiple paths and the tree’s age column](plan-2026-10-09-fdu-multiple-paths-and-tree-age.md)
- [Design principles](../../architecture/fdu-design-principles.md)
- [Output design](../../architecture/fdu-output-design.md)
- Beads: `fdu-eas0` (volumes), `fdu-a6dz` (reducer registry and its mtime-recency
  histogram), `fdu-8ybz` and `fdu-vw9r` (checkpoints and the disk-pressure profile),
  `fdu-f6n7` (macOS attribute set), `fdu-m3x5` (Windows allocated size)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
