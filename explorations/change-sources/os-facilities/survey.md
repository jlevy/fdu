# OS facilities that track disk-usage change as it happens (macOS, Linux)

**Date:** 2026-09-27. **Host:** macOS 26.5.2 (Darwin 25.5.0, xnu-12377.121.10), APFS,
loaded M-series Mac.
**Privilege used:** none (no sudo, no settings changes).
**Companion results:** `results/` (sanitized), `src/rusage_sampler.c`.

Sibling reviewers already established (not re-tested here): FSEvents replay costs about
0.12 s per MB of volume journal behind the cursor and about 10 µs per matching record;
FSEvents emits no content event for writes through an open descriptor or mapping until
last close/unmap (XNU `vn_close`); `searchfs` by ctime takes 110–330 s volume-wide;
Spotlight indexing is disabled on this host’s Data volume and excludes hidden
directories; APFS fast directory sizing (`apfs.util -M`,
`ATTR_CMNEXT_RECURSIVE_GENCOUNT`) is being verified separately; libproc open-writer
enumeration costs about 15 ms.

## Summary table

| Facility | Maintains / emits | Latency | Completeness | Privilege | Availability | Cost | Verdict for fdu |
| --- | --- | --- | --- | --- | --- | --- | --- |
| FSEvents (live, replay) | Changed paths; no sizes; content events at last close/unmap | Seconds live; replay ~0.12 s/MB journal | Misses open writers until close; other users’ paths visible | None | All macOS | fseventsd CPU ∝ volume churn | Default path (already the plan) |
| Endpoint Security, native client | Per event: path + `struct stat` at event time for CLOSE(`modified`, `was_mapped_writable`), CREATE, UNLINK, RENAME, TRUNCATE, CLONE, COPYFILE, EXCHANGEDATA, LINK, SETATTRLIST; WRITE carries target only (no byte count, no offset) | Immediate | Everything on the host incl. other users; drops signalled by `seq_num`/`global_seq_num` gaps; mmap writes only via `was_mapped_writable` | root + Full Disk Access (TCC) + Apple-granted entitlement `com.apple.developer.endpoint-security.client`, packaged as a system extension | macOS 10.15+; `was_mapped_writable` msg v6 | Undocumented; muting/inverted muting APIs exist to bound it | No for an unprivileged open-source tool (entitlement gate) |
| `eslogger` | Same events as JSON Lines | Immediate | As above | root + FDA | Ships with macOS | As above | At most an opt-in diagnostic via `sudo`; Apple: “NOT API … may change from release to release” |
| kdebug: `fs_usage`, `ktrace` | Per-syscall records with `B=bytes` and path as they happen | Immediate | All processes; ring buffer can drop | root ("must be run as root"; `ktrace info` fails unprivileged); DTrace also SIP-gated | Ships with macOS | Undocumented; buffer-size flag | Opt-in diagnostic only |
| `proc_pid_rusage` (RUSAGE_INFO_V4+) | Per-process lifetime `ri_diskio_byteswritten` (physical), `ri_logical_writes` (internal storage only); no paths | Instant sample (2.8–19 ms for ~265 pids here; one 323 ms outlier under load) | Same-uid processes only (EPERM otherwise, verified); exited processes lose counters; writes to external volumes absent from `ri_logical_writes`; overwrites/deletes not netted (except invalidated dirty pages) | None | macOS 10.9+ (V4: 10.12-era) | Negligible | Hint: process attribution, sampled between checkpoints |
| Coalition I/O ledgers (`coalition_info_resource_usage`) | Per-app-coalition logical writes | Instant | Same limits | Private (no SDK header; `sysmond` imports it) | — | — | No |
| APFS snapshots + Time Machine snapshot diff | Point-in-time tree; TM diffs two snapshots (`TMSnapshotDiffer`, private `fsctl`) | Minutes (TM job) | Complete at snapshot boundaries | “All snapshot functions require superuser privileges and also require an additional entitlement” (`fs_snapshot_create(2)`); `tmutil` verbs need root + FDA | 10.12+ | Snapshot retention pins deleted blocks | No |
| APFS volume space accounting (`statfs`, `ATTR_VOL_SPACEUSED`) | Bytes used/free per volume; purgeable via private CacheDelete | Instant | Volume-level only; space-sharing caveat | None | All | Zero | Hint: cheap bound and consistency check on attributed growth |
| APFS fast directory sizing (dir stats) | Kernel-maintained recursive size/count + gencount per marked directory | Instant | Per sibling verification; Apple’s own DiskSpaceDiagnostics collects it via `APFSIOC_DIR_STATS_OP` | Unprivileged `apfs.util -M` | APFS | Per-op metadata update | Opt-in per-tree (per sibling) |
| APFS per-volume quota/reserve | Volume cap only; no directory quotas; `quota -v` reports none | — | — | root to create volumes | — | — | No |
| Storage Management (System Settings > Storage; `storagekitd`, `diskspaced`, `StorageManagement.Service`, STMUIHelper, category app extensions) | Category sizes: FSEvents streams + its own directory sizing (`_STMSizeDirectoryWithExcludedChildURLs`) + Spotlight queries (Applications extension: “Spotlight is still gathering so returning an unknown total size”) + CacheDelete purgeable | Not measured; secondary sources say “a reasonable period” and approximate results | Depends on Spotlight for some categories | User-level agents + root daemon | Built in | Periodic scans (`com.apple.diskspaced.periodicScan`) | No API; not reusable |
| Spotlight / mds | `kMDItemFSSize` per indexed item | New visible file not queryable after 7 min on the enabled scratch volume (mds_stores busy at 126% CPU); Data volume indexing disabled | Excludes hidden dirs, `.noindex`, much of Library; per-volume opt-out | None | All | Ongoing indexing | Hint only, never a completeness source |
| NSFilePresenter / File Provider | Notifications only for coordinated (NSFileCoordinator) writes | — | Cooperative apps only | None | All | — | No |
| Watchman, fswatch, TM, CCC, SuperDuper | FSEvents consumers; no sizes; TM picks FSEvents, snapshot diff, or deep scan per source | Live | As FSEvents | None | — | Resident daemon (Watchman) | Prior art only |
| Linux: XFS/ext4 project quotas | Kernel-maintained per-directory-tree block and inode usage | Instant | Complete: every allocation by any process or user, open writers included | Setup root (`prjquota` mount, `chattr +P`/`-p`, `xfs_quota project -s`); reading “limited to super-user only” (`quota(1)`) | XFS long-standing; ext4 kernel 4.5 (`project` feature); `PRJQUOTA` in quotactl since 4.1 | Per-allocation counter update | Opt-in precise mode: the only true “as it happens” per-tree accounting found |
| Linux: btrfs qgroups | Per-subvolume referenced/exclusive bytes | Instant | Subvolumes only, not directories | root to enable | btrfs | “affects all extent processing, which takes a performance hit” | Opt-in where subvolumes exist |
| Linux: ZFS `used`/`written`/`written@snap` | Per-dataset, per-snapshot deltas | “Pending changes are generally accounted for within a few seconds” | Datasets only | Reading unprivileged | OpenZFS | Native | Opt-in where datasets exist |
| Linux: inotify | IN_MODIFY per write (identical consecutive events coalesced), IN_CLOSE_WRITE, CREATE, DELETE, MOVE; no sizes; no mmap | Immediate | Per-directory watches; overflow loses events; `max_user_watches` default 1% RAM clamped to [8192, 1048576] since 5.11 (8192 before) | None | All | ~1 KiB per watch | Default live path (already via `notify`) |
| Linux: fanotify | FAN_MODIFY/CLOSE_WRITE/CREATE/DELETE/MOVE; no sizes; events merged | Immediate | `FAN_MARK_FILESYSTEM` (4.20) and `FAN_MARK_MOUNT` need CAP_SYS_ADMIN; unprivileged since 5.13 limited to inode marks, no pid | CAP_SYS_ADMIN for whole-filesystem | 4.20+ / 5.13+ | Queue limits | Opt-in privileged live path |
| Linux: eBPF `filetop`/`biotop` | Per-file/process read and write bytes as they happen (kprobes on `vfs_read`/`vfs_write`) | Immediate | Misses mmap; “can begin to cost measurable overhead at high I/O rates” | CAP_BPF + CAP_PERFMON (5.8+) or CAP_SYS_ADMIN | 4.x+ with bcc | Per-call probe | Opt-in diagnostic |
| Linux: `/proc/<pid>/io` | `wchar`, `write_bytes`, `cancelled_write_bytes` | Instant | Same-user (PTRACE_MODE_READ_FSCREDS); exited processes lost | None | 2.6.20+ | Negligible | Hint: process attribution |

