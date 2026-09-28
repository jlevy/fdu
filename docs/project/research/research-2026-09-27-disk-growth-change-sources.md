# Finding Disk Growth Fast: Change Sources Compared

**Date:** 2026-09-27

**Status:** Review of the FSEvents replay spike and its alternatives.
Research only: no engine or command-line behavior changes.
The spike README and the September 27 prior-art research it cites are on PR #131’s
branch; links to them resolve once that PR merges.

## The Question

A user who is running coding agents wants to know, within seconds, where disk space went
over the last hour or day.
The scopes are a project, the home folder, and the temporary directories.
fdu already answers “how big is this tree now” quickly, and the
[checkpoint plan](../specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md)
describes how to compare two retained inventories.
The open question is how to produce the current inventory without re-walking millions of
unchanged entries.

On this host, the home folder holds about 6.7 million entries.
A full-index walk costs about 12 µs per entry with metadata caches cold, so a whole-home
walk takes minutes.
The [FSEvents spike](../../../explorations/fsevents-replay/README.md)
tested whether replaying persistent FSEvents history could nominate the few directories
that changed. This review asks whether that is the right mechanism, and what else could
do the job.

It has seven workstreams:

- the causal cost of replay;
- which writes FSEvents reports, and when;
- an audit of the spike’s harness and its live-root misses;
- filesystem-native change sources;
- the measured cost of a full walk and the architecture around it;
- a resident-monitor soak;
- a survey of operating-system facilities.

Instruments and sanitized results are in
[change-sources](../../../explorations/change-sources/README.md).

## Answer in Brief

- **FSEvents cannot see open writers, replayed or live.** The kernel emits a content
  event for descriptor writes only at the last close of the open file description, and
  for writable mappings only at the last unmap.
  `fsync` changes nothing.
  - On the live agent-state root, 13 and 14 files per trial changed with no event.
  - All were session logs or SQLite write-ahead logs, nearly all held open by
    long-running agent processes.
  - A resident watcher on the same stream misses them too.
- **One-shot replay cost is a property of the volume, not the root.** It scans the
  volume’s whole journal behind the cursor (about 0.12 s per compressed MB, CPU-bound in
  `fseventsd`), plus about 10 µs per matching record.
  - A day took 45 s on a churning external scratch volume, and 1.8 s on the internal
    Data volume for a quiet folder.
  - Receiving the expected event quickly says nothing about when the replay completes.
- **`stat` sees everything FSEvents misses here.** A walk is correct for open writers.
  - The walk is fast enough for project and agent-state scopes: 0.1 s at 12,000 entries,
    5.7 s at 452,000, and 18.6 s at 1.5 million.
  - At whole-home scale it is not.
- **The flat snapshot is not a shortcut.** Loading it costs about as much as the
  cheapest walk, so a refresh that starts from it cannot beat a walk.
  A resident recorder avoids the load; otherwise the checkpoint plan’s bounded
  persistent store (slice 4) is required.
- **Asking the filesystem gave one promising signal and one dead end.**
  - `searchfs` by change time works, but costs 110–330 s per query regardless of scope.
  - APFS fast directory sizing keeps a per-tree generation count current on every write,
    including writes through descriptors held open.
    It can be enabled without privileges through a shipped tool, but that tool uses a
    private interface and leaves a persistent flag on the directory.
    On one 225,653-entry fixture, a pruned refresh took 1.04 s against a 7.9–9.7 s walk
    and missed no size or membership change (single runs on a loaded host).
    But it cannot see mtime-only changes, marking a populated tree is a slow synchronous
    kernel operation, and its totals are not fdu’s accounting, so it is parked as a
    research result.
- **Open writers can be listed directly.** Same-user processes’ open-for-write files
  enumerate through libproc in about 15 ms without root.
  A monitor or replay plus this list covers the macOS gap, except for writers owned by
  other users.
- **fdu’s own watcher is exact today only by accident.** On the busy agent-state root,
  every macOS rename escalated to a full-root reconcile: 172 reconciles (174 root
  invalidations) in about an hour, costing 48% of a core and 6.9 GB of snapshot
  rewrites.
  - Without that polling, events alone would have missed 99.8% of in-place growth bytes,
    all of it in held-open files.
  - The writer list recovers all of it.
  - Windows without a root reconcile cost about 1% of a core.
    That is an estimate the rename fix (`fdu-822y`) must confirm.
- **Directory deltas do not explain free-space changes by themselves.** During the
  review, Time Machine local snapshots released about 6 GB with no tree change, and 931
  MiB was held by deleted-but-open files.
  A report needs a signed, labeled residual.

