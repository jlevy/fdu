# Research: Uniformly Faster Than pdu, Subject by Subject and Mode by Mode

**Date:** 2026-09-29

**Author:** fdu project, with Claude Code

**Status:** Attribution complete; hypotheses H185–H190 registered (beads `fdu-n3yx`,
`fdu-1xa5`, `fdu-427s`, `fdu-54jm`, `fdu-glh5`); measured cells to follow on this branch
(`claude/pdu-uniform-lead`, stacked on [#162](https://github.com/jlevy/fdu/pull/162)).
Epic `fdu-faqa`. It makes no new wall claim: wall figures are cited from the quiet
20-pair cells of exp-194 and exp-195, and everything measured here is load-independent
(instructions, system calls, page faults, context switches) or marked as a screen taken
under load. **Built on the branch, unmeasured on wall (2026-09-29, host busy):** H185
(`c0da65ae`), H188 with H189 (`a0666bf0`, `7a3a7058`), H186 (`a356d456`) and H187
(`cfae174e`), each with its answers proved identical by the differential tests, the
goldens and the three-format answer diff, and its load-independent secondary recorded in
[the registry](../guides/performance-loop.md#hypotheses); their cells run when the host
is quiet, as exp-197 onward.

## Question

At the head of the Linux parity round, fdu’s default command is level with pdu’s default
on both nominated real trees and behind pdu’s `--max-depth 2` by 7% on
`node-modules-dense` and by 2.5% on the generated million-entry tree
([the Linux comparison](../reports/report-2026-09-27-fdu-linux-tool-comparison.md)). The
maintainer’s goal is stronger than level: fdu ahead of every pdu mode a user runs, on
every nominated subject, on Linux and, by plan, macOS.

So, for each subject and each pdu mode:

1. Where does each tool’s wall time go, and what is the per-entry difference made of?
2. What does fdu do that pdu does not, and which of it can the default path avoid
   without changing an answer?
3. Which levers, pre-registered, would put fdu ahead by a margin the accept rule can
   see, and in what order?

## Method and Regime

- **Host:** the 4-vCPU Intel Xeon at 2.1 GHz, Firecracker guest, Linux 6.18.44, ext4 on
  virtio, warm cache, root.
  The stabilization work for the release was building and testing on the same host
  throughout, so no timed cell was taken here.
- **Binaries:** fdu at `b8320ecb` (the engine is `ebc06c78`’s, the final head of the
  round: no file under `crates/` changed after it), the stripped release binary the
  round’s tool cells measured, and a `profiling` build of the same source with line
  tables for callgrind; pdu 0.24.0 and diskus 0.9.0 from `cargo install --locked`, with
  symbols.
- **Subjects:** `linux-v6.12` (92,474 entries, 5,769 directories, 358 `.gitignore` files
  holding 1,593 rules), `node-modules-dense` (79,957 entries, 9,439 directories, no
  `.gitignore`), and `linux-balanced-1m` (1,000,001 entries, 125,001 directories) as a
  screen. An empty directory isolates fixed cost.
- **Evidence labels:**
  - **[quiet]:** the 20-pair tool cells of exp-194 (`linux-v6.12`) and exp-195
    (`node-modules-dense`), which recorded every sample’s `wait4` rusage.
    Medians over the timed samples: 60 for fdu (three peers, each paired with it), 20
    per peer.
  - **[count]:** load-independent.
    Instructions are valgrind 3.22.0 callgrind with
    `--separate-threads=yes --fair-sched=yes`, user space only; system calls are
    `strace -f -c`; page faults, peak RSS and context switches are `wait4` rusage over 7
    runs, which loading changes little.
  - **[screen]:** a throwaway build of this head with a monotonic mark at each phase
    boundary, 10 runs per subject with the host at load 1.0–1.6. Proportions only: the
    walk phase is what load distorts, and the serial phases are what the screen is for.
- **pdu modes:** its default (`--max-depth 10`, `--min-ratio 0.01`, block size),
  `--max-depth 1` (the root’s total alone), `--max-depth 2` (the tree fdu’s `--depth 1`
  renders), `--quantity apparent-size`, `--json-output`, and `--no-sort`. Under
  callgrind and strace fdu’s adaptive pool scales to eight walkers, because the
  instrumentation makes every entry look slow to the calibration; walker sums stay
  comparable with native runs, futex counts do not.

## Findings

### 1. The Gap Is Utilization, Not Work

The quiet cells already say where fdu stands, and it is not where the instruction
profiles of the design study left it.
**fdu spends the least CPU of the three tools on both real trees and keeps the fewest
cores busy.** [quiet]

| `linux-v6.12`, 92,474 entries | Wall | User CPU | Kernel CPU | CPU total | Cores busy | Kernel share | Minor faults | Voluntary switches | Involuntary switches |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fdu default tree | 121.9 ms | 67.1 ms | 380.3 ms | 445.3 ms | 3.65 | 85.4% | 1,492 | 608 | 436 |
| pdu default | 121.2 ms | 55.9 ms | 406.1 ms | 461.3 ms | 3.81 | 88.0% | 2,326 | 128 | 26 |
| pdu `--max-depth 2` | 114.3 ms | 48.3 ms | 394.9 ms | 437.1 ms | 3.83 | 90.3% | 303 | 59 | 16 |
| diskus | 125.2 ms | 71.0 ms | 426.1 ms | 485.6 ms | 3.88 | 87.7% | 942 | 164 | 2,574 |

| `node-modules-dense`, 79,957 entries | Wall | User CPU | Kernel CPU | CPU total | Cores busy | Kernel share | Minor faults | Voluntary switches | Involuntary switches |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fdu default tree | 118.5 ms | 59.8 ms | 360.6 ms | 419.3 ms | 3.54 | 86.0% | 2,037 | 1,038 | 758 |
| pdu default | 118.9 ms | 53.0 ms | 388.0 ms | 442.0 ms | 3.72 | 87.8% | 2,144 | 399 | 18 |
| pdu `--max-depth 2` | 112.5 ms | 48.9 ms | 378.4 ms | 423.6 ms | 3.76 | 89.3% | 306 | 388 | 8 |
| diskus | 117.4 ms | 64.1 ms | 386.0 ms | 452.8 ms | 3.86 | 85.2% | 1,187 | 173 | 2,980 |

Per entry, fdu’s kernel time is the lowest (4.11 µs on `linux-v6.12` against pdu’s 4.27
and 4.39) because its `statx` is relative to the open directory and it pays no `opendir`
`fstat`; its user time is the highest (0.73 µs against 0.52 and 0.60) because it
classifies, retains an exact tree, and merges on a fifth thread.
The sums favour fdu by 3.5% on `linux-v6.12` and 5% on `node-modules-dense` against
pdu’s default, and are level with pdu at depth 2.

Wall is CPU divided by the cores kept busy.
Had fdu run at pdu’s utilization it would already be 3–5% ahead of pdu’s default on both
trees and level with depth 2. So the question splits in two: where the 0.35–0.46 idle
cores go, and which CPU is still avoidable.

**Where the idle cores go.** On four vCPUs with four walkers and one consumer, the walk
itself keeps every core busy: while it runs, CPU is the constraint and every CPU
millisecond, on any thread, costs a quarter of a millisecond of wall.
Idle cores come from the serial phases before and after it.
The phase screen [screen]:

| Phase, medians of 10 runs | `linux-v6.12` | `node-modules-dense` |
| --- | ---: | ---: |
| Process total from `main` to return | 80.3 ms | 75.6 ms |
| Argument parsing, request, plan, root stat, queue | 0.41 ms | 0.36 ms |
| Spawning four walkers | 0.30 ms | 0.27 ms |
| Walk: first walker’s exit after spawn | 75.5 ms | 70.2 ms |
| Walk: spread between the first and last walker exit | 0.19 ms | 0.14 ms |
| Consumer drains its channel after the last walker exits | 0.28 ms | 0.22 ms |
| `finish` (kept files, bottom-up merge, baseline) | 0.38 ms | 0.76 ms |
| Freshness pass | 0.02 ms | 0.03 ms |
| `report` (the tree query) | 1.16 ms | 2.29 ms |
| Releasing the folded index, inline | 0.84 ms | 1.12 ms |
| Render and write the tree, flush diagnostics | 0.30 ms | 0.18 ms |
| **Serial after the walk** | **2.8 ms** | **4.5 ms** |

Outside `main`, `execve`, dynamic linking and process exit cost about 3 ms for fdu and
pdu alike (an empty directory takes 4.2 ms and 3.9 ms end to end).
The walk’s own tail is tight: the last walker leaves 0.2 ms after the first, and the
consumer is idle within 0.3 ms of it.
What is serial and fdu’s own is the 2.8–4.5 ms after the walk, 3.5–6% of the run, and
most of it is two things the answer does not need at that cost: the tree query builds a
row, with a `PathBuf` and a `String`, for every child of every expanded directory, sorts
them all with a comparator that clones two `String`s per comparison, and only then
applies the 1% share threshold (9.1M instructions on `linux-v6.12`, 16.3M on
`node-modules-dense`); and the folded index, below the 64Ki-entry threshold for a
detached release (H156), is freed inline, 30k frees on the critical path.

pdu’s own serial work after the walk is smaller: its main thread executes 4.9M
instructions in all on `linux-v6.12` (cull, sort, render) and 0.9M at depth 1.

### 2. System Calls, Mode by Mode

`strace -f -c` [count]. fdu’s `futex` counts are inflated by the eight walkers strace
provokes; natively the round measured 608 and 1,038 voluntary switches.

| `linux-v6.12` | `statx` | `getdents64` | `openat` | `close` | `fstat` | `read` | `futex` | `sched_yield` | `mprotect` | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fdu default tree | 92,836 | 11,538 | 6,135 | 6,135 | 4 | 1,261 | 5,210 | 0 | 76 | 123,447 |
| fdu `--no-gitignore` | 92,478 | 11,538 | 5,776 | 5,776 | 4 | 13 | 5,093 | 0 | 82 | 121,003 |
| fdu `--view summary` | 87,007 | 11,538 | 6,135 | 6,135 | 4 | 1,261 | 3,451 | 0 | 74 | 115,836 |
| pdu default | 92,484 | 11,540 | 5,782 | 5,781 | 5,773 | 28 | 16 | 45 | 523 | 122,156 |
| pdu `--max-depth 2` | 92,484 | 11,540 | 5,782 | 5,781 | 5,773 | 28 | 25 | 140 | 20 | 121,719 |
| pdu `--max-depth 1` | 92,484 | 11,540 | 5,782 | 5,781 | 5,773 | 28 | 13 | 306 | 22 | 121,865 |
| pdu `--json-output` | 92,484 | 11,540 | 5,782 | 5,781 | 5,773 | 28 | 89 | 1,157 | 511 | 123,286 |
| pdu `--quantity apparent-size` | 92,484 | 11,540 | 5,782 | 5,781 | 5,773 | 28 | 53 | 235 | 508 | 122,369 |
| diskus | 92,478 | 11,540 | 5,782 | 5,782 | 5,773 | 19 | 46 | 5,530 | 162 | 127,356 |

| `node-modules-dense` | `statx` | `getdents64` | `openat` | `close` | `fstat` | `read` | `futex` | `sched_yield` | `mprotect` | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fdu default tree | 79,961 | 18,878 | 9,447 | 9,447 | 4 | 14 | 8,825 | 0 | 95 | 126,921 |
| fdu `--view summary` | 70,417 | 18,878 | 9,447 | 9,447 | 4 | 14 | 6,421 | 0 | 114 | 114,977 |
| pdu default | 79,967 | 18,879 | 9,452 | 9,451 | 9,443 | 28 | 84 | 6,988 | 413 | 134,877 |
| pdu `--max-depth 2` | 79,967 | 18,879 | 9,452 | 9,451 | 9,443 | 28 | 16 | 3,983 | 25 | 131,382 |
| pdu `--max-depth 1` | 79,967 | 18,879 | 9,452 | 9,451 | 9,443 | 28 | 16 | 645 | 24 | 128,042 |
| diskus | 79,961 | 18,879 | 9,448 | 9,448 | 9,443 | 15 | 1,141 | 20,579 | 331 | 149,524 |

Three things follow.

- **Every pdu mode makes the same calls.** Depth, quantity, sort and JSON change only
  the post-walk work on pdu’s main thread and how much it retains (`mprotect` 523
  against 20 is the retained tree’s arena growth).
  The kernel budget of every pdu mode is one, so a fdu change that clears the accept
  rule against pdu’s default clears it against every mode except where the retained-tree
  difference shows: 2–3% on the real trees between pdu’s default and its depth-2 mode,
  from 2,300 fewer page faults and 4.9M fewer main-thread instructions.
- **fdu still describes every directory twice** on the tree route: a `statx` from the
  parent’s listing and the listing itself.
  The summary route skips it by `d_type` (H72), which is the 5,829 and 9,544 fewer
  `statx` in its rows.
  On `node-modules-dense` that is 11.9% of all `statx` calls; on `linux-v6.12` 6.3%; on
  `linux-balanced-1m` 12.5%. The tree report never reads a directory’s own attributes
  (section 5), so the tree route can skip it the same way.
- **fdu reads each `.gitignore` in eight `read` calls** of 32, 32, 64, 128, 256, 512,
  1,024 and 2,048 bytes, because `read_to_end` behind `take` cannot see the length the
  preceding `statx` returned: 11 calls per control file, 3,938 on `linux-v6.12`, 3.2% of
  the run’s system calls, on the walkers.

The `futex` difference is the fifth thread: the consumer parks on its channel when it is
empty and a walker’s send wakes it, once per chunk of at most four listings.

### 3. Instructions, Thread by Thread

callgrind, user space, per entry [count]. “Main” is fdu’s consumer, pdu’s calling thread
and diskus’s receiver.

| Run | `linux-v6.12` main | walkers | total | `node-modules-dense` main | walkers | total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| fdu default tree | 2,604 | 1,166 | 3,770 | 1,509 | 1,339 | 2,847 |
| fdu `--no-gitignore` | 1,116 | 1,124 | 2,240 | 1,452 | 1,335 | 2,787 |
| fdu `--view summary` | 4,038 | 1,433 | 5,471 | 90 | 1,573 | 1,663 |
| pdu default | 53 | 2,779 | 2,831 | 137 | 3,059 | 3,196 |
| pdu `--max-depth 2` | 12 | 2,594 | 2,606 | 11 | 2,853 | 2,864 |
| pdu `--max-depth 1` | 10 | 2,594 | 2,603 | 11 | 2,858 | 2,869 |
| pdu `--json-output` | 18 | 2,805 | 2,823 | 127 | 3,069 | 3,196 |
| pdu `--quantity apparent-size` | 54 | 2,779 | 2,834 | 177 | 3,024 | 3,201 |
| pdu `--no-sort` | 50 | 2,779 | 2,829 | 132 | 3,054 | 3,185 |
| diskus | 6 | 2,073 | 2,079 | 7 | 2,237 | 2,244 |

fdu’s walkers are the cheapest walkers measured (1,166 against pdu’s 2,594–2,779); its
consumer is what pdu does not have.
Against the Q0 profile of the design study (23,554 per entry, 22,254 of them on the
consumer) the round removed 84% of the consumer’s work; what is left is worth reading in
detail because it now sets the CPU budget.

**The tree route’s consumer, `linux-v6.12`, 240.8M instructions**, grouped by inclusive
cost:

| Work | Instructions | Per entry | Note |
| --- | ---: | ---: | --- |
| `.gitignore` classification (`is_ignored_within`, `decide`, literal and tail probes, `memcmp`) | 111M | 1,200 | 1.7 governing sources per entry; about 400 per `decide` call after H171 and H183 |
| Name sort and dedup of every listing (`sort_unstable_by`) | 28M | 308 | Sorts 92k names although the folded index keeps 5.9k of them as entries |
| Per-file roll-up: `file_contribution`, `InternedRollUp::merge`, two empty `BTreeMap`s built, cloned and dropped per file | 21M | 227 | The `by_ext` maps are empty on this tier (H176) and still constructed |
| `ControlTable::upsert` (parsing and indexing 358 files, 1,593 rules) | 13.6M | 147 | 38k per file |
| Directory map: `HashMap<PathBuf>` SipHash, `Path` hashing and component comparison | 19M | 205 | H167’s target; `Path` hashes and compares by component, not by byte |
| Tree query (`report_in`) | 9.1M | 98 | Serial, after the walk |
| Index release (`drop_glue::<Index>`) | 3M | 33 | Serial, inline |
| `finish` (kept files, bottom-up merge, baseline) | 1.5M | 16 | Serial |
| Allocation (`malloc`, `free`) | 9M | 97 | 1.1 per entry: the name of each kept entry, `Box<DirectoryEntry>`, child vectors |
| Startup (`ld.so`, clap, request, `TypeRegistry`) | 1.1M | — | pdu’s is 0.86M |

Without `.gitignore` the same consumer runs 103M instructions, and on
`node-modules-dense` 121M: the tree build itself (sort, roll-up, directory map, arena)
costs 1,100–1,500 per entry, pdu’s whole worker 2,600–3,000 including the walk.

**The summary route’s consumer, `linux-v6.12`, 373M instructions** — more than the tree
route’s, on the route pdu’s `--max-depth 1` is compared with.
180M of it is `std::path`: `compare_components` (94M) because the fold compares each
op’s parent with the cached parent by `Path` equality, which walks components;
`Components` parsing (56M + 52M + 43M) for `parent()` and `file_name()` on each op’s
owned path; and `Path` hashing (28M) for the `HashSet<PathBuf>` of ignored heads.
Classification proper (`is_ignored_within`) is 111M as on the tree route.
On `node-modules-dense`, with no rules, the same consumer is 7M. The quiet cells put the
whole difference in wall: the controls-on summary took 116.0 ms against 104.4 ms with
`--no-controls`, 77.3 ms of user CPU against 31.9.

**fdu’s walkers, per thread on `linux-v6.12`**: `malloc` and `free` 30% (one `OsString`
per retained name, freed on the walker after the listing comes back), the native
`getdents64` reader 25% (record parsing, the `memchr` for each NUL), `record_entry` 7%,
admission 3%, `memcpy` 3%, the `syscall` wrapper 1%.

**pdu’s workers, per thread**: `malloc`, `free` and `realloc` 30% (an `OsString`, a
joined `PathBuf` and a `DataTree` node per entry), `CStr::from_bytes_with_nul` 5%,
`try_statx` 4%, rayon’s collect and bridge 8%, `Path::_join` 3%, `readdir` and
`ReadDir::next` 4%. pdu’s is the cost of statting by absolute path and building a node
per entry; the mode changes none of it.

### 4. Memory and Switches

`wait4` rusage over 7 runs under load [count]; the launcher’s own 9.2 MiB high-water
mark is the floor of every peak, so pdu’s depth-2 peak is a bound.

| Command, `linux-v6.12` | Minor faults | Peak RSS | Voluntary switches | Involuntary switches |
| --- | ---: | ---: | ---: | ---: |
| fdu default tree | 1,974 | 11.7 MiB | 139 | 902 |
| fdu `--view summary` | 3,261 | 14.7 MiB | 69 | 962 |
| pdu default | 2,573 | 12.4 MiB | 21 | 1,817 |
| pdu `--max-depth 2` | 547 | ≤ 9.2 MiB | 23 | 1,373 |
| pdu `--max-depth 1` | 547 | ≤ 9.2 MiB | 13 | 1,246 |
| diskus | 1,322 | ≤ 9.2 MiB | 99 | 1,882 |

| Command, `node-modules-dense` | Minor faults | Peak RSS | Voluntary switches | Involuntary switches |
| --- | ---: | ---: | ---: | ---: |
| fdu default tree | 2,560 | 13.4 MiB | 225 | 1,555 |
| fdu `--view summary` | 3,583 | 12.8 MiB | 566 | 1,611 |
| pdu default | 2,395 | 11.8 MiB | 36 | 1,435 |
| pdu `--max-depth 2` | 554 | ≤ 9.2 MiB | 18 | 1,622 |
| pdu `--max-depth 1` | 539 | ≤ 9.2 MiB | 13 | 1,365 |
| diskus | 1,480 | ≤ 9.2 MiB | 85 | 1,815 |

fdu’s default tree holds fewer pages than pdu’s default and about 1,500 more than pdu at
depth 2; at about a microsecond per fault in a guest that is under 0.5% of CPU. The
involuntary switches in the quiet cells (436 and 758 for fdu against pdu’s 26 and 18)
are the fifth thread being preempted on four vCPUs; the voluntary ones (608 and 1,038
against 128 and 399) are the consumer parking on its channel.
Each pair costs a few microseconds of kernel time, about 1% of CPU in all, which is why
H181’s cut in wakes did not reach wall.

### 5. What fdu Does That pdu Does Not

Ranked by CPU on the default path, `linux-v6.12`, with `node-modules-dense` in
parentheses. “Avoidable” means without changing any answer of the command.

| # | Work | Cost | Where | Avoidable on the default path? |
| ---: | --- | --- | --- | --- |
| 1 | `.gitignore` classification and the control files’ parsing | 125M consumer instructions (0); 3,938 walker system calls (0) | `control.rs`, `control/gitignore.rs`, `scan.rs:4239` | Not the classification: it is the answer’s ignored share. Its `decide` is about 400 instructions per source call and could fall further (H190); the control-file read can be 4 calls instead of 11 (H189) |
| 2 | The directory `statx` from the parent’s listing | 5,829 (9,544) `statx`; 6.3% (11.9%) of all | `scan.rs`, `linux_dents::StatPolicy` | **Yes.** The tree report reads no directory’s own attributes: rows carry roll-ups, `newest_mtime_ns` is the files', symlinks and other kinds contribute nothing, and `dev` is read only under `--one-filesystem`, where the summary policy already keeps the stat (H185) |
| 3 | A name sort of every listing | 28M (12M) | `index.rs:1813` | **Yes.** Only the entries the folded index keeps need name order; folded files need only the repeated-name dedup, which a per-listing hash set gives without a sort (H187) |
| 4 | Two empty extension maps built, cloned and freed per file | 21M (17M) | `index.rs:5574`, `:335` | **Yes.** A scalar roll-up path for the tier that keeps no `by_ext` (H187) |
| 5 | The directory map keyed by `PathBuf` under SipHash with component-wise hashing | 19M (15M) | `index.rs:1627` | **Yes.** Key by bytes, or carry a walker-assigned token (H167, inside H187) |
| 6 | The tree query building and sorting a row for every child of every expanded directory before the share threshold | 9.1M (16.3M), serial | `query_report.rs:3129`, `:3282` | **Yes.** Apply the threshold to the roll-up scalars first and sort borrowed names (H186) |
| 7 | Freeing the folded index inline | 3M (5.3M), serial | `lib.rs:314` | **Yes.** The threshold for a detached release counts entries; a folded index has few entries and many allocations (H186) |
| 8 | One owned name per retained entry on the walker | about 350 walker instructions per entry | `scan.rs:3730` | Yes, H177’s arena, queued |
| 9 | A fifth thread, and its handoffs | 608 (1,038) parks, 436 (758) preemptions | `scan.rs` | Partly: H178’s consumer-walks-when-idle, queued |
| 10 | Exact retention of every directory and the largest files | `Box<DirectoryEntry>` and a child vector per directory; 1,500 more faults than pdu at depth 2 | `index.rs` | No: it is the product. H172 already folded the files |
| 11 | Counts, apparent and allocated bytes, newest file time per row | scalar adds | `index.rs` | No, and negligible |
| 12 | Rendering, notes and the performance footer | 0.5M | `report_format.rs` | No, and negligible |

pdu’s own extra work, for the record: one `fstat` per directory (glibc’s `opendir`), a
`statx` that resolves the full path for every entry, a `PathBuf` join and a `DataTree`
node per entry, rayon’s spinning (up to 6,988 `sched_yield` on the dense tree), and at
its default depth the retained tree’s growth and cull.
None of it is ours to remove; fdu’s kernel time is already below it.

### 6. pdu’s Modes and fdu’s Analogues

| pdu mode | Its cost against pdu’s default | fdu analogue | fdu’s standing now [quiet] |
| --- | --- | --- | --- |
| default (`--max-depth 10`, `--min-ratio 0.01`) | — | `fdu PATH` (depth 5, 1% share) | level: +1% [−2%, +2%] and +1% [−2%, +4%] |
| `--max-depth 2` | 2,300 fewer faults, 4M fewer main-thread instructions; −6% wall on `linux-v6.12`, −5% on the dense tree | `fdu --depth 1 PATH`; the same walk and index as the default | behind by 1% [−7%, +1%] and 7% [−10%, −4%] |
| `--max-depth 1` (the total) | as depth 2 | `fdu --view summary PATH` | 116.0 ms against 114.3 ms on `linux-v6.12` (level); 101.1 against 112.5 on the dense tree (fdu ahead) |
| `--quantity apparent-size` | none | `--size apparent` | as the default |
| `--json-output` (the rendered tree, 2 KB) | none; 3M fewer main-thread instructions | `fdu --view tree --format json` (28 KB, 0.080 s screen) | as the default |
| `--no-sort` | none | none: fdu’s output is always deterministic | — |
| `-H` | a `DashMap` insert per file with `nlink > 1` | none yet (planned unique bytes) | — |
| `--threads N` | at 4 vCPUs, `auto` is 4 | `Workers::scan` in Rust only | — |

One mode deserves a warning rather than a cell.
`fdu --format json PATH` with no view is not the analogue of `pdu --json-output`: a
machine format defaults to the complete List view, so on `node-modules-dense` it writes
50 MB in 0.88 s [screen]. 74% of its 6.7G instructions are the row sort of section 5
item 6 cloning two `String`s per comparison, 57k instructions per row, and 45% of the
rest is the JSON emitter at 35k per row.
The tree view’s JSON is 28 KB in 0.080 s. H186 fixes the comparator for every view,
which takes the List view’s sort from 4.6G instructions to a few tens of millions; the
emitter is `fdu-faqa`’s follow-up, not a pdu question.

### 7. The Model, and What Each Lever Buys

```
wall ≈ 3 ms (exec, link, exit) + serial(fdu) + CPU(walk) / 4     on four vCPUs, warm
serial(fdu) ≈ 0.7 ms before the walk + 2.8–4.5 ms after it
CPU(walk)   ≈ kernel 360–380 ms + walkers 30 ms + consumer 30–60 ms
```

pdu’s terms: serial about 2 ms, CPU 437–461 ms.
To be ahead of pdu’s depth-2 mode by the 3% the accept rule can see, fdu needs about 8
ms off `node-modules-dense` and 6 ms off `linux-v6.12`, from CPU at a quarter each or
from serial time at par.
The levers, with what the counts above say each is worth:

| Lever | Mechanism | CPU or serial saved | Predicted wall, `linux-v6.12` | `node-modules-dense` |
| --- | --- | --- | --- | --- |
| H185 | Skip the directory and symlink `statx` on the tree route (H72’s policy) | 5.8k and 9.5k `statx` at 2–3 µs | −2% to −4% | −4% to −6% |
| H186 | Threshold before rows, borrowed-name sort, detached release | 1.5–2.5 ms serial | −1.5% to −2.5% | −2.5% to −4% |
| H187 | Sort only kept kinds, scalar roll-up, byte-keyed directory map | 60–75M consumer instructions | −3% to −5% | −2% to −4% |
| H188 | Byte-wise paths in the summary fold | 150M consumer instructions on the summary route | −5% to −9% on `aggregate-summary` | 0 |
| H189 | Size the control-file read from its `statx` | 2.5k walker system calls | −0.5% to −1% | 0 |
| H190 | `decide` below 250 instructions per source call | 40–50M consumer instructions | −1.5% to −3% | 0 |
| H177 (queued) | Per-listing name arena | 30M walker instructions | −1% to −2% | −1% to −2% |
| H178 (queued) | The consumer walks when its channel is empty | 700 preemptions, 900 parks | −1% to −3% | −1% to −3% |
| H169 phase 3 (queued) | Open each directory relative to its parent | one path walk per directory | −1% to −2% | −1.5% to −2.5% |

The first three, stacked, predict fdu 7–11% ahead of pdu’s default and 4–8% ahead of its
depth-2 mode on both real trees; H188 puts the summary 4–7% ahead of pdu’s
`--max-depth 1` on `linux-v6.12`, where it is level today.
The queued items each buy 1–3% and carry more risk: H178 is a scheduling policy and
takes the three-stage qualification; H169 phase 3 needs its fd budget.

Two queued items lose their case on this profile.
**H179** (directory attributes from the opened fd) removes nothing once H185 removes the
parent-side `statx` from the tree route; it remains valid for the full-index route,
which needs directory mtimes for the cache, but that is not the route a user’s default
command takes. **H174** (a walker-side listing digest) moves consumer work to walkers;
under CPU saturation moving work saves nothing, and the consumer is busy about 60% of
the walk on `linux-v6.12` (46 ms of a 76 ms walk) and 30% on `node-modules-dense`, below
H174’s own 80% gate.
**H164**’s tree route (classification on walkers) is the same case.

## Hypotheses

Registered in [the registry](../guides/performance-loop.md#hypotheses) as H185–H190,
with the accept rule of [the loop](../guides/performance-loop.md#the-accept-rule): a
median at least 3% faster with the 95% interval below zero, 20 pairs because every
effect is predicted under 10%, a nominated real tree deciding, placebos on the routes a
change does not touch, and answers identical under the differential tests and the
three-format answer diff over all three subjects.
Ranked by expected gain per unit of risk:

1. **H185** — the tree route skips directory and symlink stats.
   One variable, the mechanism exists, the exactness argument is in section 5 item 2,
   and the proof is the transient-versus-indexed differential of H172 plus the answer
   diff. Deciding: `default-tree` on `node-modules-dense` and `linux-v6.12`; placebo
   `aggregate-summary --no-controls` (already skipping) and `cold-scan-index` (full
   index, unchanged); secondary `strace -c` `statx` down by the directory count.
2. **H188 with H189** — control-file handling: byte-wise paths in the summary fold, and
   a pre-sized control-file read.
   Deciding: `aggregate-summary` on `linux-v6.12`; `default-tree` there as H189’s
   secondary; placebos: both jobs on `node-modules-dense`, which has no rules.
3. **H186** — the serial tail: the share threshold before rows, borrowed names in the
   sort, a detached release of the folded index.
   Deciding: `default-tree` on both real trees; placebo
   `aggregate-summary --no-controls`; the List view’s JSON as a screen.
4. **H187** — the tree tier’s consumer, as one structural composite: sort and dedup only
   what the tier keeps, a scalar roll-up for folded files, a byte-keyed directory map.
   Deciding: `default-tree` on both real trees; secondary consumer instructions −35% or
   more and `cold-scan-index` non-inferior (the full-index builder shares the map);
   placebo `aggregate-summary --no-controls`.
5. **H177**, then **H178**, as queued, re-predicted above.
6. **H190** — a second pass over `decide`, only after H187 and a fresh profile.
7. **H169 phase 3**, after its fd budget is designed.
8. **H179** and **H174** stay behind their gates, which this profile does not open.

## macOS

The platform review’s plan (§7 of
[the platform review](research-2026-09-29-platform-review-of-the-linux-round.md)) has
cells M1–M10; this track adds the following, to run in the same regime and order, after
M4, with ids from the Darwin block once it is opened.

| Lever | Platform reach | macOS analogue and cell |
| --- | --- | --- |
| H185 | Linux only in effect | None: `getattrlistbulk` returns every child’s attributes in the listing, so there is no per-directory stat to skip (`macos_bulk.rs:33-44`). The `StatPolicy` change compiles everywhere and is a no-op there; **M11** is a placebo: `default-tree` on S1 and S2, H185 against its base, must include zero |
| H186 | Portable | The same code runs. **M12**: `default-tree` on S1, S2 and S5 (55k directories, the largest serial tail), 20 pairs, predicted −1% to −4%; `aggregate-summary` placebo |
| H187 | Portable | The same code runs. **M13**: `default-tree` on S1, S2 and S5, 20 pairs; on the M1 Pro’s eight performance cores the consumer is not saturating a core, so the prediction is the tail’s share only, −1% to −2%; the RSS primary of M5 must not move |
| H188, H189 | Portable | **M14**: `aggregate-summary` on S1 and S3 (rule-bearing), 20 pairs, predicted −3% to −7%; S2 placebo |
| H177 | Linux reader | H184, the borrowed-name bulk listing, is the macOS form; screen only, gated on M9 |
| H178 | Portable | A scheduling policy: its deterministic characterization runs in CI on every platform; its macOS cell waits for M7’s `starved_ns` |
| H169 phase 3 | Linux | Refuted on macOS (exp-024, exp-038) |
| H190 | Portable | With H188 on M14 |

The “uniformly faster” claim on macOS needs, besides these, M8’s peer standings anchored
on the head that carries the accepted levers, with `pdu-default`, `pdu` (depth 2) and a
`pdu --max-depth 1` adapter beside `dumac`.

## References

- [The Linux comparison](../reports/report-2026-09-27-fdu-linux-tool-comparison.md) and
  exp-194’s and exp-195’s evidence
- [The hot-path brief](research-2026-09-29-linux-peers-matchers-and-hot-path.md) and
  [the pdu brief](research-2026-09-28-pdu-and-the-linux-peer-gap.md)
- [The platform review](research-2026-09-29-platform-review-of-the-linux-round.md)
- [The performance loop](../guides/performance-loop.md) and
  [the runbook](../guides/performance-loop-runbook.md)
- pdu 0.24.0 at `c30e46f`: `src/tree_builder.rs`, `src/fs_tree_builder.rs`,
  `src/app/sub.rs`, `src/data_tree/retain.rs`, `src/data_tree/sort.rs`

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
