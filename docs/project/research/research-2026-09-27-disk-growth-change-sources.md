# Finding Disk Growth Fast: Change Sources Compared

**Date:** 2026-09-27

**Status:** Review of the FSEvents replay spike and its alternatives.
Research only: no engine or command-line behavior changes.
Items marked *pending* are experiments still running when this was written.

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
  cheapest walk. Every incremental design first needs a delta-only checkpoint store, in
  which a query costs O(changed directories).
- **Asking the filesystem gave one strong candidate and one dead end.**
  - `searchfs` by change time works, but costs 110–330 s per query regardless of scope.
  - APFS fast directory sizing keeps a per-tree generation count current on every write,
    including writes through descriptors held open.
    It can be enabled without privileges through a shipped tool, but that tool uses a
    private interface and leaves a persistent flag on the directory.
    Verification is *pending*.
- **Open writers can be listed directly.** Same-user processes’ open-for-write files
  enumerate through libproc in about 15 ms without root.
  A monitor or replay plus this list covers the macOS gap, except for writers owned by
  other users.
- **Directory deltas do not explain free-space changes by themselves.** During the
  review, Time Machine local snapshots released about 6 GB with no tree change, and 931
  MiB was held by deleted-but-open files.
  A report needs a signed, labeled residual.

The [recommendation](#recommendation-and-the-case-against-it) is:

1. Build the delta-only checkpoint store and the identity fixes first, captured by
   walks. This is identical on Linux.
2. Choose the home-scale accelerator between a resident fdu monitor with an open-writer
   supplement and APFS directory statistics, using the experiments ranked at the end.
3. Keep one-shot replay only for bounded gaps, under an explicit cost budget.

## How the Review Was Run

All measurements come from one host: macOS 26.5.2, arm64, 10 cores, 32 GiB.

- **Volumes:** an internal APFS Data volume (8.6 million inodes, 98–99% full) and an
  external APFS USB SSD (14 million inodes).
  The external volume holds agent worktrees and Cargo targets, so it churns heavily.
- **Load:** the host ran concurrent agents throughout, with load averages of 6–28.
- **Timing isolation:** timed and volume-wide runs held one shared lock, so parallel
  workstreams did not overlap them.
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
about 17 times a non-matching one.
The same 2-hour cursor, with 2.8–3.8 million records behind it, cost:

| Filter | Wall time | Delivered events | `fseventsd` CPU |
| --- | ---: | ---: | ---: |
| Quiet empty directory | 2.7–4.6 s | 2 | ~2 s |
| Busy scratch root (87% of churn), `FileEvents` | 38.7–50.8 s | 2.56–2.74 M | 35–38 s |
| Same root, directory events | 41.0–42.9 s | 0.61 M | 34.5–35.5 s |
| Volume root, `FileEvents` | 88.8 s | 2.91 M | 50 s |

A root therefore pays for its own churn, even for a short window.
Coalescing to directory events cut delivered events 4.5× but not `fseventsd` CPU.

**The spike’s variance came from contention, not history growth or the instrument.**

- Two day-old replays of one cursor took 32 s and then 92.5 s, about 15 minutes apart.
- The log behind that cursor grew only 1.6% in between.
- In between, full Python scans ran and five replays were abandoned.

Abandoned replays do not leave a backlog.
Abandoning an 8-hour replay after one second leaves `fseventsd` idle within about a
second, whether the client stops cleanly, calls `_exit`, or is killed.
The next replay then runs at baseline speed.

Contention does slow replays:

- The same 84 MB cursor took 6.2–7.0 s at one time and 10.0–11.0 s at another, with only
  host state changing.
- Two concurrent 4-hour replays on one volume took 10.7 s each, against 6.2–7.0 s alone.
- A replay on the internal volume ran twice as slow while one ran on the external
  volume, so the scan capacity is shared across volumes and clients.
- Cursor lookups normally take 1–30 ms.
  Behind other work, some took 0.1–1.8 s, and one took 13.3 s.

One or two concurrent `fseventsd` clients plus a loaded host produce a slowdown of this
size. Which client slowed the 92.5 s run is unknown.

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

- **Break-even on the scratch volume (14 MB/h):** a 450,000-entry project breaks even at
  40–70 minutes of cursor age.
- **On the internal volume:** replay wins for a quiet folder above about 1 million
  entries.
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
| Processes denied (other users, root) | 22–27% (183–220 of ~800) |
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
- The totals come from a private `fsctl` in about 2 µs.
- Apple already uses it on this host’s iCloud sync roots.
  `~/Documents` advanced by 122 generations in 12 minutes.

**What the first workstream found, on its own fixtures on the external volume:**

- **Enabling it.** The shipped `apfs.util -M <dir>` marked directories without
  privileges. That included populated trees: a 6,265-entry, 45 MB tree took 40 ms, with
  exact totals immediately.
  Apple’s guide says only empty directories are supported.
- **What moves the generation count:**
  - an open writer’s `write` before any `fsync` or close;
  - deep appends (depth 13 and 121);
  - truncation, deletion, renames, and moves in and out;
  - hard links and clones;
  - `msync` of a mapping;
  - a write through a hard link that lives outside the tree.
- **What does not:** chmod, utimes, xattr changes, and reads.
- **Localization.** With 55 nested origins, one deep append changed exactly the three
  origins on its path.
- **Write overhead.** With one origin, writes slowed by 3–11%. A 13-deep chain of
  origins was too noisy to measure on the loaded host.

**How a refresh would use it:**

1. Read the root’s generation count, which takes microseconds.
2. If it is unchanged, nothing below it changed size or membership, including files held
   open.
3. If it changed, list the root, descend only into child origins whose count changed,
   and walk unmarked subtrees fully.

**Risks, most severe first:**

1. Marking uses a private, undocumented interface.
2. The flag is persistent and inherited on user data, with no documented way to clear
   it.
3. Marking populated directories is undocumented behavior.
4. The count reports *that* a subtree changed, not *what* changed.
5. Its size accounting differs from fdu’s: clones count in full per holder, and hard
   links once.
6. It is APFS-only.

**Verification (pending).** An adversarial verification covers five areas:

- privilege on the internal volume;
- persistence and `fsck` behavior across detach and forced detach, inside a throwaway
  disk image;
- scale at 200,000–300,000 entries;
- the pruned refresh checked against a full-walk oracle;
- exact accounting semantics.

Evidence: [catalog](../../../explorations/change-sources/catalog/).

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
- On the external volume, fixture files were still unindexed after 15 minutes.
- A 2-hour volume query returned 372 hits, against 65,706 ctime changes in 16 minutes.
- Spotlight excludes hidden directories such as `~/.codex` by design, and it does not
  report deletions.

## What a Walk Costs, and Why the Snapshot Is Not a Shortcut

The installed fdu, which predates the `--cache auto|on|off` change, was measured warm on
the loaded host, with the cache on the external volume.
Its `--cache only` is today’s `--stale-ok`. At these sizes the regime is catalog-evicted
(`kern.maxvnodes` is 263,168).

| Scope | Entries | Full-index walk | Summary only | Load snapshot (`--stale-ok`) |
| --- | ---: | ---: | ---: | ---: |
| Agent state A (`~/.claude`) | 11,899 | 0.14 s | 0.11 s | 0.08 s |
| Agent state B (`~/.codex`) | 451,711 | 5.72 s (5.03–5.95), 220 MB | 3.14 s, 41 MB | 3.15 s, 315 MB, one thread |
| A project | 1,546,103 | 18.6 s (16.9–20.2) | 11.2 s | 10.0 s, 1.0 GB |
| Home | ~6.7 M | minutes (not completed) |  |  |

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

1. **The store comes before the change source.** A delta-only checkpoint store makes
   “now vs T” cost O(records in the interval): a per-directory log that records only
   directories whose totals changed, which is closed under ancestors.
   It is sound for apparent and per-path allocated bytes.
   - It is not sound for unique-allocated attribution, where adding a link elsewhere
     moves a file’s size without a local change.
     Keep per-path allocated as the default ranking.
   - The plan sequences replay before bounded persistent access.
     The measurements say the store is a prerequisite.
2. **Root identity misses firmlinks.** Path canonicalization leaves `/Users/…` and
   `/System/Volumes/Data/Users/…` as two cache keys for one tree.
   - `ATTR_CMNEXT_NOFIRMLINKPATH` plus the volume UUID is the right identity.
   - A device-relative FSEvents filter must be the firmlink-free path minus the mount
     point: `Users/<u>/…` matched events, while `System/Volumes/Data/Users/<u>/…`
     matched nothing.
   - The spike strips the mount point from `realpath` output.
     That works for `/Users` paths by luck.
3. **The device number sits in every entry fingerprint.** A remount or reboot that
   renumbers `st_dev` therefore rewrites every entry.
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

## Resident Monitoring (pending)

fdu’s opened root with native observation already treats events as hints and verifies
them by `stat`. It also closes the registration gap before watching, and serves
`since(clock)`. Run as a resident process, it removes replay cost and makes queries
immediate.

It does not remove three gaps:

- On macOS it still needs the open-writer supplement.
- It holds the whole index: 220–315 MB at 450,000 entries.
- It covers only the time it runs.
  After downtime it must replay the gap or reconcile.

A one-hour soak of `fdu --watch` on agent state B is *pending*. It is read-only, with
writer enumeration every 30 s and a final comparison against two walks.
It measures resident cost and classifies every difference as caught by the watch,
recoverable by the writer list, or unexplained.

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
processes per sample; 59–83 other-user processes returned `EPERM`. A sample cost 3–19
ms.

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
- optionally, the directory-statistics generation count as a gate;
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
| (b) One-shot replay plus writer list plus scoped relist, delta store | replay (volume journal plus matching) + relist | scratch volume ~45 s; internal quiet ~2 s | unknown: home’s matching records dominate | Other users’ writers; journal completeness unproven |
| (b′) (b) with today’s flat snapshot | adds 3 s load plus save | same | adds ~45 s load | as (b) |
| (c) Resident fdu monitor plus writer list, feeding the store | milliseconds | milliseconds | milliseconds while running; about 3 GB resident with today’s index | Other users’ writers; downtime needs replay or reconcile |
| (d) APFS directory-statistics pruned refresh | list changed origins only (*pending*) | same | same, no daemon | Private marking interface; APFS only |
| (e) `searchfs` by ctime | 115–330 s | same | same | Deletes and renamed-directory contents |

## Recommendation and the Case Against It

1. **Build the checkpoint store first, captured by walks.**
   - A per-directory roll-up log with per-path allocated ranking.
   - Keep the link count.
   - Key identity by volume UUID plus firmlink-free path.
   - Sample free space per volume, and report a labeled residual.
   - This serves project, agent-state, and temporary scopes in seconds, is correct for
     open writers, and runs the same code on Linux.
   - Keep the low-space diagnostic’s writes off the volume being diagnosed.

2. **For home scale, pick an accelerator by experiment, not by precedent.**
   - A resident fdu monitor with the open-writer list is public-API, cross-platform
     (complete on Linux), and reuses the opened-root contract.
     It needs a lighter resident mode than today’s full in-memory index, and an explicit
     decision to allow background monitoring.
   - APFS directory statistics are daemonless and see open writers natively, but rest on
     a private marking interface and leave a persistent flag on user data.
   - Both need a periodic full sweep as the safety net.

3. **Demote one-shot replay** to bounded roles:
   - recovering a monitor’s downtime;
   - a no-daemon fallback when the budget rule predicts it beats the walk.

   Always pair it with the writer list, and never trust `HistoryDone` alone.

**The case against this recommendation:**

- Stage 1 does not meet “seconds” for the whole home folder.
- “One hour ago” needs a baseline that exists only if something ran an hour ago.
- A background job or monitor is therefore unavoidable for the headline workflow.
- Deferring the accelerator defers that workflow.
- Two persisted stores per root (the flat snapshot for opened roots and analysis, the
  delta log for checkpoints) is a transitional duplication.
  The design principles warn against it.

## Ranked Next Experiments

The beads sit under epic `fdu-tawn`. The abandoned-replay backlog test (`fdu-lwcz`) is
done: there is no backlog.
The epic also tracks two engine fixes the review found: firmlink-free root identity
(`fdu-43bc`) and keeping low-space diagnostics from writing into the diagnosed volume
(`fdu-hbjp`).

| Rank | Bead | Experiment | Discriminating outcome | Go if |
| --- | --- | --- | --- | --- |
| 1 | `fdu-gpqz` | APFS directory statistics, adversarial verification (*pending*) | Privilege on the internal volume; persistence and `fsck` after forced detach; pruned refresh against a walk oracle at 200k–300k entries | Zero oracle misses across randomized workloads including open writers; refresh ≤ 10% of the walk; write overhead ≤ 10% at realistic depth; a clean removal path |
| 2 | `fdu-2o00` | Resident soak of `fdu --watch` on agent state B (*pending*) | Resident memory and CPU; misses caught by the writer list vs unexplained | Every stable miss explained by an open writer or another user; steady CPU below 1% of a core |
| 3 | `fdu-yj8z` | Home-filter replay cost on the internal volume (1 h, 24 h; directory events) | Size of the matching-record term for home | Replay plus relist ≤ 25% of a home walk |
| 4 | `fdu-uq1y` | Delta-store prototype | Capture writes only changed roll-ups at 450k and 1.5 M; query time; daily state growth | Capture ≤ walk + 5%; query ≤ 0.2 s; growth bounded |
| 5 | `fdu-cv15` | Writer list over three live refreshes | Share of changed files in the event set or the writer set | ≥ 99%, remainder attributed to other users |
| 6 | `fdu-befp` | Multi-path device-relative stream | Why `FSEventStreamStart` fails with several paths | One log scan for several roots |

## Corrections to Earlier Records

The spike’s evidence stands, but several statements needed narrowing.
These are now reflected in the
[spike README](../../../explorations/fsevents-replay/README.md):

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
