# Research: pdu and the Linux Peer Gap

**Date:** 2026-09-28

**Author:** fdu project, with Claude Code

**Status:** Complete for the source study and this session’s measurements.
H162 and H163 are accepted (exp-173, exp-174); H159 is accepted on a directory-dense
real tree (exp-190) after no effect on the sparse kernel tree (exp-188, exp-189); H157
is rejected again (exp-191); H164–H170 are proposed.

## Overview

pdu ([parallel-disk-usage](https://github.com/KSXGitHub/parallel-disk-usage)) is the
fastest Linux peer in fdu’s published comparison: 1.02 s against fdu’s 1.25 s indexed
tree on the million-entry generated tree
([Linux report](../reports/report-2026-09-27-fdu-linux-tool-comparison.md)). No project
document studies it.
The 2026-08-06 survey read its documentation only, and the 2026-08-10 frontier review
read its source for two mechanisms in four bullets.

This brief reads pdu 0.24.0 end to end, sets it beside fdu’s Linux walker mechanism by
mechanism, and measures both on a subject the published comparison lacked: a real source
tree, Linux v6.12, with 358 `.gitignore` files.

The real tree changed the picture.
fdu’s default command took 564 ms there against pdu’s 70 ms; with `--no-gitignore` it
took 76 ms. The gap was `.gitignore` classification on one thread, not the walk.
Two changes built in this session (H162, H163) remove most of it, and the remainder
motivates classifying on the walker threads.
On the generated tree, which has no ignore rules, the remaining index-tier gap is
allocator and scheduling cost, not system calls.

The decisions this supports are which hypotheses run next and in what order, what the
published Linux comparison must add, and which pdu capabilities fdu should adopt.

## Questions to Answer

1. What does the project already record about pdu, diskus, and dumac, and where?
2. How does pdu 0.24.0 walk, fold, and report, and what does each step cost?
3. How does fdu’s Linux walker differ, mechanism by mechanism?
4. Does the published Linux ranking hold on a real tree with ignore rules?
5. Which of pdu’s user-visible capabilities does fdu have, lack, or reject?
6. Which of pdu’s mechanisms transfer, and what should be measured next?

## Scope

**Included:**

- pdu at `c30e46f`, 14 commits past tag `0.24.0` (`4e192606`), Apache-2.0. Since the
  tag, the walker files (`src/tree_builder.rs`, `src/fs_tree_builder.rs`,
  `src/get_size.rs`, `src/hardlink/aware.rs`, `src/data_tree/hardlink.rs`) changed only
  in import style and one doc comment; `src/app/hdd.rs` was refactored for dependency
  injection, and its virtual-disk rule is already present at the tag.
  The measured binary is the released 0.24.0.
- diskus 0.9.0 (`d8a77db`) for contrast, and dumac `1ffbe3c` for one matrix row.
- fdu at `a15b20f4`: this branch, which stacks H159, H162, and H163 on `main`.
- Measurements on a 4-vCPU Intel Xeon at 2.1 GHz, Linux 6.18.44 in a Firecracker KVM
  guest, ext4 on virtio: the host class of the 2026-09-27 report.
  Warm-steady cache, quiet host.

**Evidence labels.** **Harness** rows are quiet paired cells of 12 interleaved pairs
after 3 warm-ups, reported as the median change with a paired-bootstrap 95% interval
under a fixed-N stopping rule, from the perf probe or the tool harness.
**Screen** rows are `hyperfine` means (10–15 runs, or 12 per pass for two passes) on the
same host: good for proportions, not for claims.

**Excluded:** cold cache (the host is virtualized), a macOS re-measure, Windows, content
analysis, and dut, which still has no harness adapter.

## Findings

### A. What the Project Already Records About Its Peers

| Document | pdu | diskus | dumac |
| --- | --- | --- | --- |
| [Survey, 2026-08-06](research-2026-08-06-file-rollup-engine.md) | Named in question 2; one matrix row (`rayon`, `std`, 1 metric, `partial` library); a row in dut’s self-reported table; “consulted via documentation”, not checked out | One matrix row; a row in gdu’s README table; documentation only | Absent |
| [Frontier, 2026-08-10](research-2026-08-10-performance-frontier.md) | Source read at 0.24.0: depth folding (“What the Fastest Walkers Do”, two bullets in “Comparator Refresh”), the one-thread-on-rotational rule; H59 and H60 cite it | The 3×-cores thread rule | A full section, “Healey/Dumac Follow-Up”, about 150 lines; H64 |
| [Cache economics, 2026-09-27](research-2026-09-27-cache-economics-and-default-plans.md) | CPU split rows (macOS, Linux); depth folding “at 3.7 MiB” | CPU split rows | macOS CPU split row |
| [macOS comparison](../reports/report-2026-09-26-fdu-live-tool-comparison.md) | Measured, +49% | Measured | Measured, the next-fastest tool |
| [Linux comparison](../reports/report-2026-09-27-fdu-linux-tool-comparison.md) | Measured, the fastest peer | Measured | macOS-only tool |
| [Peer agreement](../reports/report-2026-09-25-peer-agreement.md) | Accounting: hard links per path, symlink and directory sizes; skips interrupted folders on macOS | Counts hard links once; skips failed folders | Not measured |
| [README “Why”](../../../README.md#why) | Absent from the dozen surveyed tools | Absent | Absent (named in Speed only) |

So pdu, the fastest Linux peer, had no dedicated analysis, with one refinement: its
source was read, but only for depth folding and the thread rule.
Its hard-link design, scheduling, per-entry costs, and user-visible surface were not
studied, and it was never measured on a tree with ignore rules.

Four details surfaced while checking:

- **The harness runs pdu at `--max-depth 1`, which renders the root alone.** pdu counts
  the root as depth 1 (`src/tree_builder.rs:51`, confirmed by running it), so the
  published pdu job is a full scan reduced to one row, not a depth-one chart.
  pdu’s own default is depth 10, which cost 5–14% more on the generated tree and 6% more
  on the kernel tree in this session’s screens.
- **diskus has a library.** The survey matrix says “no”; diskus 0.9.0 exports
  `diskus::DiskUsage` from `src/lib.rs`.
- **diskus’s thread cap is 192, not 64.** The code is
  `3 * available_parallelism().min(64)` (`src/walk.rs`), so it caps the core count, not
  the thread count, although its comment says 64 threads.
  On four vCPUs both readings give 12.
- **dumac sizes its pool to the core count capped at 224** (`MAX_FILE_HANDLES`) at
  `1ffbe3c`; the frontier review’s “64-way bounded concurrency” quotes the article.

### B. How pdu 0.24.0 Works

Paths are relative to the pdu repository at `c30e46f`.

**The walk is one recursive function.** `TreeBuilder::from`
(`src/tree_builder.rs:41–70`) calls `get_info(path)`, then maps the children through
`.into_par_iter().map(TreeBuilder::from)` on rayon’s global pool.
Each child is a rayon task; a worker that runs out of work steals one.
`FsTreeBuilder`’s `get_info` (`src/fs_tree_builder.rs:104–165`) calls
`std::fs::symlink_metadata(path)` on the entry’s full path, which on Linux is one
`statx` against `AT_FDCWD`, so the kernel resolves every path component again for every
entry. For a directory on the same device it then calls `read_dir(path)` and collects
each name as an `OsString`; `join_path` allocates each child’s `PathBuf`. There is no
use of `d_type`: every entry, directories included, is statted by full path.

**Depth folding bounds what is retained, not what is scanned.** Each level decrements
`max_depth`; once it reaches zero, the node returns
`size + children.map(|child| child.size()).sum()` and drops the children
(`src/tree_builder.rs:64–69`). A `DataTree` is `{ name, size, children }`
(`src/data_tree.rs:25–29`), so only nodes within the display depth survive, and deeper
nodes live only as long as their recursive frame.
Every entry below the display depth is still statted.
The root-only harness job peaks at 3.7 MiB on a million entries.
Two parallel post-passes run on the retained nodes only: `par_cull_insignificant_data`
drops descendants below `min_ratio` of the root total (`src/data_tree/retain.rs`), and
`par_sort_by` orders children by size, largest first (`src/data_tree/sort.rs`).
Defaults: `--max-depth 10`, `--min-ratio 0.01`, and on Unix `--quantity block-size`
(`st_blocks × 512`, `src/get_size.rs`; `apparent-size` and `block-count` are the
others).

**Threads: the rayon default, or one on a spinning disk.** `--threads auto`, the
default, builds no custom pool, so rayon uses `available_parallelism`, unless any root
is on an HDD, in which case pdu prints a warning and uses one thread (`src/app.rs`).
`max` skips the HDD check; `N` fixes the count.
Detection (`src/app/hdd.rs`) asks `sysinfo` for the kind of the disk holding the
canonicalized root. On Linux it reclassifies an HDD report as unknown when the block
driver is `virtio_blk`, `xen_blkfront`, `vbd`, `vmw_pvscsi`, or `hv_storvsc`, because
the kernel’s `rotational` flag defaults to 1 for virtual disks; LVM volumes are a
documented gap. On this host `/sys/block/vda/queue/rotational` reads 1 with driver
`virtio_blk`, so without that fix pdu would run single-threaded here, and at two threads
it already takes twice as long (section D.3).

**Hard links cost one branch per entry unless a file has several.** Without `-H`, pdu
counts a hard-linked file once per path, as fdu does.
With `-H`, `record_hardlinks` (`src/hardlink/aware.rs`) returns early for directories
and for `nlink <= 1`; only a file with several links is inserted into a `DashMap` keyed
by `(ino, dev)` holding its size, link count, and paths
(`src/hardlink/hardlink_list.rs`), and a size or link-count conflict is an error.
After the walk, `par_deduplicate_hardlinks` (`src/data_tree/hardlink.rs`) visits the
retained nodes: for each recorded inode it keeps the link paths under the node’s prefix
and subtracts `size × (n − 1)` when `n ≥ 2` of them are inside.
JSON output carries the links as `shared.details` and `shared.summary`. With several
roots, overlapping paths are removed first so no link is counted twice
(`src/app/overlapping_arguments.rs`).

**The rest of the surface.** Several roots are walked one after another and joined under
a synthetic root that is renamed `(total)` (`src/app/sub.rs`). `--json-output` writes a
schema-versioned tree, converting names with `par_convert_names_to_utf8().expect(…)`, so
a name that is not UTF-8 aborts JSON output.
`--json-input` re-renders a saved tree with the depth, ratio, and sort options applied.
The default `ErrorOnlyReporter` ignores every event except errors, and reporters are
selected by const generics, so the default path has no per-entry dynamic dispatch
(`src/app.rs`, “DYNAMIC DISPATCH POLICY”). `--progress` switches to a reporter that does
an atomic `fetch_add` per event and redraws from its own thread every 100 ms, “at the
expense of performance”.
Separate binaries write shell completions (bash, zsh, fish, elvish, PowerShell) and a
man page. The library crate, `parallel_disk_usage`, exposes `FsTreeBuilder`, `DataTree`,
and the visualizer.

**diskus 0.9.0, for contrast** (`src/walk.rs`): a rayon pool of `3 × min(cores, 64)`
threads recurses over `PathBuf`s, calls `symlink_metadata` per entry, collects each
child’s full path from `read_dir`, and sends one crossbeam message per counted entry to
a receiver thread that sums sizes and keeps a `HashSet` of `(dev, ino)` for files with
`nlink > 1`. On the kernel tree it made 12,495 `sched_yield` calls, 12 threads on 4
vCPUs.
It still matches pdu: 1.034 s against 1.018 s in the 2026-09-28 harness, 0.74 s of
user CPU against 0.69 s. A cross-thread message per entry is cheap when the receiver
does constant work.

### C. How fdu Walks on Linux

A source study of this branch, set against pdu.

| Mechanism | pdu | fdu |
| --- | --- | --- |
| Threads | rayon global pool: 4 on 4 cores | `automatic_worker_pool`: `min(cores, 6)` = 4 walkers, up to `min(2 × cores, 16)` = 8 after an adaptive unlock at 30 µs per entry, which a warm Linux scan at 1.5–2 µs never reaches (H84); plus the calling thread as the one consumer ([scan.rs](../../../crates/fdu-core/src/scan.rs), `PORTABLE` in [platform_tuning.rs](../../../crates/fdu-core/src/platform_tuning.rs)) |
| Work distribution | Work stealing | One `Mutex<DirectoryQueueState>` and `Condvar`; per-region LIFO buckets served round-robin; claims of up to four directories (`DIR_CLAIM`); `notify_all` after every `extend` (`DirectoryQueue`, scan.rs) |
| Walker to consumer | None: results return up the recursion | Unbounded `std::sync::mpsc` channel; drained listings go back to their walker (H159) |
| Directory open | `read_dir` of the full path | `fs::read_dir(root.join(rel_dir))`: also a full path |
| Entry metadata | `statx(AT_FDCWD, full path)` | `DirEntry::metadata()`: `statx` relative to the open directory; the transient summary skips directories and symlinks by `d_type` (H72); the index stats every entry |
| Per file (fdu: index build) | Name, path, and a node dropped past the display depth | An owned extension `String` from `classify::ext_bucket` before interning, and a one-entry per-extension contribution merged into the parent roll-up ([index.rs](../../../crates/fdu-core/src/index.rs) `DetachedIndexBuilder`, [classify.rs](../../../crates/fdu-core/src/classify.rs)) |
| Per directory (fdu: index build) | A `Vec` of names | A `Box<DirectoryEntry>`, a child `Vec<EntryId>`, and a joined path keyed into `HashMap<PathBuf, EntryId>` under SipHash |
| Allocations per million entries | Not measured | 7.03M before H157’s rejected cut (exp-161) |
| Allocator | glibc | glibc (`std::alloc::System`) behind a counting wrapper ([main.rs](../../../crates/fdu/src/main.rs)) |
| `.gitignore` | Not read | Read by walkers, delivered ahead of each directory’s entries, classified on the consumer for both the index and the transient summary ([execution.rs](../../../crates/fdu-core/src/execution.rs) `SummaryFold`) |

The CPU profile of the 2026-09-28 Linux matrices, as medians over each tool’s timed
samples (84 for fdu, 12 per peer) computed from the committed
[indexed](../reports/fdu-linux-tool-comparison-result-2026-09-28-indexed.json) and
[summary](../reports/fdu-linux-tool-comparison-result-2026-09-28-summary.json.gz)
results (**harness**, balanced tree):

| Tool and job | Wall | User CPU | Kernel CPU | Voluntary switches | Involuntary switches | Minor faults |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| fdu indexed tree | 1.25 s | 1.17 s | 3.03 s | 14,668 | 3,803 | 81,124 |
| fdu summary, `--no-gitignore` | 0.94 s | 0.87 s | 2.69 s | 17,118 | 3,753 | 2,911 |
| pdu `--max-depth 1` | 1.02 s | 0.69 s | 3.30 s | 17 | 86 | 230 |
| diskus | 1.03 s | 0.74 s | 3.35 s | 161 | 2,386 | 1,038 |

fdu spends less kernel time than pdu: 0.28 s less for the indexed tree, which issues the
same number of `statx` calls, and 0.61 s less for the summary, which also skips the
directory `statx` calls.
It spends more user time, 0.48 s more for the index and 0.18 s for the summary, and its
threads block: 14,668 voluntary switches to pdu’s 17.

### D. New Measurements

| Subject | Entries | Directories | Files | Max depth | `.gitignore` files | Rules |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `linux-balanced-1m` (generated, digest `4bbd97c0`) | 1,000,001 | 125,001 | 875,000 | 7 | 0 | 0 |
| `linux-v6.12` (torvalds/linux tag v6.12, `adc21867`) | 92,474 | 5,769 | 86,643 | 14 | 358 | 1,593 |

Binaries: fdu `main` (`45c7c577`, `0d73ed54`), the H159 layer (`aa58a6b1`), H162
(`892cef40`), H163 (`a15b20f4`), and the pre-H161 control `a5c0ab46`; pdu 0.24.0 and
diskus 0.9.0 from `cargo install --locked`.

#### D.1 On a real source tree, classification was the gap

**Screen**, 15 runs, command line, means, `linux-v6.12`, fdu before H162 (`main` and the
H159 layer do not differ on this tree, exp-188):

| Command | Wall | User CPU | Kernel CPU |
| --- | ---: | ---: | ---: |
| `fdu PATH` (default tree) | 564.5 ms | 562 ms | 191 ms |
| `fdu PATH --no-gitignore` | 76.3 ms | 74 ms | 174 ms |
| `fdu PATH --view summary` | 490.2 ms | 548 ms | 171 ms |
| `fdu PATH --view summary --no-gitignore` | 64.9 ms | 63 ms | 169 ms |
| pdu, default depth 10 | 69.7 ms | 55 ms | 199 ms |
| pdu `--max-depth 1` | 65.9 ms | 50 ms | 195 ms |
| diskus | 74.5 ms | 62 ms | 222 ms |

With ignore rules off, fdu is within 10% of pdu and diskus, so the walk is not the gap.
With them on, user time nearly equals wall time: one thread does the extra work.
fdu’s allocation counter read 7,216,227 allocations with controls against 388,225
without, 78 per entry against 4.2. Before H162, `Gitignore::matches` collected the
entry’s relative path components into a `Vec` for every control file on its path, and
`segment_path_matches` allocated a `Vec<bool>` row per call plus a fresh one per pattern
segment for every anchored or path pattern
([gitignore.rs](../../../crates/fdu-core/src/control/gitignore.rs), diff of `892cef40`).
`ControlMatcher::is_ignored` also looked up every ancestor of every entry in a
`BTreeMap<PathBuf>` of control files
([control.rs](../../../crates/fdu-core/src/control.rs)). All of it ran on the consumer.

The published comparison could not see this: its tree holds no `.gitignore`, and its
summary contract ran with `--no-gitignore`.

**H162** (`892cef40`) matches without allocating: path components go to a 32-slot stack
buffer, a pattern without `**` matches only a path of its own length, segment for
segment, and `**` patterns keep the same dynamic program on 64-slot stack rows.
The commit’s screen counted 7.2M allocations falling to 402k. Quiet cell exp-173
(**harness**, probe, control the H159 layer):

| Job, `linux-v6.12` | H159 layer | H162 | Change [95% interval] | Placebo, both arms `--no-controls` |
| --- | ---: | ---: | --- | --- |
| `aggregate-summary` | 504.9 ms | 260.2 ms | −47.02% [−52.43%, −44.35%] | +0.21% [−6.89%, +2.89%] |
| `default-tree` | 590.1 ms | 320.3 ms | −46.19% [−48.68%, −44.96%] | +0.55% [−6.00%, +2.97%] |

User CPU fell 45.14% and 41.85%; kernel CPU did not move.

**H163** (`a15b20f4`) resolves each listing’s governing controls once:
`ControlTable::chain_for` collects them for a directory, and `ControlChain::is_ignored`
matches each child by slicing the directory’s components plus the child’s name.
The index builder resolves one chain per listing; the summary fold caches it for the
last parent. **Screen** against H162: summary 264 → 149 ms, tree 324 → 193 ms.

Quiet cell exp-174 (**harness**, probe, control H162):

| Job, `linux-v6.12` | H162 | H163 | Change [95% interval] | Placebo, both arms `--no-controls` |
| --- | ---: | ---: | --- | --- |
| `aggregate-summary` | 260.7 ms | 166.5 ms | −36.43% [−38.58%, −28.98%] | −2.17% [−7.51%, +3.65%] |
| `default-tree` | 312.2 ms | 211.3 ms | −35.86% [−39.14%, −21.50%] | −4.06% [−8.84%, +0.26%] |

On `linux-balanced-1m`, which holds no `.gitignore`, H162 and H163 together moved
neither job: `aggregate-summary` +1.34% [−4.64%, +3.47%] and `default-tree` +0.78%
[−2.27%, +2.00%].

After both changes the probe’s default tree takes 211 ms against a controls-off floor of
81 ms (exp-174’s `--no-controls` arms) and pdu’s 70 ms, and the summary 167 ms against
71 ms.
About 130 ms of the tree and 95 ms of the summary remain above the floor, and that
remainder is classification on the one consumer thread (H164).

H159 ran into the same cost on this tree.
Its deciding cell, exp-188, measured `default-tree` −2.19% [−4.50%, +1.08%] and was
rejected, because on the H159 layer that job was 86% classification: 590 ms with
controls against 82 ms without (exp-173). Re-run on top of H162 and H163 (exp-189,
control `main` with both cherry-picked), it was rejected again: `default-tree` +2.26%
[−5.33%, +12.96%], and `cold-scan-index` +3.22% [+1.21%, +12.87%], a regression
interval. The recycle pays on the directory-dense generated tree (eight entries per
directory) and not on this source tree (16 per directory).
Its saving is per directory, about 1.25 µs each, so a fair real deciding subject must be
directory-dense: on a `node_modules` tree of 79,953 entries in 9,439 directories
(exp-190, pre-registered for the purpose) it was accepted, `default-tree` −8.61%
[−19.47%, −5.04%] and `cold-scan-index` −6.79%.

#### D.2 On the generated tree, the gap is allocation and scheduling

**Screen**, two passes of 12 runs, means, `linux-balanced-1m`:

| Command | Pass 1 | Pass 2 |
| --- | ---: | ---: |
| `main` indexed tree (`--cache off --depth 1 --limit 10`) | 1.61 s | 1.55 s |
| H159 indexed tree | 1.39 s | 1.34 s |
| `main` summary `--no-gitignore` | 1.18 s | 1.17 s |
| H159 summary `--no-gitignore` | 1.17 s | 1.20 s |
| H159 default summary | 1.19 s | 1.22 s |
| H159 default tree | 1.39 s | 1.35 s |
| pdu `--max-depth 1` | 1.31 s | 1.29 s |
| pdu, default depth 10 | 1.38 s | 1.48 s |
| diskus | 1.39 s | 1.40 s |

The host ran slower this session than on the harness day (pdu 1.29–1.48 s against 1.02
s), so only proportions carry.
In this screen, H159’s default tree is level with pdu at pdu’s own default depth.

**System calls are the same.** `strace -c` on `linux-v6.12` (the balanced tree was not
traced):

| Tool and job | `statx` | `getdents64` | `openat` | `fstat` | `futex` | `sched_yield` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| fdu indexed tree (H159 layer) | 92,836 | 11,540 | 6,134 | 5,773 | 3,854 | — |
| fdu default tree (H159 layer) | 92,836 | 11,540 | 6,134 | 5,773 | 3,738 | — |
| fdu summary `--no-gitignore` | 86,648 | 11,540 | 5,777 | 5,773 | 3,849 | — |
| pdu `--max-depth 1` | 92,484 | 11,540 | 5,783 | 5,773 | 9 | 129 |
| diskus | 92,478 | 11,540 | 5,778 | 5,773 | 348 | 12,495 |

Every tool makes one `statx` per entry, two `getdents64` and one `openat` per directory,
and one `fstat` per directory from glibc’s `opendir`. fdu’s roughly 350 extra `openat`
and `statx` calls read the 358 `.gitignore` files; its summary skips directories and
symlinks. The one large difference is `futex`: fdu 3,738–3,854, pdu 9.

**The allocator is.** **Screen**, 10 runs, means, the unchanged binaries under
`LD_PRELOAD` jemalloc:

| Build and job | glibc | jemalloc | Change | User CPU, glibc → jemalloc |
| --- | ---: | ---: | ---: | --- |
| `main` indexed tree | 1.673 s | 1.314 s | −21% | 1.31 → 0.83 s |
| H159 indexed tree | 1.477 s | 1.294 s | −12% | 1.12 → 0.81 s |
| H159 summary `--no-gitignore` | 1.205 s | 1.006 s | −17% | 0.86 → 0.60 s |
| pdu `--max-depth 1` | 1.379 s | 1.369 s | −1% | 0.56 → 0.53 s |

`glibc.malloc.arena_max=2` made H159’s indexed tree 2.511 s (+70%), and `arena_max=8`
1.420 s. pdu does not care which allocator runs, because it allocates and frees within
one recursive frame, mostly on one thread; fdu’s walkers allocate what its consumer
frees. H159 recovered about half of `main`’s allocator-attributable time (0.36 s under
jemalloc for `main`, 0.18 s for H159), and about 0.2 s remains even in the summary,
which builds no index.

**H159’s cell** (exp-188, **harness**, control `main` `0d73ed54`):

| Subject and job | Change [95% interval] | Note |
| --- | --- | --- |
| balanced, `default-tree` (screening) | −10.63% [−13.00%, −8.23%] | involuntary switches −24.53% |
| balanced, `cold-scan-index` | −7.23% [−8.27%, −5.05%] | peak RSS −4.95% and −5.71% |
| `linux-v6.12`, `default-tree` (deciding) | −2.19% [−4.50%, +1.08%] | **Rejected**; 86% classification (D.1) |
| `linux-v6.12`, `cold-scan-index` | +5.27% [−4.28%, +8.81%] | peak RSS +0.33% and +1.43% |
| Tool harness, product `fdu-default-tree`, balanced | `main` +10% [+8%, +15%] against H159 |  |
| Tool harness, product `fdu-default-tree`, `linux-v6.12` | `main` +2% [−6%, +6%] against H159 |  |

#### D.3 Thread sweep: fdu’s walkers wait

**Screen**, 10 runs, means, `linux-balanced-1m`; fdu through the perf probe with
`--threads N`:

| Threads | pdu | fdu `aggregate-summary --no-controls` | fdu `default-tree` |
| ---: | ---: | ---: | ---: |
| 2 | 2.743 s | — | — |
| 3 | — | 1.536 s | 1.768 s |
| 4 | 1.359 s | 1.238 s | 1.417 s |
| 6 | — | 1.163 s | 1.395 s |
| 8 | 1.390 s | 1.126 s | 1.275 s |

At four threads pdu used 0.52 s of user and 4.80 s of kernel CPU, 3.9 cores busy; fdu’s
summary 0.89 s and 3.70 s, 3.7 cores; fdu’s tree 1.07 s and 4.04 s, 3.6 cores.

pdu halves from two threads to four and gains nothing at eight: it is kernel-bound at
four cores. fdu gains 9–10% from four walkers to eight on a four-vCPU guest.
Oversubscription helps only a pool whose threads sit idle, and fdu’s do: walkers park on
the queue’s condition variable, which wakes all of them on every `extend`, and the
consumer parks on the channel.
pdu’s rayon workers steal instead of parking.

At equal `statx` counts fdu’s kernel time is lower: 0.28 s less for the indexed tree in
the harness, 0.76 s less for the tree at four threads in this screen.
Both tools open directories by full path, so the difference is in `statx`: fdu’s is
relative to the open directory, pdu’s resolves the whole path.
That attribution follows from the mechanism; no experiment isolated it.
If fdu’s user CPU matched pdu’s while keeping its own kernel time and cores equally
busy, it would use 7–21% less CPU than pdu: 3.71 against 3.99 CPU-seconds on the harness
indexed figures, 4.56 against 5.32 for the screened tree, and 4.22 against 5.32 for the
screened summary.

#### D.4 H161 on Linux

H161 (`fdu-1ovb`, [#149](https://github.com/jlevy/fdu/pull/149)) lets the default
summary classify ignored entries without retaining an index.
Its Linux cell, exp-187 (**harness**, control `a5c0ab46`, candidate `0d73ed54`), ran
against the pre-registered bars:

| Subject and measure | Result | Pre-registered bar |
| --- | --- | --- |
| `linux-v6.12`, `aggregate-summary` wall | −6.93% [−11.29%, −0.17%] | −3% with the interval below zero: **met** |
| `linux-v6.12`, peak RSS | −22.86%, 35.8 → 27.6 MiB | −50%: **missed** |
| `linux-v6.12`, placebo, both arms `--no-controls` | +0.95% [−8.52%, +4.21%] | includes zero: met |
| `linux-v6.12`, placebo, `default-tree` | +0.85% [−8.62%, +10.94%] | includes zero: met |
| balanced, `aggregate-summary` wall (screening) | −19.27% [−22.68%, −17.34%] |  |
| balanced, peak RSS (C launcher, median of 5) | 314.2 → 8.6 MiB, −97% |  |
| balanced, placebo, both arms `--no-controls` | +0.56% [−0.67%, +3.83%] |  |

On the kernel tree the classifying summary peaks at 27.6 MiB against 7.6 MiB with
controls off (C launcher: 28,228 against 7,736 KiB), so observing `.gitignore` costs
about 20 MiB there. What holds that memory was not attributed.
H161 is accepted on macOS on peak RSS (exp-170, exp-171); on Linux it met its wall bar
and missed its RSS bar, and how to record that is the registry’s decision.

### E. Feature Parity, pdu to fdu

fdu features were checked in [the usage guide](../../usage.md) and
[cli.rs](../../../crates/fdu/src/cli.rs).

| pdu capability | fdu | Status | Note |
| --- | --- | --- | --- |
| Size tree with bars (default output) | `fdu PATH`, Tree format | Has | Share bar, root percentage, size; `--bar-size` sets the bar width |
| `--max-depth N` (default 10) | `--depth N` (default 5) | Has | pdu’s depth 1 is the root, fdu’s is depth 0, so pdu `N` is fdu `N − 1`; both bound display only, and fdu’s `--scan-depth` also bounds discovery |
| `--min-ratio F` (default 0.01) | `--min-share P%` (default 1%) | Has | Both compare against the root total |
| `--no-sort` | `--sort size\|count\|mtime\|name\|metric`, `--reverse` | Not planned | No filesystem order: deterministic output is deliberate |
| `--quantity apparent-size\|block-size\|block-count` | `--size apparent\|allocated` | Partial | No block count. pdu adds directory and symlink sizes; fdu counts regular files only |
| `--bytes-format plain\|metric\|binary` | Binary units in text; exact integers in JSON, JSONL, YAML | Partial | No switch for text units |
| `--json-output` | `--format json\|jsonl\|yaml` | Has, richer | Versioned schemas, completeness, errors, native path identity; pdu requires UTF-8 names |
| `--json-input` (render a saved tree) | None | Gap | Nearest: `--cache on`, then `--stale-ok` answers from fdu’s own snapshot |
| `-H` hard-link deduplication | Counts per path | Gap, planned | Unique allocated bytes are designed in [the checkpoint plan](../specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md); `nlink` is not retained today |
| `-x` one file system | `--one-filesystem`; `one_filesystem` in Rust and Python | Has | Part of the scan scope and cache identity; refused where the platform has no device identity, and with `--watch` |
| `--threads auto\|max\|N` | Rust `query::Workers::scan`; the CLI’s `--workers` bounds content analysis only | Partial | No scan-thread control on the command line or in Python |
| One thread on an HDD | None | Not planned | The adaptive pool measures chunk service time rather than classifying the device |
| `--progress` (slower) | `--progress auto\|always\|never`, drawn at an interactive terminal | Has |  |
| `--silent-errors` | `-q` hides notes, tips, and progress, and keeps warnings and errors | Partial, by design | A partial result must say so; exit status 2, or 0 with `--allow-partial` |
| Several roots under `(total)` | One `PATH` | Gap |  |
| Bottom-up default, `--top-down`, `--align-right`, `--total-width`, `--column-width` | Top-down only; `--bar-size` | Partial | No plan recorded |
| Shell completions, man page | None; `--docs` and `--skill` print the guide | Gap |  |
| Library crate `parallel_disk_usage` | `fdu-core` in Rust and the `fdu` Python package | Has, richer | Retained index, `open`, watch, queries, views |

fdu has much that pdu lacks: ignore classification with ignored shares, grouped views,
filters, content analysis, a cache, watch, and completeness provenance.
The design principles’
[“Subsume the Neighbours”](../architecture/fdu-design-principles.md#subsume-the-neighbours)
table maps `du -sh` and diskus to `fdu --view summary`, but does not list pdu, whose
nearest equivalent is `fdu PATH --depth 9`.

## Key Insights

- **A tree without ignore rules measured the walk and hid the default command’s cost.**
  On Linux v6.12 the default `fdu PATH` took eight times pdu’s time; with
  `--no-gitignore` it came within 10%. Real subjects belong in the published comparison.
- **The expensive work ran on the one thread that cannot scale.** Classification sat on
  the consumer, so neither more walkers (H84) nor a better walk could help it.
  H162 and H163 made it cheaper; H164 would make it parallel.
- **On the generated tree, fdu’s system calls are no worse than pdu’s, and its kernel
  time is lower.** It loses in user space: an allocator its producer and consumer share,
  and walkers that block where pdu’s steal.
- **A result on one subject can vanish on another.** H159 cut the balanced default tree
  10.63% and did nothing measurable on the kernel tree, where classification dominated.
  The deciding subject has to be one whose cost the change targets.
- **pdu’s speed comes from doing less, not from a better walk.** It does not read ignore
  rules, retains only what it prints, records hard links only for files with several
  links, and reports nothing per entry by default.
- **pdu’s published job is smaller than its name suggests.** At `--max-depth 1` it
  renders the root alone.

## What Transfers, and What Does Not

**Transfers:**

- **Retain only what the answer needs.** pdu’s depth folding is H66 (`fdu-sk7v`), the
  exact transient tree.
  H59 confirmed the mechanism (RSS −95.28%, wall topology- sensitive), and H161 already
  applies it to the summary.
  For fdu it must be a planner tier that falls closed and never lets output depth prune
  the scan.
- **Work stealing instead of a locked queue.** The 14,668-to-17 voluntary-switch gap is
  the scheduling part of the user-space gap (H166).
- **No per-entry work the answer does not use.** pdu’s default reporter does nothing for
  any event but an error, and its hard-link record is sparse; fdu’s per-file extension
  `String` and per-directory path key are the counterparts to remove (H167, H168).
- **Sparse hard-link recording.** When fdu builds unique allocated bytes, recording only
  files with `nlink > 1` keeps the common case at one comparison; that needs `nlink`,
  which `statx` already returns but `Attrs` does not keep.
- **Several roots in one run**, with overlapping roots removed, is a small surface gap.

**Does not transfer:**

- **Full-path `statx`.** fdu’s directory-relative call is already cheaper.
- **Unsorted output.** It conflicts with deterministic output.
- **A static device rule.** pdu’s HDD check needed a driver list to stop misreading
  virtual disks, this host among them; measuring service time, as fdu’s adaptive pool
  does, needs no list.
  That is
  [“Derive the Choice; Do Not Inherit It”](../architecture/fdu-design-principles.md#derive-the-choice-do-not-inherit-it).
- **pdu’s accounting as an oracle.** It adds directory and symlink sizes and aborts JSON
  output on non-UTF-8 names; its totals calibrate throughput only.
- **Its allocator neutrality as evidence that fdu needs no change.** pdu is neutral
  because it never frees on another thread; fdu’s fix is structural.

## Proposed Hypotheses (H162–H170)

This session holds H162–H170 and exp-173–186 (`fdu-92hp`,
[runbook](../guides/performance-loop-runbook.md)). Every proposal is judged by
[the accept rule](../guides/performance-loop.md#the-accept-rule): a median at least 3%
faster with the 95% interval entirely below zero, and peak RSS non-inferior unless
stated otherwise.

**H162, built** (`892cef40`). Match `.gitignore` rules without allocating.
Accepted in exp-173: `aggregate-summary` −47.02%, `default-tree` −46.19% on
`linux-v6.12`, both `--no-controls` placebos including zero.

**H163, built** (`a15b20f4`). Resolve a listing’s governing controls once.
Accepted in exp-174: `aggregate-summary` −36.43%, `default-tree` −35.86% on
`linux-v6.12`, the `--no-controls` placebo including zero.

**H164: classify on walker threads.** *Mechanism:* each queued directory carries its
governing `ControlChain` (an `Arc`, plus its own `.gitignore` once read); the walker
classifies each child before sending it, and children of an ignored directory inherit
the flag.
Classification stays a pure function of the chain, so workers still never touch
the index. *Expected:* the roughly 110 ms (tree) and 80 ms (summary) above the
controls-off floor on `linux-v6.12`, now spread over four walkers; the tree toward 110
ms. Estimate. *Pre-registered:* `default-tree` and `aggregate-summary`, controls on,
`linux-v6.12` deciding; placebos with both arms `--no-controls` include zero;
balanced-1M non-inferior.
*Risk:* which files the control budget refuses depends on the order controls are applied
(`SummaryFold` documents this); a walker-side table must reach an order the index could
also meet, and the refusal decision must stay whole-tree.
Watch and reconcile keep the per-entry matcher.

**H165: revisit the Linux walker count.** *Mechanism:* H84 found `--threads 8` regressed
the controls-on default summary on nominated `/usr` (+7.12% quiet) while `--no-controls`
improved (−10.06%): extra walkers only fed a consumer that was already the bottleneck.
With H162–H164 that consumer is cheaper, and D.3 shows eight walkers 9–10% faster on a
tree without ignore rules.
*Expected:* 5–10% on large trees if the knee sits above four; nothing if H166 removes
the idle time first.
*Pre-registered:* fixed 4 against 8 via the probe’s `--threads`, then the shipped
`PORTABLE` constant; `default-tree` and `aggregate-summary`, controls on; `linux-v6.12`
deciding, nominated `/usr` non-inferior with the upper bound under +3%, balanced-1M
screening; minor faults non-inferior.
*Risk:* oversubscription hides blocking instead of removing it; one 4-vCPU guest is one
regime, so the constant stays `inherited` elsewhere until bare metal is measured
(`fdu-tk1b`). Do not lower the unlock threshold (H84).

**H166: a queue that does not park walkers.** *Mechanism:* replace
`Mutex<DirectoryQueueState>`, `Condvar`, and `notify_all` with per-walker deques plus an
injector (dua’s shape), or a single-CAS batch push that wakes
`min(new directories, parked)` walkers (dut’s shape).
A first step is waking only as many walkers as directories were pushed.
*Expected:* voluntary switches down by an order of magnitude; wall down by part of the
9–10% that oversubscription recovers today.
*Pre-registered:* `cold-scan-index` on balanced-1M deciding (no classification there);
`default-tree` on `linux-v6.12` non-inferior; voluntary switches down at least 50%.
*Risk:* the region round-robin and parent-first delivery are deliberate properties that
progressive consumers need; a work-stealing order must preserve both.
A new dependency needs the supply-chain review.

**H167: walker-assigned directory tokens.** *Mechanism:* a walker gives each directory
it queues a dense token from an atomic counter; its listing carries the parent’s token,
and the builder keeps a `Vec<EntryId>` indexed by token instead of
`HashMap<PathBuf, EntryId>`, removing a path join, two SipHash lookups, and a path key
per directory from the consumer.
*Expected:* 125k fewer consumer allocations on balanced-1M and one of the free sites
H159’s context-switch profile named.
*Pre-registered:* `cold-scan-index` on balanced-1M deciding, allocations down at least
the directory count; `default-tree` on `linux-v6.12` non-inferior.
*Risk:* walkers still need paths to open directories until H169; repeated names and the
serial walker’s path identity must keep their behavior.

**H168: intern extensions without a per-file `String`.** *Mechanism:* lowercase the
extension into a stack buffer and look it up by bytes, or derive the extension id on the
walker. *Expected:* 875k fewer allocations on balanced-1M, on the consumer’s own thread;
wall likely under 3% alone, since H157 removed 2.75M allocations for −2.22%.
*Pre-registered:* bundled with H167 as one change against `cold-scan-index` on
balanced-1M, or kept alone only on an allocation-count target declared beforehand.
*Risk:* small effect; a non-UTF-8 extension must still bucket as it does today.

**H169: open relative to the parent, read `getdents64` into a reused buffer.**
*Mechanism:* open each directory with
`openat(parent_fd, name, O_DIRECTORY | O_NOFOLLOW)`, read `getdents64` into a per-walker
buffer, and take the directory’s own metadata from `fstat` on the opened descriptor.
For the index, glibc’s `opendir` `fstat` and the parent-side `statx` collapse into one
call; the summary, which already skips directory `statx`, drops glibc’s `fstat`. Each
open resolves one component instead of a path.
*Expected:* 125k fewer system calls on balanced-1M (one per directory), less kernel path
resolution per open, and no per-directory `PathBuf` or `DIR` buffer.
*Pre-registered:* `cold-scan-index` and `aggregate-summary --no-controls` on balanced-1M
deciding, system calls down at least 125k under `strace -c`; `linux-v6.12` non-inferior.
*Risk:* descriptors must be bounded across a breadth-first queue; the code is a native
boundary behind `cfg(target_os = "linux")` and needs `make cross-lint` and an audit; the
portable reader stays the reference.

**H170: fold the transient summary on each walker.** *Mechanism:* `SummaryFold` folds an
`Op::Upsert` carrying an owned path per entry on the consumer.
The tallies are commutative, so each walker can fold counts, bytes, and the newest mtime
locally and send one partial per chunk; the ignored share needs H164. *Expected:* most
of the 0.2 s the jemalloc screen attributes to allocation in the summary; user CPU
toward pdu’s. *Pre-registered:* `aggregate-summary --no-controls` on balanced-1M
deciding, user CPU down at least 20%; after H164, controls on at `linux-v6.12`. *Risk:*
the transient summary must stay equal to the indexed answer under the existing
differential test, including errors at paths, partial provenance, and progress counts.
H85 recovered only part of this cost by recycling buffers; removing the messages is the
larger lever.

**The allocator question.** H74 and H85 declined an allocator dependency: H74’s mimalloc
build won 23% on the aggregate tier but raised its peak RSS 139%, and the dependency
builds C code. This screen reopens it with new numbers (jemalloc: `main` indexed −21%,
H159 −12%, summary −17%, pdu −1%). The structural route (H159, H167, H170) fits the
rules better, and whether to accept a dependency is the maintainer’s decision.

## Recommendations

1. **H159 is decided** (updated 2026-09-29): no effect on the sparse `linux-v6.12`
   (exp-188, exp-189), accepted on the directory-dense `node-modules-dense` (exp-190,
   −8.61%); #150 merged into 0.2.1.
2. **Build H171 next, then H164 if still needed** (updated 2026-09-29). The
   [design study](research-2026-09-29-linux-default-tree-point-solution.md) found the
   remaining default-path cost is a linear scan that bucketed matching (H171) removes;
   H164 then targets the residual, and H165 and H170 still depend on it.
3. **Run H166 and H167 as the index-tier pair** for the generated tree: they address the
   scheduling part and the cross-thread-free part of the user-space gap.
4. **Treat H169 as the Linux-native lever** after those, since it touches a native
   boundary.
5. **Add `linux-v6.12` to the published Linux comparison**, measure the default
   invocations with ignore rules on, and state pdu’s depth: `--max-depth 1` is a total.
6. **Add pdu, diskus, and dumac to README “Why” and to “Subsume the Neighbours”**, and
   correct the survey’s diskus library entry (done in the matrix note).
7. **Consider three surface gaps**: several roots, scan threads on the command line and
   in Python (the Rust `Workers::scan` exists, so it would be presentation only), and
   shell completions. Unique allocated bytes are already planned.

## Next Steps

- [x] Record exp-174 (H163) in this brief and the ledger
- [x] Re-run H159’s deciding cell on the H162 + H163 stack (exp-189, rejected)
- [x] Register H164–H170 in
  [the hypothesis registry](../guides/performance-loop.md#hypotheses), with beads
- [ ] Attribute the 20 MiB the classifying summary holds on `linux-v6.12`
- [x] Decide how H161’s Linux result is recorded (wall met, RSS missed):
  [H161’s registry row](../guides/performance-loop.md#current-engine-010) records it as
  accepted on wall, one of its two Linux bars; whether the RSS bar binds on Linux is
  left to the maintainer
- [ ] Put the allocator question to the maintainer

## Methodology

The pdu and diskus sources were read in full for the walk, fold, thread, hard-link, and
output paths; `pdu --max-depth 1` and `2` were run on a small directory to confirm the
depth convention. fdu’s mechanisms were read at `a15b20f4` in `scan.rs`, `index.rs`,
`execution.rs`, `control.rs`, `control/gitignore.rs`, `classify.rs`, and
`platform_tuning.rs`. Screens used `hyperfine -N` with fdu’s cache directory isolated;
the generated-tree comparison ran 3 warm-ups and two passes of 12 runs in opposite
orders. Harness cells used the perf probe (`aggregate-summary` from a cold start;
`default-tree` with a snapshot present, which a metadata report does not read;
`cold-scan-index`) or the tool harness for the product `fdu-default-tree` contract.
Peak RSS outside the harness came from a small C launcher reading `wait4`, median of
five, because the harness’s own RSS floor is about 57 MiB on Linux.
System-call counts are `strace -c` summaries over all threads on `linux-v6.12`. The CPU
profile table was computed from the committed 2026-09-28 result files.

Not established here: bare-metal Linux, core counts other than four, cold cache, and
which structure holds the classifying summary’s extra memory.

## References

- [pdu 0.24.0 source](https://github.com/KSXGitHub/parallel-disk-usage/tree/c30e46f16478800d3304264fb9a49bebef9db5c6)
  (Apache-2.0; tag `0.24.0` is `4e192606`)
- [diskus 0.9.0 source](https://github.com/sharkdp/diskus/tree/d8a77db0f693ec32007cf337572b559cbbff50fe)
  (MIT or Apache-2.0)
- [dumac `1ffbe3c`](https://github.com/healeycodes/dumac/tree/1ffbe3c38d1066c45cac9b4ec1e31eb74edc1076)
  (no license declared; inspect only)
- [Linux tool comparison](../reports/report-2026-09-27-fdu-linux-tool-comparison.md) and
  its [indexed](../reports/fdu-linux-tool-comparison-result-2026-09-28-indexed.json) and
  [summary](../reports/fdu-linux-tool-comparison-result-2026-09-28-summary.json.gz)
  results
- [macOS tool comparison](../reports/report-2026-09-26-fdu-live-tool-comparison.md)
- [Peer agreement](../reports/report-2026-09-25-peer-agreement.md)
- [Cache economics brief](research-2026-09-27-cache-economics-and-default-plans.md)
- [Performance frontier](research-2026-08-10-performance-frontier.md) and
  [file roll-up engine survey](research-2026-08-06-file-rollup-engine.md)
- [Design principles](../architecture/fdu-design-principles.md) and
  [engine architecture](../architecture/fdu-engine-architecture.md)
- [Hypothesis registry](../guides/performance-loop.md#hypotheses): H59, H60, H64, H66,
  H72, H74, H84, H85, H157, H159, H161–H170
- [Experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md):
  exp-161, exp-170, exp-171, exp-173, exp-174, exp-187, exp-188, exp-189, exp-190,
  exp-191

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