## Direct answers

**Does macOS ship anything that already answers “what grew in the last hour” in
seconds?** No. Nothing on macOS maintains per-directory growth over time.
What exists: FSEvents gives the *names* of paths changed since a cursor (no sizes, cost
scales with volume churn per the sibling measurements); APFS dir stats give a
directory’s *current* recursive size instantly but no history; Storage Management
computes categories with its own walks plus Spotlight and exposes no API; Time Machine’s
snapshot diff is private and Apple itself reportedly moved back to FSEvents because
diffing “isn’t as quick or accurate” (Eclectic Light, secondary).
“What changed” for a short window is answerable in seconds (FSEvents replay + `stat`);
“what grew” requires a baseline the tool keeps.

**Best unprivileged “compute as it happens” architecture on macOS.** fdu-owned
checkpoints (sizes at time T) plus: FSEvents live watch while running and replay while
not, with `stat` for sizes; the libproc open-writer sweep (~15 ms) to re-stat files
FSEvents cannot have reported yet; optional APFS dir-stats gencount on marked roots as a
cheap “anything changed?”
gate; `proc_pid_rusage` sampling for process attribution; `statfs` volume deltas as a
bound ("volume grew 12 GB, attributed 11.5 GB"). This never gets other users’ files,
mmap writes before unmap, or writes by processes that exited between samples; the
fallback is the fast full metadata walk.