The [recommendation](#recommendation-and-the-case-against-it) is:

1. Fix the watcher first: scope one-sided renames (`fdu-822y`), decouple persistence
   from the render interval (`fdu-88p7`), and add the writer-list re-stat (`fdu-vhrb`).
   These are engine bugs under any plan.
2. Ship the hour and day report for project, agent-state, and temporary scopes on
   walk-captured checkpoints (the checkpoint plan’s slice 2), where walks cost 0.1–19 s.
   This is identical on Linux.
3. For home scale, prototype a resident dirty-directory recorder and measure it.
   Build the plan’s bounded persistent store (slice 4) only if the flat load then
   dominates.
4. Keep one-shot replay for gap recovery only, under a cost budget computed at measured
   walk rates, and park APFS directory statistics as a research result.

## How the Review Was Run

All measurements come from one host: macOS 26.5.2, arm64, 10 cores, 32 GiB.

- **Volumes:** an internal APFS Data volume (8.6 million inodes, 98–99% full) and an
  external APFS USB SSD (14 million inodes).
  The external volume holds agent worktrees and Cargo targets, so it churns heavily.
- **Load:** the host ran concurrent agents throughout, with load averages of 6–28 and
  spikes to 51 during the soak.
- **Timing isolation:** most timed and volume-wide runs held one shared lock, so
  parallel workstreams did not overlap them.
  The soak’s startup did not hold it, and other agents’ replays ran during the soak, so
  its startup and reconcile timings include contention.
- **Absolute times:** treat them as exploratory; compare within a workstream.
- **Mutation rules:** real trees were read metadata-only.
  Mutations touched only synthetic fixtures the review created.
- **Log access:** the external volume is mounted `noowners`, which made its `.fseventsd`
  log readable without root.
  That allowed replay time to be compared with the exact journal volume behind each
  cursor.

## FSEvents Replay Cost Scales With the Volume’s Journal

A device-relative stream replaying from an old cursor spends its time inside
`fseventsd`. Before it can send `HistoryDone`, it decompresses and scans the volume’s
entire log behind the cursor, regardless of the stream’s path filter.

The fitted model uses 26 completed runs with a quiet, empty-directory filter on the
external volume:

```text
history_seconds = -0.65 + 0.124 × compressed_MB_behind_cursor    (r² 0.98)
```

- **Throughput:** about 8 MB/s compressed, about 107 MB/s decompressed, and about
  850,000 records/s.
- **CPU:** `fseventsd` CPU rose about 0.079 s per MB (r² 0.95), which is 60–90% of wall
  time.
- **I/O:** repeats did not speed up, so the cost is not I/O-bound.

| Cursor age (external) | Log behind cursor | Time to `HistoryDone` |
| --- | ---: | ---: |
| 5 min (pilot) | 0.9 MB | 0.19 s |
| 1 h (pilot) | 17.9 MB | 1.97 s |
| 8 h | 150 MB | 16.8–17.1 s |
| 12 h | 233 MB | 27.7–31.0 s |
| 16 h | 234 MB | 24.5–25.8 s |
| 24 h | 334 MB | 45.5 s |

Age is the wrong variable.
The hours between 11 and 15 back had no churn, so 16 h cost less than 12 h. The same 24
h on the internal Data volume took 1.8 s (8 h: 0.7 s). Agent build output on the scratch
volume made the difference: about 14 MB/h on average and 34 MB/h at peak.
Across two days, that volume logged 42 million records.

**Matching records cost more.** A matching record costs about 10 µs inside `fseventsd`,
8 to 13 times a non-matching one (about 1.2 µs of wall time and 0.75 µs of CPU). The
same 2-hour cursor, with 2.8–3.8 million records behind it, cost:

| Filter | Wall time | Delivered events | `fseventsd` CPU |
| --- | ---: | ---: | ---: |
| Quiet empty directory | 2.7–4.6 s | 2 | ~2 s |
| Busy scratch root (87% of churn), `FileEvents` | 38.7–50.8 s | 2.56–2.74 M | 35–38 s |
| Same root, directory events | 41.0–42.9 s | 0.61 M | 34.5–35.5 s |
| Volume root, `FileEvents` | 88.8 s | 2.91 M | 50 s |

A root therefore pays for its own churn, even for a short window.
Coalescing to directory events cut delivered events 4.3× but not `fseventsd` CPU.

**The spike’s variance is not history growth, the instrument, or abandoned replays.
Its cause is unresolved.**

- Two day-old replays of one cursor took 32 s and then 92.5 s, about 15 minutes apart.
- The log behind that cursor grew only 1.6% in between.
- In between, full Python scans ran and five replays were abandoned.

Abandoned replays do not leave a backlog.
Abandoning an 8-hour replay after one second leaves `fseventsd` idle within about a
second, whether the client stops cleanly, calls `_exit`, or is killed.
The next replay then runs at baseline speed.

Concurrent replays do slow each other:

- Two concurrent 4-hour replays on one volume took 10.7 s each, against 6.2–7.0 s alone.
- A replay on the internal volume ran twice as slow while one ran on the external
  volume, so the scan capacity is shared across volumes and clients.
- Cursor lookups normally take 1–30 ms.
  Behind other work, some took 0.1–1.8 s, and one took 13.3 s.

Host load alone did not explain the rest.
A paired load-versus-no-load control showed no effect (10.6–10.8 s against 11.0 s). In
one batch, the slower runs even had less journal behind the cursor (69.7 MB) than the
faster ones (80.7 MB). A 1.6× spread at a fixed cursor and the 92.5 s run remain
unexplained; concurrent `fseventsd` clients are one measured cause, not a proven one for
that run.

**Flush and lookup semantics:**

- `FSEventStreamFlushSync` called right after start blocks until `HistoryDone` (3 of 3
  runs).
- `FSEventStreamFlushAsync` at start returns 0 at once.
- `FlushSync` after `HistoryDone` takes 0.05–0.08 ms.
- `FSEventsGetLastEventIdForDeviceBeforeTime` returns an ID at a log-file boundary at or
  before the requested time.
  That makes it conservative and cheap (1–30 ms when idle), but it queues behind
  `fseventsd`’s current work.

**Two more observations:**

- The spike’s per-event line-buffered `printf` added about 8 µs per event on a
  2.5-million-event replay, which is irrelevant for its 20-event runs.
  A deliberately slow client made `fseventsd` do more work.
- One device-relative stream with four paths failed `FSEventStreamStart` (2 of 2),
  although each path alone started.
  Paying for the log scan once across several roots is therefore untested.

**Budget rule:** before replaying, estimate

```text
0.124 s/MB × journal_MB_behind_cursor + 10 µs × expected_matching_records
```

and double it for contention.
Walk instead if the estimate exceeds the walk.

- **Walk rates:** the measured full-index walk costs 12.7 µs per entry (5.72 s at
  451,711 entries), and the summary-only walk 7.0 µs per entry.
- **Scratch volume (14 MB/h):** a quiet-folder replay breaks even with a 450,000-entry
  full-index walk at about 51 MB of journal: 3.7 hours of cursor age, or 1.8 hours with
  the contention factor.
  Against the summary-only walk it is 31 MB: 2.2 hours, or 1.1 hours.
- **Internal volume:** a quiet folder’s day of history (1.8 s) beats a full-index walk
  above about 140,000 entries.
- **For home on the internal volume:** the matching-record term is unknown, because
  nearly all of that volume’s records fall under home.

Bound each attempt by this computed budget, not by fixed 10, 60, or 120 s deadlines.
Abandoning a replay is free, so a budgeted attempt that falls back to the walk wastes
only the time already spent.

Evidence: [replay-cost](../../../explorations/change-sources/replay-cost/) and
[pilot](../../../explorations/change-sources/pilot/).

## FSEvents Cannot See Open Writers, Replayed or Live

A live stream received **no event while the file stayed open** in 3 of 3 repetitions for
each content-growth operation.
The stream was started after a settle period and flushed at +1 s and +10 s. Short-cursor
replay matched the live stream on every path and event ID (63 of 63 trials).

| Write path | Event while open | When the event arrives | `stat` while open |
| --- | --- | --- | --- |
| Append, then `fsync` or `F_FULLFSYNC` | none | last close of the open file description | size, blocks, mtime, ctime at once |
| In-place `pwrite` | none | last close | mtime, ctime |
| `dup`ed descriptor | none at first close | last close | updated |
| Shared mapping write plus `msync` | none | last unmap, not descriptor close | mtime and ctime at `msync` |
| SQLite WAL, connection held open (259 frames) | none for WAL growth | close | WAL size visible |
| Child process killed or exits | none while alive | at exit | updated |
| `ftruncate`, `O_TRUNC`, `F_PREALLOCATE`, utimes, chmod, xattr, clone | immediate metadata event | — | updated |
| Unlink or rename-over of an open file | immediate remove/rename | content event at close, on the stale path | open inode grows; free space drops |

The kernel source explains the table (xnu-12377, the macOS 26.0 tag):

- `write` only sets `FWASWRITTEN` (`bsd/kern/sys_generic.c`).
- `FSE_CONTENT_MODIFIED` is emitted in `vn_close()` (`bsd/vfs/vfs_vnops.c`) when the
  last reference to the fileglob drops, and in `ubc_unmap()` (`bsd/kern/ubc_subr.c`) for
  writable mappings.
- `fsync`, `fdatasync`, and `F_FULLFSYNC` emit nothing.
- Truncation, preallocation, time, mode, and xattr changes go through `vnode_setattr` or
  `VNOP_ALLOCATE`, which emit immediately.

The omission therefore happens when events are generated.
Watchman, Git’s fsmonitor daemon, Jujutsu with Watchman, and fdu’s own `--watch` all
consume this stream, so all share the gap on macOS.

**Linux is the opposite case.** inotify reports `IN_MODIFY` on every `write`, so a
resident Linux watcher has no open-writer gap.
Nothing persists without a listener, though.

Evidence: [writer-coverage](../../../explorations/change-sources/writer-coverage/) and
[harness-audit](../../../explorations/change-sources/harness-audit/).

### The Live-Root Misses Were Open Writers

The spike reported one, then two, stable misses on the 455,000-entry agent-state root.
An audit of the saved traces found a larger, systematic gap:

- **No event anywhere.** No event named any missed file, its parent, or any ancestor
  within the root.
- **Nothing dropped by the harness.** Re-running the harness normalization reproduced
  the saved scope plans exactly.
- **Open writers.** Both reported misses were held read/write by agent processes that
  had started about 51 hours before the fence.
- **Ordering.** In a controlled test, a close-time record received a higher ID than a
  sibling file created five seconds after the write.
  That rules out `fseventsd` lag.

Two classification choices hid the rest of the gap:

- **The “concurrent” bucket** held every path whose two oracles disagreed.
  That mixed changes made after the replay with stale pre-replay baselines.
  These were 4 and 4 of the unresolved paths, not intermediate states.
- **Shallow relisting** refreshed unevented files that share a directory with an evented
  one, which counted them as correct (8 and 8).

|  | First refresh | Repeat |
| --- | ---: | ---: |
| Reported stable misses | 1 | 2 |
| Stale baselines inside the concurrent bucket | 4 | 4 |
| Refreshed only because a sibling had an event | 8 | 8 |
| **Files changed before the replay with no event** | **13** | **14** |
| Of those, open read/write by an agent process when checked | 12 | 13 |

Every one was an agent session log (`.jsonl`) or a SQLite write-ahead log.
Future trials should report per-file coverage against the baseline, not just oracle
agreement.

### Finding Open Writers Directly

A libproc prototype needs no root.
It lists processes, then open descriptors, then vnode path information for each
descriptor.

| Measure | Value |
| --- | --- |
| Same-user processes readable | 100% (~590) |
| Processes denied (other users, root) | 23–27% (183–225 of 779–834) |
| Open-for-write regular files | ~2,000 files, ~2,500 descriptors, ~238 processes |
| Descriptor walk | 14–15 ms (33 ms cold); `lsof` agrees at 0.31 s |
| Deleted-but-open files | ~110 files, ~62 MB |
| Writable shared file mappings | ~255 on 7 files: 0.23–0.49 s with a private selector, 3.3 s public-only |

**Ordering with replay or a stream.** A refresh from cursor C runs in four steps:

1. Replay to `HistoryDone` at ID H, or flush a resident stream.
2. Then enumerate writers W.
3. Re-stat the named paths together with W.
4. Store H as the next cursor, never “now”.

Event IDs are assigned in arrival order, so a writer that closed before H is in the
replay, one still open is in W, and one that closed after H appears next time.

**What stays unobserved:** in-progress growth by processes the user cannot inspect,
mappings upgraded with `mprotect` (untested), and journal loss.
Deleted-but-open space is reportable only as a total.

**macOS alternatives, all insufficient here:**

- kqueue `EVFILT_VNODE` reports every write, but only for files a process already holds
  open.
- Endpoint Security reports every write, but needs root, an entitlement, and a system
  extension.

## Asking the Filesystem Instead

### `searchfs` by Change Time Works but Scans the Whole Volume

What works:

- `VOL_CAP_INT_SEARCHFS` is set on both APFS volumes.
- ctime and mtime bounds are honored exactly.
- Matches return object ID, parent ID, and sizes.
- Continuation was robust: 0 `EBUSY` in 756 calls.

What rules it out:

- Every query scans the volume’s catalog.
  The kernel returns `EAGAIN` about once per second, so a query is hundreds of calls.
- No subtree scoping is possible.
- Deletions and the descendants of a renamed directory are invisible to a timestamp
  query.

| Volume | Window | Matches | Wall time |
| --- | --- | ---: | ---: |
| Internal NVMe, 8.6 M inodes | none (zero matches) | 0 | 162.6 s |
| Internal | 1 h | 86,692 | 114.5 s |
| Internal | 1 day | 315,822 | 151.1 s |
| External USB SSD, 14 M inodes | 16 min | 65,706 | 327.6 s |

Verdict: not a refresh source.

### APFS Directory Statistics: A Subtree-Changed Signal That Sees Open Writers

APFS maintains optional per-directory statistics, which Apple calls fast directory
sizing (Apple File System Reference: `j_dir_stats_val_t`, `INODE_MAINTAIN_DIR_STATS`,
`INODE_DIR_STATS_ORIGIN`).

**How it works:**

- A directory marked as an origin keeps a recursive generation count, a descendant
  count, and a physical-size total for its whole subtree.
  These update in the same transaction as each change.
- The generation count is readable through the public `ATTR_CMNEXT_RECURSIVE_GENCOUNT`
  attribute, including in `getattrlistbulk`.
- On an origin, the totals come from a private `fsctl` in a few microseconds.
- Apple already uses it on this host’s iCloud sync roots.
  `~/Documents` advanced by 122 generations in 12 minutes.

**What was verified.** Two workstreams tested it: the first on the external volume, and
an adversarial second one with its own code on both volumes and inside a throwaway disk
image.

**Enabling and removing it:**

- **No privileges needed.** The shipped `apfs.util -M <dir>` issues a private `fsctl`
  (`0xC1104A71`, a 272-byte structure).
  It marked directories without privileges on the external volume, on an owned directory
  on the internal Data volume, and on a disk image mounted with ownership enforced.
- **Populated directories.** Apple’s guide says only empty directories are supported.
  On populated trees the totals were exact against a walk immediately.
- **The cost of marking is large.** Marking the root of a populated 225,653-entry
  fixture was a synchronous 66 s kernel operation.
  During it, a concurrent create/delete probe on the same volume slowed from 5.9 ms to
  70 ms at p90. Marking the 8,049 directories at depth 3 or less took 46 s.
- **Removing the flag.** The same `fsctl` with a different flag value clears an origin
  in 0.1 ms. It is undocumented, so it must be re-verified per release.
- **Recovery.** Marks and counts were identical before and after detach and reattach
  (the published `survived: false` flag is a script error; the values match).
  After a forced detach in the middle of a write burst, totals were still exact.
  `fsck_apfs -n` reported the image clean at every stage.
- **Drift on real volumes.** Apple’s discussion forums show First Aid reporting
  directory-statistics totals that drifted on real Data volumes and needed repair.
  A retired Apple engineer called the feature “never really hooked up or implemented
  right”
  ([summary](https://mjtsai.com/blog/2025/01/13/what-happened-to-apfs-fast-directory-sizing/)).

**What moves the generation count:** it updates synchronously, visible in the first read
about a microsecond after `write` returns.
Results were identical across three external-volume replicates and the internal volume.

- **It moves for** every size or membership change:
  - appends, in-place and same-size writes, and writes without `fsync`, including by an
    open writer;
  - truncation, deletion, and renames, including a populated directory (+1, not per
    descendant);
  - moves in and out;
  - hard links, and writes through a link that lives outside the tree;
  - clones, and SQLite WAL inserts while the connection is open;
  - `msync`, and resource-fork writes.
- **It does not move for:** chmod, utimes, xattr changes, reads, or a mapped store
  before `msync`.
- **Localization.** Only the origins on the path from a change to the root move.
- **New directories.** A directory created inside a marked tree inherits maintenance but
  is not an origin: its count reads 0, meaning “must descend”.
- **Totals are cheap only on origins.** The totals `fsctl` takes about 4 µs on an
  origin. On any other directory it walks the subtree in the kernel.

**Refresh at scale.** The fixture had 225,653 entries (2.2 GB), shaped like agent data:
a 20,000-file directory, deep trees, and Cargo-, `src`-, and `node_modules`-like
subtrees. The workload was agent-like:

- a 3,000-file build directory;
- a log and a SQLite WAL, both held open;
- a subtree deletion and a subtree rename;
- a clone, a hard link from outside, and deep appends;
- five mtime-only touches.

| Origins marked | Entries visited | Pruned refresh | Full walk | Missed size or membership changes |
| --- | ---: | ---: | ---: | ---: |
| Depth ≤ 3 (8,049) | 24% | 1.04 s | 7.9–9.7 s | 0 |
| Every directory (21,572) | 17% | 1.87 s | 9.6 s | 0 |

The five mtime-only touches were missed in both layouts, by design.

Every cell is a single run on a loaded host, and the walks around it varied widely.
The first full walk after marking the root took 111 s. One warm walk that read only the
generation count took 13.0 s, against 7.7–8.9 s without it.
Both are open questions, so treat the 1.04 s against 7.9–9.7 s ratio as one sample.

A totals-only diff, which reads each origin’s totals without listing anything, found the
119–131 changed origins in 44–150 ms.
It attributed bytes correctly except for the file hard-linked from outside the tree: a
file counts under the origin holding its primary name.

**Write overhead** (seven paired, interleaved rounds at depth 12; per-round ratios
ranged from 0.1× to 9.8×, so only the every-level row stands out from noise):

| Layout | Create | Append | Delete |
| --- | ---: | ---: | ---: |
| One origin | 1.01× | 0.92× | 0.96× |
| Origins at depth ≤ 3 | 1.10× | 0.75× | 1.07× |
| An origin at every level | 1.28× | 1.26× | 1.86× |

**Accounting.** A total counts the data-fork allocation of each regular file whose
primary link is inside the subtree:

- clones in full per holder;
- no resource-fork or xattr bytes;
- compressed files at their compressed allocation.

An open writer’s bytes appear only at `fsync`, at close, or 3.9–8.3 s later when the
syncer runs.
That makes the totals a cross-check, not fdu’s authoritative allocated size.

**Verdicts:**

- **As a skip-unchanged-subtree signal: promising, but parked.**
  - It cannot see mtime-only, chmod, or xattr changes.
    fdu serves recency as a first-class value, and its design principles let an
    accelerator cost speed, never accuracy.
    A pruned refresh could serve size and membership as verified only if recency is
    labeled as unverified or swept separately.
  - Its marking interface is reverse-engineered, and it leaves persistent flags on user
    data.
  - If it is revisited (`fdu-ns3n`), the conditions are:
    - read only `ATTR_CMNEXT_RECURSIVE_GENCOUNT` in the existing bulk walk, and treat 0
      as “descend”;
    - mark only on explicit opt-in, and only user-owned directories;
    - warn before marking a large populated root, and prefer marking a directory before
      it fills;
    - mark new directories as they are discovered, and never issue the totals `fsctl` on
      a non-origin.
- **As a totals source: no-go**, for the accounting rules and drift above.

**A separate lead.** On an *unmarked* 225,000-entry root, the same `fsctl` returned
exact totals from an in-kernel walk in 1.84 s, against 7.7–9.9 s for a userspace bulk
walk. That was one sample, with no side effect.
It is worth a measurement of its own as a fast summary path.

**Risks, most severe first:**

1. The flag is a persistent, inherited, undocumented on-disk change to user data.
   Only `fsck` checks it, and drift has been reported on real volumes.
2. Marking a populated tree is slow and degrades concurrent I/O.
3. New directories are not origins until fdu marks them.
4. The marking and unmarking interface is reverse-engineered; only the reader is public.
5. It is APFS-only, and iCloud and File Provider interplay was not tested.

Evidence: [catalog](../../../explorations/change-sources/catalog/) and
[dirstats-verify](../../../explorations/change-sources/dirstats-verify/).

### Clone-Private Size Is an Exact “Freeable” Measure

`ATTR_CMNEXT_PRIVATESIZE`, `CLONEID`, `CLONE_REFCNT`, and `EXT_FLAGS` are returned on
both volumes, even though `VOL_CAP_FMT_CLONE_MAPPING` is not advertised.

| Case | Private size |
| --- | --- |
| Fresh `cp -c` clone pair | 0 on both halves |
| After writing 8 MiB into the clone | 8 MiB on both |
| Each hard-link name | full size |

Adding the group costs about 5 µs per entry (2.5×), so it belongs in an opt-in
drill-down, not the default walk.
Retaining the link count (`ATTR_FILE_LINKCOUNT`) costs nothing measurable.

### Spotlight Is Not a Source

- Indexing is disabled on this host’s Data volume.
- On the external volume, where indexing is on, new fixture files were still not
  queryable after about five to seven minutes.
- Spotlight does not report deletions.
  Apple documents that it skips hidden directories such as `~/.codex`; this host’s test
  could not separate that exclusion from indexing backlog.

## What a Walk Costs, and Why the Snapshot Is Not a Shortcut

The installed fdu, which predates the `--cache auto|on|off` change, was measured warm on
the loaded host, with the cache on the external volume.
Its `--cache only` is today’s `--stale-ok`. At these sizes the regime is catalog-evicted
(`kern.maxvnodes` is 263,168).

| Scope | Entries | Full-index walk | Summary only | Load snapshot (`--stale-ok`) |
| --- | ---: | ---: | ---: | ---: |
| Agent state A (`~/.claude`) | 11,899 | 0.14 s | 0.11 s | 0.08 s |
| Agent state B (`~/.codex`) | 451,711 | 5.72 s (5.03–5.95), 209 MB | 3.14 s, 39 MB | 3.15 s, 300 MB, one thread |
| A project | 1,546,103 | 18.6 s (16.9–20.2) | 11.2 s | 10.0 s, 1.0 GB |
| Home | ~6.7 M | > 200 s (a bounded attempt timed out under load) |  |  |

**The snapshot load costs as much as a walk.** The flat snapshot rebuilds the whole
index on load, so a refresh that starts by loading it has spent about 55% of a walk
before observing anything.

**What the spike’s refresh time went to.** In the spike’s Python refresh of the same
455,000-entry root, 84% of 11 s went to whole-tree load, roll-up, and save.
Filesystem observation took 1.8 s.

**Changed directories are a small fraction of the tree.** Over a 14-minute interval of
agent activity on agent state B, 4,618 of 95,500 directory totals changed (+603 MB at
the root). Diffing only those records took 0.077 s.

By contrast, the checkpoint plan’s interim workflow compares two full JSON reports.
At this scope each report is 498 MB and takes 14–27 s to write, and the two take about 6
s to load.

Evidence: [architecture](../../../explorations/change-sources/architecture/).

## Architecture Findings

Ranked by how much each changes the design:

1. **A refresh that starts from the flat snapshot cannot beat a walk.** The checkpoint
   plan already says the flat loader and writer cost O(N), and that bounded persistent
   access (slice 4) is needed before fast whole-home refresh.
   The measurements confirm it: loading costs 55% of a walk.
   - There are two ways around it: a resident recorder that never reloads, or a store
     whose reads are proportional to change.
   - One candidate store is a per-directory roll-up log that records only directories
     whose totals changed, which is closed under ancestors.
     It would serve slice 4’s checkpoint reads, and it is sound for apparent and
     per-path allocated bytes.
   - It is not sound for unique-allocated attribution, where adding a link elsewhere
     moves a file’s size without a local change.
     Keep per-path allocated as the default ranking.
2. **Root identity misses firmlinks.** Path canonicalization leaves `/Users/…` and
   `/System/Volumes/Data/Users/…` as two cache keys for one tree.
   - The identity should be the volume UUID plus the path relative to that volume’s
     mount point.
   - `ATTR_CMNEXT_NOFIRMLINKPATH` gives that path for the Data volume, but not for other
     volumes. `/Volumes` is itself a firmlink, so an external path comes back as
     `/System/Volumes/Data/Volumes/<name>/…`, which embeds a mount name that can change.
   - A device-relative FSEvents filter needs the same volume-relative path.
     `Users/<u>/…` matched events, while `System/Volumes/Data/Users/<u>/…` matched only
     the `HistoryDone` sentinel.
   - The spike strips the mount point from `realpath` output.
     That works for `/Users` paths by luck.
3. **Reconcile compares the device number.** `dev` is an entry attribute that reconcile
   compares, so a remount or reboot that renumbers `st_dev` rewrites every entry.
   A delta store must treat a device-only change as a non-event.
4. **Quiet-cursor advancement needs a pre-replay fence.** Store the ID read before the
   stream starts, or the `HistoryDone` ID. Re-observing the overlap next time is
   harmless, because events nominate work rather than carry byte increments.
5. **Sweeps should be time-based.** Flags can force a sweep but never prove one
   unnecessary. Exposure grows with hours of history, not with the number of opens.
6. **Publication across files needs ordering.** Write and fsync blocks first, then the
   manifest. A dangling reference means “checkpoint unavailable”, never zero.
7. **The low-space diagnostic must not write into the volume it diagnoses.** The
   measured binary wrote a 35–114 MB snapshot to the cache on the nearly full Data
   volume for every one-shot report.
   - Under today’s `--cache auto`, a one-shot metadata report no longer writes one.
   - `--watch`, opened indexes, and content analysis still write to the default cache
     directory, which sits on the Data volume.
   - The disk-pressure profile needs an off-volume store for those.
8. **`watch` is a live observer that starts with a full reconcile.** It has the same
   open-writer blind spot, and it holds the whole index in memory.

## Accounting: Directory Deltas Are Not Free-Space Deltas

| Measure | Hard link | APFS clone | Open writer | Deleted but open |
| --- | --- | --- | --- | --- |
| Apparent bytes (`st_size`) | each path in full | each clone in full | seen | not seen |
| Allocated bytes (`st_blocks`) | each path in full | each clone in full | seen | not seen |
| Clone-private bytes | per inode | unshared blocks only | seen | not seen |
| Volume free space | exact | exact | exact | seen |

**What the walk could not see on this host during the review:**

- Three Time Machine local snapshots disappeared.
  The Data container’s free space rose from 4.87 GB to 11.0 GB with no tree change.
- This user’s processes held 633 unlinked-but-open files, about 931 MiB.
- Swap took 7 GiB in a sibling volume of the same container.
- A `cp -c` clone reported 64 MiB allocated while free space did not change.

A report should state four things:

- the per-volume free-space change, labeled as container-shared;
- per-scope, per-path directory deltas;
- an explicit signed residual that names its likely causes: snapshots, out-of-scope
  paths, deleted-open files, clone sharing, swap, and churn inside the interval;
- partial coverage, per row.

Net-zero churn inside the interval stays invisible to every measure.

## Resident Monitoring: `fdu --watch` Soak

fdu’s opened root with native observation already treats events as hints and verifies
them by `stat`. It also closes the registration gap before watching, and serves
`since(clock)`.

A one-hour soak ran `fdu --watch` read-only on agent state B, measured at 475,674
entries and 40.4 GB. Settings:

- `--interval 10s`, with the cache on the external volume;
- libproc writer enumeration every 30 s;
- a raw FSEvents listener for ten minutes;
- a final comparison of the watcher’s persisted view against two walks.

**Its view was exact, but only because it kept re-walking the tree:**

| Class | Paths |
| --- | ---: |
| Stable miss (both walks agree, watcher differs) | 0 |
| False positive | 0 |
| Changed between the two walks | 8, +1 MiB net (one SQLite file) |

Apart from that file, the per-directory totals matched the second walk exactly.
The soak did not hold the shared lock at startup, and other agents’ replays ran during
it, so its timings include contention.

**Why it was exact: every macOS rename escalates to a full-root reconcile.**

- The `notify` crate’s FSEvents backend reports each `ItemRenamed` as an unpaired
  rename, and `watch.rs` escalates any unpaired rename to the whole root.
- FSEvents flags are sticky per path, so later events on the same path escalate too.
- 61.5% of this root’s raw events carried `ItemRenamed`, mostly from atomic temp-file
  writes.
- The watcher therefore reconciled the entire root 172 times (174 root invalidations) in
  about an hour: once every 20 s, at 9.9 CPU s each.
- Those reconciles re-stat’ed the open writers and hid the gap.

**What that cost:**

| Measure | Value |
| --- | --- |
| Startup to first report | 60.5 s from a cold start: 7 s parallel scan, then a 37 s serial revalidation walk. From an existing snapshot, 33–73 s at 452k entries (518 MB) and 123–126 s at 1.5 M (1.6 GB) |
| CPU | 47.8% of one core over the hour, mostly system time; about 1% in the 5 of 114 30-second windows with no root reconcile |
| Resident memory | median 442 MB, peak 906 MB |
| Raw events | 326 per minute on average, bursts to 1,443 |
| Snapshot writes | 187 full rewrites of 36.7 MB, which is 6.9 GB written per hour |

**Without the root escalation, events alone would miss the writes.**

- 26 files grew in place between the start and the end.
- 11 of them were held open.
  Those 11 carry 99.8% of the in-place growth bytes.
- Re-stat’ing the writer list recovers all of it.
  Enumeration took 45 ms at the median and 7.3 s at worst under load (116 samples).
- Everything else in the change set was visible to events: 1,578 new entries, 1,072
  removed, and 202 metadata-only changes.
- Over the hour, 56% of gross growth, and nearly all in-place growth, was in files held
  open.

**How much state a recorder would need.** The hour touched 530 distinct parent
directories, plus about 70 open-for-write files per sample.
A resident recorder of dirty directories needs tens of kilobytes.
The resident index floor is about 190 B per entry, which is about 1.3 GB for this host’s
6.7 M-entry home.

**Scaling today’s watcher up:**

| Entries | Index | Per root reconcile | Per persist |
| --- | ---: | ---: | ---: |
| 1.5 M | ~285 MB | ~31 CPU s | ~116 MB |
| 5 M | ~950 MB | ~104 CPU s | ~386 MB |

At 1.5 M entries, one rename every 20 s already needs more than a core (31 CPU s per 20
s); at 5 M, about five cores.

**The conclusion:** a resident fdu is viable only after three fixes, filed as beads:

1. Scope one-sided renames (`fdu-822y`, P1).
2. Persist deltas on their own cadence instead of full rewrites (`fdu-88p7`).
3. Add the writer-list re-stat (`fdu-vhrb`).

With those fixes, the event-driven cost should approach the 1% of a core seen in
reconcile-free windows; the fix’s own re-soak must confirm it.
For home-scale roots, a dirty-directory recorder (`fdu-d2iz`) should replace the full
resident index. The 37 s startup revalidation is tracked separately (`fdu-eru0`).

## Operating-System Facilities That Compute Changes As They Happen

The principle is to compute usage changes as they happen rather than reconstruct them
later. A survey from primary sources, with local unprivileged tests, asked which
operating-system facilities already do that.
The full table and sources are in the
[survey](../../../explorations/change-sources/os-facilities/survey.md).

**macOS ships nothing that answers “what grew in the last hour”.**

- FSEvents gives changed names, with no sizes.
- APFS directory statistics give a *current* recursive size, with no history.
- Storage Management, the System Settings storage pane, sizes directories itself, leans
  on Spotlight, and exposes no API.
- Time Machine’s `backupd` carries three strategies: FSEvents, snapshot diffing, and a
  deep scan, each with catch-up variants.
  It ends in a deep scan when the others fail.

Apple’s own DiskSpaceDiagnostics service does use the APFS directory-statistics
interface (`APFSIOC_DIR_STATS_OP`), together with purgeable-space queries and
`getattrlistbulk`, for its “SpaceAttribution” snapshots.
That is evidence for the directory-statistics route, from the vendor.

| Facility | What it provides | Privilege | Verdict for fdu |
| --- | --- | --- | --- |
| Endpoint Security; `eslogger` | Per-event path plus `stat` for close-modified, create, unlink, rename, truncate, clone; per-write events name only the target | Root, Full Disk Access, and an Apple-granted entitlement for a native client; `eslogger` is “not API” | Opt-in diagnostic at most |
| `fs_usage`, `ktrace` (kdebug) | Per-syscall bytes and paths | Root (verified) | Opt-in diagnostic |
| `proc_pid_rusage` | Per-process lifetime bytes written, no paths | None for same-user processes | Process attribution hint |
| Volume accounting (`statfs`) | Used and free bytes per volume | None | Bound and consistency check |
| APFS snapshots and Time Machine diffs | Point-in-time trees; a private diff `fsctl` | Root plus entitlement | No |
| Spotlight | `kMDItemFSSize` per indexed item | None | No: new files unqueryable after 7 minutes here; hidden directories excluded |
| Linux XFS and ext4 project quotas | Kernel per-tree block and inode usage on every allocation | Root to set up and to read | Opt-in exact mode; the only true as-it-happens per-tree accounting found |
| btrfs quota groups, ZFS `written` | Per-subvolume or per-dataset usage | Root to enable (btrfs) | Where that layout exists |
| inotify | Per-write `IN_MODIFY`, no sizes, no mmap | None; `max_user_watches` | Default Linux live path |
| fanotify, eBPF `filetop` | Whole-filesystem events; per-file bytes | `CAP_SYS_ADMIN`; `CAP_BPF` and `CAP_PERFMON` | Opt-in privileged |
| `/proc/<pid>/io` | Per-process `write_bytes` | Same user | Process attribution hint |

Endpoint Security does not escape the open-writer problem for sizes.
Its close event’s `modified` flag is the same `FWASWRITTEN` bit that drives the FSEvents
content event. Its per-write event carries neither byte count nor offset.

**Per-process counters, measured.** The unprivileged sampler covered 182–214 same-user
processes per sample; 59–83 other-user processes returned `EPERM`. A sample usually cost
3–19 ms; one took 323 ms.

Over 60 s, same-user processes wrote 22–66 MiB, and an agent command-line tool was the
top writer both times.
The counters have gaps:

- Processes that exit lose their counters.
  One run lost 2.0 GiB of lifetime writes between samples.
- Writes to the external volume are absent from the logical-write ledger.
- Overwrites and deletions are not netted out.

So the counters answer “which process wrote a lot”, never “which directory grew”.

**The best unprivileged architecture, macOS.** It combines:

- fdu checkpoints;
- FSEvents, live while running and replayed otherwise;
- `stat` for sizes;
- the libproc open-writer list;
- optionally, the directory-statistics generation count as a gate (parked; see above);
- per-process counters for attribution;
- the per-volume free-space delta as a bound, for example “volume grew 12 GB, attributed
  11.5 GB”.

**On Linux,** the same combination uses inotify, or unprivileged fanotify inode marks on
kernel 5.13 or later, with `/proc/<pid>/io` and no journal replay.

**What an opt-in privileged mode adds:** `eslogger`-based exact attribution across all
users on macOS; project quotas, fanotify filesystem marks, and eBPF per-file bytes on
Linux.

**The strongest case against the principle:**

- Everything precise needs root, so the default path is approximate either way.
- Always-on cost lands on the whole volume.
- Observer downtime forces gap detection plus a rescan, so the rescan must be fast
  anyway.
- Bookkeeping drift (cursor invalidation, open writers, mappings) is a permanent
  correctness liability that a stateless walk does not have.
- Apple’s own backup daemon ends the same ladder in a deep scan.

## Prior Art

No surveyed system attributes growth over a time window to directories.
The consumers of FSEvents split two ways:

- Working-copy tools keep a resident monitor with a clock and recrawl when it is lost.
- Backup tools replay between runs and force periodic full audits.

The tools that are correct about missed writers compare `stat` facts on every run:
restic, Borg, Kopia, Backblaze.
Arq instead reads from a snapshot.
Sources for the rows below are listed in
[prior-art-sources](../../../explorations/change-sources/architecture/raw/prior-art-sources.txt);
Time Machine’s behavior comes from secondary sources, because Apple publishes none.

| System | Change source | Lost or unavailable history | Problem solved |
| --- | --- | --- | --- |
| Jujutsu | Parallel walk by default; optional Watchman with a persisted clock | Watchman restart: full crawl | Repository change detection |
| Watchman | Resident FSEvents or inotify stream behind a clock | Fresh instance: recrawl; in-process FSEvents resync reverted over correctness | Working-copy change lists |
| Git builtin fsmonitor | Resident FSEvents from `SinceNow` | New token: full `lstat` | Working-copy status |
| Carbon Copy Cloner Quick Update | FSEvents since the last successful run | Audit when history is unavailable or the last success is older than two weeks | Eventual backup consistency |
| SuperDuper Turbo | FSEvents change database | Silent full compare | Backup consistency |
| Time Machine | Stored FSEvents ID | “Deep traversal”, often over an hour | Backup consistency |
| restic, Borg, Kopia | Full walk; compare mtime, ctime, size, inode | Re-read everything | Backup |
| ncdu, gdu, dust, DaisyDisk | Full walk; export for manual comparison | — | Static usage |

**Jujutsu in detail** (source review of v0.45.1, `lib/src/fsmonitor.rs` and
`lib/src/local_working_copy.rs`):

- **Default path.** jj compares file type, millisecond mtime, and size against its saved
  tree state. It uses Git’s racy-mtime rule: a file is clean only if its mtime precedes
  the state file’s own mtime.
- **Optional build feature.** It adds Watchman through `watchman_client`, and nothing
  else: no direct FSEvents, no inotify, no Spotlight.
- **Clock handling.** A fresh Watchman instance returns “crawl everything”, and the
  clock is not advanced on an empty result.
- **Issues that match this review’s lessons:**
  - an empty answer with a valid clock reported a clean working copy
    ([#10097](https://github.com/jj-vcs/jj/pull/10097));
  - about 90 modified files stayed hidden for nine hours until a full scan
    ([#10130](https://github.com/jj-vcs/jj/pull/10130));
  - Watchman cookie files caused a snapshot loop at a million files
    ([#9818](https://github.com/jj-vcs/jj/issues/9818));
  - `jj st` went from 15 ms to 4 s with Watchman on macOS
    ([#5826](https://github.com/jj-vcs/jj/issues/5826)).
- **Maintainers’ stance.** In discussion
  [#8204](https://github.com/jj-vcs/jj/discussions/8204), they report no definite plan
  to replace Watchman, and longer-term interest in a virtual filesystem.
  That is the same “let the filesystem maintain the answer” idea as APFS directory
  statistics.

The
[persistent-change prior-art research](research-2026-09-27-persistent-change-prior-art.md)
covers Carbon Copy Cloner, SuperDuper, and Watchman’s synchronization warning.
The [performance-frontier research](research-2026-08-10-performance-frontier.md) covers
Watchman’s and Git’s FSEvents configuration.

## Linux

Linux has no persistent change journal, so the full walk is the fallback.
The event facilities differ from macOS in useful ways:

- **inotify** reports every write, so a resident watcher has no open-writer gap.
  It needs one watch per directory and hits `max_user_watches` on large trees, and it
  drops events on queue overflow.
- **fanotify** filesystem marks need `CAP_SYS_ADMIN`.
- **`/proc/<pid>/fd`** and `fdinfo` enumerate open writers, including `(deleted)` files.
- **Filesystem-specific sources** have separate privileges and gates:
  - btrfs `find-new` (data extents only);
  - `btrfs send --no-data` (needs snapshots);
  - `zfs diff` (needs snapshots and delegation).

## Options Compared

These are end-to-end estimates from invocation to a ranked answer, excluding the first
baseline, on this host.

| Option | 450k entries, 1 h | 450k entries, 1 day | ~6.7 M (home), 1 h / 1 day | Correctness gaps |
| --- | --- | --- | --- | --- |
| (a) Full walk plus delta-store diff | ~6 s (measured walk) | ~6 s | minutes | None beyond `stat`’s view; needs a retained baseline |
| (b) One-shot replay plus writer list plus scoped relist | replay (volume journal plus matching) + relist; breaks even with a full-index walk at 1.8–3.7 h of scratch-volume history | scratch volume ~45 s; internal quiet ~2 s | unknown: home’s matching records dominate | Other users’ writers; journal completeness unproven |
| (b′) (b) with today’s flat snapshot | adds 3 s load plus save | same | adds ~45 s load | as (b) |
| (c) Resident fdu monitor plus writer list, feeding the store | milliseconds | milliseconds | milliseconds while running; today’s watcher re-walks the root on every rename (48% of a core at 476k entries) and holds ~190 B per entry | Other users’ writers; downtime needs replay or reconcile; needs the rename, persistence, and recorder fixes |
| (d) APFS directory-statistics pruned refresh | 1.04 s at 225k entries (one run, relists included) | same | lists changed origins only, no daemon | Private marking interface; slow marking of populated trees; mtime-only changes; APFS only |
| (e) `searchfs` by ctime | 115–330 s | same | same | Deletes and renamed-directory contents |
| (f) Linux: resident inotify watcher plus walk-captured checkpoints | milliseconds while running | same | inotify needs one watch per directory; the limit at home scale is unquantified | Nothing persists without a listener; a walk after downtime |

## Recommendation and the Case Against It

1. **Fix the watcher first.** These are engine bugs under any plan:
   - scope one-sided renames to their own path (`fdu-822y`);
   - persist on a cadence decoupled from the render interval, or as deltas (`fdu-88p7`);
   - re-stat the libproc writer list at each checkpoint (`fdu-vhrb`).

   Then re-run the soak at 476k and 1.5 M entries.
   The gate is exactness against two walks at under 2% of a core.

2. **Ship the hour and day report for small scopes now.** Project, agent-state, and
   temporary scopes can use walk-captured checkpoints (the checkpoint plan’s slice 2),
   where walks cost 0.1–19 s. The same code runs on Linux.
   - Key identity by volume UUID plus the path relative to the volume’s mount point.
   - Keep the link count, rank by per-path allocated bytes, sample free space per
     volume, and report a labeled residual.
   - Keep the low-space diagnostic’s writes off the volume being diagnosed.

3. **For home scale, prototype a resident dirty-directory recorder** (`fdu-d2iz`) in the
   watch layer: a persisted set of dirty directories plus the writer list.
   - Measure recorder, relist, and roll-up diff at home scale.
   - Build the plan’s bounded persistent store (slice 4) only if the flat load then
     dominates.
   - The fixed watcher at one reconcile per change is the recorder’s core.
     The soak showed that re-stat’ing open writers and relisting evented parents was
     exact at 476k entries.

4. **Keep one-shot replay for gap recovery only**, under the budget rule at measured
   walk rates. Always pair it with the writer list, and never trust `HistoryDone` alone.

5. **Park APFS directory statistics as a research result.** Pursue in-kernel sizing of
   unmarked directories (`fdu-22hd`) as the daemonless lead, because it has no side
   effect.

6. **Make background monitoring an explicit opt-in**, with an energy budget and a Linux
   watch-limit check.

**The case against this recommendation:**

- The recorder is unbuilt and unproven at home scale, and the whole-home answer in
  seconds depends on it.
- “One hour ago” needs a baseline captured an hour ago, so a scheduled capture or a
  resident process is implied either way.
  That is a consent decision.
- If the recorder still has to load the flat snapshot for roll-ups, slice 4 is needed
  anyway, and building the store first would have been faster.
- Parking directory statistics gives up the only daemonless mechanism that sees open
  writers natively.

**Before acting, account for:**

- energy and battery cost of a resident monitor on a laptop;
- multi-user hosts, where other users’ writers stay invisible;
- external-volume unmount and remount (device renumbering, cursor invalidation, changed
  mount names);
- SSD wear from persistence (6.9 GB per hour today);
- privacy of a persistent log of what changed when;
- OS-upgrade risk to FSEvents semantics and to any private interface.

## Ranked Next Experiments

The beads sit under epic `fdu-tawn`. It also tracks two engine fixes the review found:
firmlink-free root identity (`fdu-43bc`) and keeping low-space diagnostics from writing
into the diagnosed volume (`fdu-hbjp`).

| Rank | Bead | Experiment | Discriminating outcome | Go if |
| --- | --- | --- | --- | --- |
| 1 | `fdu-822y` | Scope one-sided macOS renames in `watch`, then re-soak | Root reconciles per hour and CPU on the same root, with the writer list added | CPU under 2% of a core; no stable misses against two walks |
| 2 | `fdu-88p7`, `fdu-vhrb` | Persistence cadence and writer-list re-stat | Bytes written per hour; per-file coverage | Persistence bounded by changes; every held-open change reflected |
| 3 | `fdu-d2iz` | Resident dirty-directory recorder | Recorder state, CPU, and checkpoint cost at home scale | Tens of KB of state; checkpoint ≤ relist of dirty directories |
| 4 | `fdu-cv15` | Writer list over three live refreshes | Share of changed files in the event set or the writer set | ≥ 99%, remainder attributed to other users |
| 5 | `fdu-yj8z` | Home-filter replay cost on the internal volume (1 h, 24 h) | Size of the matching-record term for home | Replay plus relist ≤ 25% of a home walk |
| 6 | `fdu-uq1y` | Bounded store (delta-only roll-up log) | Capture writes only changed roll-ups; query time; daily growth | Only if the recorder’s flat load dominates |
| 7 | `fdu-22hd` | In-kernel sizing of unmarked directories | Replicate 1.84 s vs 7.7–9.9 s at 225k; accounting against fdu’s totals | A faster exact summary path with a stated accounting rule |
| 8 | `fdu-befp` | Multi-path device-relative stream | Why `FSEventStreamStart` fails with several paths | One log scan for several roots |
| Parked | `fdu-ns3n` | Opt-in gencount-gated walk | See the directory-statistics verdict | Only with a recency answer and a documented marking contract |

Done: `fdu-gpqz` (directory-statistics verification), `fdu-2o00` (resident soak), and
`fdu-lwcz` (no replay backlog).

## Corrections to Earlier Records

The spike’s evidence stands, but several statements needed narrowing.
They are reflected in the
[spike README](../../../explorations/fsevents-replay/README.md) on PR #131’s branch:

- **Miss counts.** The live-root runs had 13 and 14 files changed without an event, not
  one and two misses.
- **“Intermediate states”.** That explanation holds only for the full-root run.
- **The asynchronous flush** returned immediately.
  It was `HistoryDone` that never arrived.
- **The synchronous-flush kill** measured the same history cost, not an independent
  failure.
- **The growth workload** did not exercise hard-link alias expansion, or per-file
  coverage for files that share a directory with an evented file.
- **The close-time event** fires on the last close of the open file description.
  Mappings notify at unmap, and `fsync` never notifies.
- **Creation-history overlap** depends on journal segmentation, not wall-clock age.

Other corrections:

- **Replay cost.** It should be stated per journal megabyte and per matching record, not
  per hour. The 60–92 s tails came from the churning scratch volume and from contention
  with other `fseventsd` clients on a loaded host.
- **Spotlight** is not available on this host’s Data volume.
- **The interim JSON workflow** costs 498 MB and 14–27 s per checkpoint at 452,000
  entries.

## References

- [Change-source evidence](../../../explorations/change-sources/README.md)
- [FSEvents replay spike](../../../explorations/fsevents-replay/README.md)
- [FSEvents-scoped revalidation plan](../specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
- [Disk-usage checkpoint plan](../specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md)
- [Persistent-change prior art](research-2026-09-27-persistent-change-prior-art.md)
- [Performance frontier](research-2026-08-10-performance-frontier.md)
- [Metadata walk floor](../reports/report-2026-08-23-metadata-walk-floor.md)
- [Apple FSEvents Programming Guide](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html)
- [Apple File System Reference](https://developer.apple.com/support/downloads/Apple-File-System-Reference.pdf)
- [XNU source](https://github.com/apple-oss-distributions/xnu)
- [Jujutsu](https://github.com/jj-vcs/jj)
- [Watchman cookies and the macOS limitation](https://facebook.github.io/watchman/docs/cookies)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