**On Linux.** Same checkpoint model with inotify (or unprivileged fanotify inode marks
on 5.13+) live and no journal to replay; `/proc/<pid>/io` for attribution.
Opt-in precise mode: XFS/ext4 project quotas make the kernel maintain exact per-tree
usage on every allocation, including open writers and other users, readable by root;
btrfs qgroups and ZFS `written` where those layouts exist.

**What an opt-in privileged mode adds.** macOS: `eslogger` (root + FDA) gives every
close-with-`modified`, unlink, rename, truncate and clone with a `struct stat` (size at
event), covering other users and catching open writers at close; kdebug gives per-write
byte counts. Neither is API for third parties (ES entitlement is Apple-granted;
`eslogger` output schema is explicitly unstable).
Linux: fanotify filesystem marks, eBPF per-file bytes, project-quota setup.

**Strongest case against the principle.** Every precise as-it-happens source needs root
(and on macOS FDA or an entitlement fdu cannot get), so the default path can only be
approximate anyway. The always-on cost lands on the wrong party: fseventsd replay CPU
scales with whole-volume churn (45 s for a day on the scratch volume), ES and kdebug tax
every process, and a resident observer is one more daemon on an already loaded box.
Observers that are down create gaps that must be detected and answered with a rescan, so
the rescan must be fast regardless; once it is, checkpoints plus a fast walk cover most
questions, and the volume-level `statfs` delta answers “how much” for free.
Even Apple keeps three strategies in `backupd` (FSEvents, snapshot diff, deep scan) with
catch-up variants, which is the same fallback ladder.
Bookkeeping drift (cursor invalidation, UUID changes, open writers, mmap) is a permanent
correctness liability that a stateless scan does not have.

## Findings by facility

### 1. Endpoint Security and `eslogger`

Primary sources: `man eslogger` (this host), SDK headers
`MacOSX26.5.sdk/usr/include/EndpointSecurity/{ESMessage.h,ESMessageCore.h,ESTypes.h,ESClient.h}`,
XNU `security/mac_file.c`, `bsd/kern/sys_generic.c`, `bsd/kern/kern_descrip.c`,
`bsd/vfs/vfs_vnops.c` (apple-oss-distributions/xnu, main).

- `eslogger --list-events` runs unprivileged and lists 104 notify events including
  `close create unlink rename truncate clone copyfile write exchangedata link setattrlist mmap`.
  Running it requires root and TCC Full Disk Access for the responsible process.
  The man page says it “is NOT API in any sense” and “may change from release to release
  without warning”.
- Event payloads (ESMessage.h): `es_event_write_t` has only `target` ("This event type
  does not support caching (notify-only)"), i.e. no byte count or offset.
  Sizes come from `es_file_t.stat` ("stat of file.
  See man 2 stat", ESMessageCore.h). `es_event_close_t` has `modified` ("only reflects
  that a file was or was not modified by filesystem syscall.
  If a file was only modified through a memory mapping this flag will be false") and
  `was_mapped_writable` (message version 6). CREATE, UNLINK, RENAME “can fire multiple
  times for a single syscall”.
- Kernel mechanism: `modified` is `fg_flag & FWASWRITTEN` (mac_file.c
  `mac_file_notify_close`), set when a write syscall returns a positive count
  (sys_generic.c, `os_atomic_or(&fp->fp_glob->fg_flag, FWASWRITTEN, ...)`), delivered on
  the close path (kern_descrip.c `mac_file_notify_close(cred, fp->fp_glob)`). The same
  flag drives FSEvents’ `FSE_CONTENT_MODIFIED` at `vn_close` (vfs_vnops.c), so ES close
  and FSEvents content events share the open-writer blind spot; ES additionally exposes
  `was_mapped_writable`.
- Drops: `seq_num` (msg v2) and `global_seq_num` (msg v4) “can be inspected to detect
  whether the kernel had to drop events for this client” (ESMessage.h).
- Requirements: `es_new_client_result_t` has `ERR_NOT_ENTITLED` ("not properly
  entitled"), `ERR_NOT_PERMITTED` (TCC), `ERR_NOT_PRIVILEGED` ("not running as root"),
  `ERR_TOO_MANY_CLIENTS` (ESTypes.h). Apple’s framework page: entitlement
  `com.apple.developer.endpoint-security.client`, packaged as a system extension, macOS
  10.15+. Muting: `es_mute_path`, `es_mute_path_events` (ESClient.h); WWDC22 session
  110345 describes target-path and inverted muting “to manage the performance impact”.
- Not verified: how often `NOTIFY_WRITE` fires per open descriptor.
  The MAC hook `mac_vnode_check_write` runs on every `vn_write`, so per-call delivery is
  possible; no primary source states coalescing, and I could not run `eslogger`.

### 2. kdebug: `fs_usage`, `ktrace`

`man fs_usage`: “requires root privileges due to the kernel tracing facility”; columns
include `B=x` bytes per call and pathname.
Unprivileged: `fs_usage -t 1` prints “'fs_usage' must be run as root...”; `ktrace info`
fails ("failed to get kernel tracing information"); `dtrace -l` reports SIP and missing
privileges.
Overhead and drop rates are not documented in the man pages; `ktrace` exposes
`-b buffer-size-mb` and ring-buffer mode, which implies loss under pressure.

### 3. Per-process I/O accounting without root (tested)

`src/rusage_sampler.c` samples `proc_listallpids` + `proc_pid_rusage(RUSAGE_INFO_V6)`
for every pid twice, 60 s apart.
Results (`results/rusage_run1.txt`, `run2.txt`):

- Cost: 2.8–19.4 ms per full sample of 259–273 pids (10–72 µs/pid); one 323 ms sample
  during a load spike.
- Permissions: 0 EPERM for same-uid processes (182–214), 100% EPERM for other-uid
  processes (59–83, including root daemons and `kernel_task`). XNU
  `bsd/kern/proc_info.c` `proc_pid_rusage` calls
  `proc_security_policy(..., CHECK_SAME_USER)`, which returns EPERM when uids differ
  unless the caller holds `PRIV_GLOBAL_PROC_INFO`.
- Deltas over 60 s: run 1 `diskio_byteswritten` +22.0 MiB, `logical_writes` +28.0 MiB;
  run 2 +66.2 MiB and +77.5 MiB. Top category both runs: agent CLIs (18.6 and 31.7 MiB
  written), then node.js (20.4 MiB in run 2) and Python tooling.
- Exited processes: 12 and 27 processes disappeared between samples; run 2 lost 2.0 GiB
  of lifetime bytes-written from processes that exited (their post-sample writes are
  unobservable). 15 and 44 new processes appeared; their lifetime counts are
  attributable.
- Semantics (XNU `osfmk/kern/bsd_kern.c`, `thread.c`, `task.c`, `vfs_cluster.c`,
  `spec_vnops.c`):
  `ri_diskio_byteswritten = task_io_stats->total_io.size − disk_reads.size`, accumulated
  in `thread_update_io_stats` from `spec_strategy` for the thread issuing the block I/O
  (metadata and paging included; writeback issued by kernel threads accrues to
  `kernel_task`, which is other-uid).
  `ri_logical_writes = get_task_logical_writes(task, false)`, the internal-storage
  ledger only (external devices go to `logical_writes_to_external`, not exposed in
  `rusage_info_v6`); credited for `TASK_WRITE_IMMEDIATE`, `DEFERRED`, `METADATA` and
  debited for `INVALIDATED`.
- Activity Monitor’s Disk tab: the app imports `sysmon_request_*` (libsysmon) and
  `/usr/libexec/sysmond` imports `proc_pid_rusage`, `coalition_info_resource_usage`,
  `proc_listpids` (`results/system_binaries_evidence.txt`); inference: its per-process
  Bytes Written come from these counters.
  `coalition_info_resource_usage` has no SDK header (private).

### 4. APFS facilities

- `fs_snapshot_create(2)`: “All snapshot functions require superuser privileges and also
  require an additional entitlement.”
  `tmutil`: “Several, but not all, verbs require root and Full Disk Access”.
  No local snapshots exist on this host’s Data volume.
- Snapshot diff: `backupd` contains `TMSnapshotDiffer`, strategies “Using FSEvents”,
  “Using APFS snapshot diffing”, “Using a deep scan”, each with “(catch-up)” variants,
  and imports `fsctl` (private selector); `APFS.framework` exports no diff symbol.
  Eclectic Light (secondary) reports Apple found snapshot diffing “isn’t as quick or
  accurate as using FSEvents”.
- Space accounting: `ATTR_VOL_SPACEUSED` “on space sharing volumes, this value may not
  be identical to the difference between the volume’s size and its” free space
  (`man getattrlist`). Purgeable space is computed by the private CacheDelete framework
  (`CacheDeleteCopyPurgeableSpaceWithInfo`, imported by StorageManagement);
  `diskutil info` printed no purgeable line for the Data volume here.
- Dir stats: Apple’s `DiskSpaceDiagnostics` `FilesystemMetadataSnapshotService` calls
  `APFSIOC_DIR_STATS_OP`, `APFSIOC_PURGEABLE_GET_BULK_INFO`,
  `APFSIOC_CLONEGROUP_ITERATE`, `getattrlistbulk` and `fts_*` to build a
  “SpaceAttribution snapshot” (strings).
  This is Apple’s own disk-space investigation tool using the same primitives fdu would.
- Quotas: `diskutil apfs addVolume -quota/-reserve` are per-volume caps; `quota -v`
  reports none; no directory quota exists.

### 5. Storage Management

Processes and jobs: `/usr/libexec/storagekitd` (root LaunchDaemon; DiskManagement
backend, strings mention purgeable space caching), `diskspaced` (user LaunchAgent inside
StorageManagement.framework; `com.apple.diskspaced.periodicScan`, `.cacheDelete`),
`com.apple.StorageManagement.Service` (CacheDelete purgeable notifications),
STMUIHelper, and per-category app extensions.
`StorageManagement.framework` imports `FSEventStream*`,
`MDItemCreateWithURL`/`MDItemCopyAttribute` with `kMDItemLastUsedDate` etc., and
CacheDelete, and exports `_STMSizeDirectoryWithExcludedChildURLs`; the Applications
extension runs an `NSMetadataQuery`
(`kMDItemContentType='com.apple.application-bundle'`) and logs “Spotlight is still
gathering so returning an unknown total size”.
No public API. Latency not measured.

### 6. Spotlight / mds

`mdutil -a -s`: `/` and Data volume “Indexing disabled”; scratch volume “Indexing
enabled”. Test (`results/spotlight_hidden_test.txt`): four probe files (visible, hidden
dir, `.noindex` dir, `Library` dir) on the enabled volume; none was returned by `mdfind`
(name or content) after 304 s, nor after about 7 minutes, while the same index returned
155 existing `Cargo.toml` hits and `mds_stores` ran at 126% CPU. `mdls` showed
`kMDItemFSSize` for the hidden and `.noindex` files but null for the visible and Library
ones (probably a live-import fallback for items the store will not index; not verified).
`mdimport -t -d1` parses the hidden file fine, so exclusion is policy, not importer
capability. `kMDItemFSSize` is “The size, in bytes, of the file on disk” (Apple docs).
Conclusion: not a seconds-latency source on this host and never a completeness source.

### 7. File Provider, NSFilePresenter, third-party watchers

`NSFilePresenter.h`: `presentedItemDidChange` and `presentedSubitemDidChangeAtURL:` are
invoked by “the file coordination machinery”, i.e. only for coordinated writes.
Watchman (installed here) is a resident FSEvents consumer with clocks; fswatch is a thin
FSEvents CLI (not installed).
Time Machine, CCC and SuperDuper use FSEvents history to narrow enumeration (prior-art
doc). DaisyDisk and iStat Menus: vendor pages not retrievable (404); not verified.

### 8. Linux

- Project quotas: `quotactl(2)` “(since Linux 4.1) PRJQUOTA”; `ext4(5)` `prjquota`
  “requires the project file system feature” (kernel 4.5); `chattr(1)` `P`: “files and
  directories created in the directory will inherit the project id of the directory”,
  renames and hard links across project ids are refused; `xfs_quota(8)` `project -s`
  “sets an inode flag and the project identifier on every file in the affected tree”,
  after which “new files created in the tree will automatically be accounted”;
  `quota(1)`: “viewing of project quota usage and limits is limited to super-user only”.
  That the kernel updates usage on every allocation is inherent to quota enforcement
  (inferred, not quoted).
- btrfs qgroups: per-subvolume referenced/exclusive; “when qgroup mode is activated, it
  affects all extent processing, which takes a performance hit”; `btrfs quota status`
  needs no root.
- ZFS: `written` = “space referenced by this dataset, that was written since the
  previous snapshot”; “Pending changes are generally accounted for within a few
  seconds”.
- inotify(7): IN_MODIFY “(e.g., write(2), truncate(2))”, identical consecutive events
  coalesced, “does not report file accesses and modifications that may occur because of
  mmap(2)”, non-recursive, queue overflow loses events; default `max_user_watches`
  raised from 8192 to 1% of RAM clamped to [8192, 1048576] in 5.11 (LKML patch).
- fanotify: `FAN_MARK_FILESYSTEM` (4.20) and `FAN_MARK_MOUNT` “requires the
  CAP_SYS_ADMIN capability”; since 5.13 unprivileged `fanotify_init` with inode marks
  only and no pid; “consecutive events for the same filesystem object and originating
  from the same process may be merged”.
- eBPF: bcc `filetop` traces `vfs_read()`/`vfs_write()`, misses mmap, “can begin to cost
  measurable overhead at high I/O rates”; `capabilities(7)` CAP_BPF and CAP_PERFMON
  (5.8).
- `/proc/<pid>/io` (proc_pid_io(5)): `wchar`, `write_bytes` ("really sent to the storage
  layer"), `cancelled_write_bytes`; access governed by PTRACE_MODE_READ_FSCREDS.

## Could not verify

- `NOTIFY_WRITE` delivery frequency and ES per-event overhead (no root).
- Storage Management latency and exact category algorithms beyond imported symbols.
- The APFS snapshot-diff `fsctl` selector used by `TMSnapshotDiffer`.
- Whether Foundation’s `volumeAvailableCapacityForImportantUsageKey` includes purgeable
  space (Apple’s reference page is a stub).
- DaisyDisk / iStat Menus behaviour (vendor pages unavailable).
- Spotlight `.noindex`/hidden exclusion on this host: the enabled volume indexed nothing
  new within 7 minutes, so exclusion could not be separated from backlog.

## Sources

Primary, local: `man eslogger`, `man fs_usage`, `man ktrace`, `man fs_snapshot_create`,
`man tmutil`, `man getattrlist`, `man diskutil`, `man quotactl`, `man quota`;
`MacOSX26.5.sdk` EndpointSecurity headers, `sys/resource.h`, `libproc.h`,
`sys/snapshot.h`, `Foundation/NSFilePresenter.h`; binaries inspected read-only with
`dyld_info`, `nm -u`, `strings` (see `results/system_binaries_evidence.txt`). Primary,
XNU (github.com/apple-oss-distributions/xnu, main): `bsd/vfs/vfs_vnops.c`,
`bsd/kern/sys_generic.c`, `bsd/kern/kern_descrip.c`, `security/mac_file.c`,
`security/mac_framework.h`, `bsd/kern/proc_info.c`, `bsd/kern/kern_resource.c`,
`osfmk/kern/bsd_kern.c`, `osfmk/kern/thread.c`, `osfmk/kern/task.c`,
`bsd/vfs/vfs_cluster.c`, `bsd/miscfs/specfs/spec_vnops.c`. Primary, Apple docs:
developer.apple.com/documentation/endpointsecurity;
developer.apple.com/videos/play/wwdc2022/110345/; kMDItemFSSize reference.
Primary, Linux: man7.org quotactl(2), fanotify(7), fanotify_init(2), fanotify_mark(2),
inotify(7), proc_pid_io(5), xfs_quota(8), ext4(5), chattr(1), quota(1), capabilities(7);
btrfs.readthedocs.io btrfs-quota, btrfs-qgroup; openzfs-docs zfsprops(7);
github.com/iovisor/bcc tools/filetop_example.txt; LKML “inotify: Increase default
inotify.max_user_watches limit to 1048576”. Secondary: eclecticlight.co “Time Machine to
APFS: Understanding backups” (2021-03-11) and “What is System Data in Storage Settings?”
(2025-02-11); github.com/redcanaryco/mac-monitor wiki “Endpoint Security Overview”;
watchexec.github.io/docs/inotify-limits.html.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
