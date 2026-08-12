# Research: fdu Performance Frontier — Cost Model, Platform Levers, and the Experiments That Matter

**Date:** 2026-08-10

**Author:** fdu project, with Claude Code research assistance

**Status:** Proposed

## Overview

The [original engine research](research-2026-08-06-file-rollup-engine.md) chose the
architecture and catalogued proven techniques from twelve tools.
The [evidence research](research-2026-08-09-end-to-end-performance-evidence.md) defined
how a performance claim must be measured.
This document does the third thing: it builds a first-principles cost model of the work
fdu performs, measures where the current implementation sits against that model, surveys
the platform-specific primitives that change the constants — several of which no
existing tool uses — and ranks the experiments that would retire the largest
uncertainties.

The framing question is not “which optimizations exist” but “which few design points
dominate the outcome, per use case.”
fdu has four distinct performance jobs, and they reward different physics:

1. **Cold active traversal** — first scan of a large tree.
   Bounded by storage latency times unhidden depth of the request queue, then by syscall
   count.
2. **Warm active traversal** — re-scan with the OS metadata cache hot.
   Bounded almost purely by syscall count times per-syscall cost, then by userland
   per-entry constants.
3. **Warm revalidation** — snapshot load plus a stat sweep proving the cache still tells
   the truth. Same physics as (2) plus snapshot I/O, with one platform-specific escape
   hatch that can skip the sweep entirely.
4. **Passive watching / incremental update** — a resident index absorbing deltas.
   Bounded by per-event constants and, critically, by how rarely the escalation paths
   (full-tree reconcile) fire.

The synthesis draws on four parallel investigations conducted for this document: a
line-level review of the current fdu engine; source review of bfs, dut, pdu, diskus, and
jwalk checked out under `attic/`; macOS/APFS platform research; and Linux/cloud platform
research. It is written to feed the live
[performance loop](../guides/performance-loop.md) and its
[experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md): the
candidate backlog below continues the loop’s hypothesis registry, and the findings are
reconciled against experiments exp-000 through exp-005. Line references are against the
2026-08-10 working tree; the loop has since landed commits through `954d27b`, so line
numbers may drift.

## Questions to Answer

1. What is the theoretical floor for each of the four jobs, per platform, and how far is
   the current implementation from it?
2. Why is the measured revalidation curve superlinear (725 ms at 100k → 63 s at 1M), and
   what does the answer imply about where to optimize?
3. Which platform primitives change the constant factor by integer multiples —
   `getattrlistbulk`, io_uring, fanotify, the FSEvents journal, inode-ordered statting —
   and which are reliable enough to build on?
4. How should traversal adapt to filesystem type, storage class, and cache state, rather
   than being tuned for one host?
5. Which uncertainties are load-bearing enough that they should be measured before
   further engine implementation?

## Scope

**Included:**

- Review of `crates/fdu/src` (scan, index, snapshot, watch, types), the perf probe, both
  benchmark harnesses, and the committed realtree baseline results.
- Direct source review of tools checked out under `attic/`: bfs (4af45dc), dut (68d4ba2,
  GPL — ideas only), pdu (c30e46f), diskus (90196e9), jwalk (v0.9.0), plus the existing
  dust checkout.
- Platform research with cited sources: getattrlistbulk, APFS concurrency, FSEvents
  journal resume, Apple Silicon QoS, io_uring status and cloud availability, fanotify,
  overlayfs, cloud storage characteristics.

**Excluded:**

- Implementation, and any performance claim.
  Numbers quoted from fdu runs are exploratory except where they cite the committed
  realtree baseline.
- Content-tier metric *implementation*, and full design of the composable CLI surface
  (owned by
  [plan-2026-08-10-fdu-composable-cli-surface.md](../specs/active/plan-2026-08-10-fdu-composable-cli-surface.md)).
  Two interactions are covered because they shape the cache format: policy parameters
  must not multiply engine variants, and the content-tier derived-data cache’s economics
  (see the tier findings).
- Rotational-disk optimization.
  HDD-specific work (FIEMAP physical ordering, single-thread heuristics beyond a safe
  fallback) is explicitly out of scope: the target hardware is MacBook internal SSDs and
  cloud storage, for interactive use and agent/CI workloads.

### Loop Update: 2026-08-12

The original research above scoped out implementation; the performance loop has since
tested its highest-ranked ideas through exp-023. The durable changes are:

- warm reconciliation and snapshot constants improved through borrowed path components,
  direct child expectations, extension interning, and single-pass checksum/parse;
- breadth-first traversal was reworked into a region scheduler and remeasured against
  depth-first, removing the original parallel queue’s wall and RSS concern;
- service-time calibration now retains six workers on fast small trees and activates up
  to sixteen only after the first 16k entries demonstrate a slow filesystem path; and
- the macOS cold walker now batches enumeration and complete stat-tier metadata through
  `getattrlistbulk`, falling back for a complete directory on any malformed response,
  unsupported filesystem, mount point, or firmlink.

The current stack versus the pre-work binary is 53.49% faster for cold indexed scans,
58.20% faster for producer-only scans, 51.32% faster through snapshot save, 20.60%
faster for full warm revalidation, and 36.08% faster for snapshot load on the current
60,067-entry APFS subject (exp-023). The platform accelerator itself improves the
720,805-entry cold-index job 30.13% and producer wall 41.60% over the adaptive portable
control (exp-022). These are warm-steady operating-system-cache results, not
controlled-cold claims.
H26 is implemented only for cold scans; using the same reader for full or
FSEvents-scoped reconciliation remains open.

## Findings

### A First-Principles Cost Model

Every job decomposes into four cost pools:

1. **Syscall count × per-syscall cost.** The irreducible work is one directory
   enumeration per directory and one metadata fetch per entry.
   The design space is how many entries each syscall amortizes over: `readdir` via libc
   batches ~32 KiB of names per `getdents`; a metadata call per entry is the dominant
   count (one `fstatat`/`statx` each) *unless* the platform offers bulk retrieval.
   macOS `getattrlistbulk` returns names *and* attributes for hundreds of entries per
   syscall — it collapses pool 1’s per-entry term by roughly two orders of magnitude.
   Per-syscall cost is roughly 1–2 µs warm on Linux and measurably higher on macOS,
   which is why syscall *count* matters more there.
2. **Metadata-cache misses × storage latency.** When the dentry/inode/vnode caches miss,
   the walk pays storage-latency round trips: ~10–100 µs on Apple/NVMe SSDs, ~0.5 ms on
   network block storage (EBS gp3), milliseconds on network filesystems.
   Cold performance is therefore `misses × latency ÷ effective queue depth`: the only
   levers are fewer misses (access ordering, batching) and more overlap (parallelism,
   io_uring).
3. **Userland per-entry work.** Allocations, path materialization, tree descents,
   hashing, reducer merges.
   The floor is near zero; the current engine spends 2–6 µs/entry here in places, which
   is the same order as the syscalls it wraps.
4. **Output and serialization.** Bounded by result shape, not tree size, if rendering is
   a query over pre-computed roll-ups; unbounded if it re-walks.

Concrete floors this model implies, for one million entries with full per-entry stats:

| Job | Floor arithmetic | Floor |
| --- | --- | --- |
| Warm scan, Linux, serial | 1M × (~1 µs statx + ~0.1 µs amortized getdents) | ~1–2 s |
| Warm scan, Linux, 8-way | above ÷ ~6 effective | ~0.2–0.4 s |
| Warm scan, macOS, bulk | 1M ÷ ~300 entries/syscall × ~5 µs + kernel B-tree work | dominated by kernel; dumac measured ~1.3 µs/entry wall at 400k |
| Cold scan, EBS gp3 | ~1M/16 inode-block reads × 0.5 ms ÷ QD 8 | tens of seconds; IOPS-capped |
| Cold scan, local NVMe | same misses at ~20–50 µs ÷ QD 32 | a few seconds |
| Warm revalidate with journal resume | O(changed entries), not O(n) | milliseconds when quiet |

Two consequences worth internalizing.
First, **on a MacBook the walk is syscall-bound, not I/O-bound**: dumac’s flamegraph
shows 91% of wall time inside syscalls after batching, so userland micro-optimization
stops mattering there once batching is in place — the syscall count *is* the program.
Second, **on cloud runners the warm case mostly does not exist**: dentries cost ~200 B
and cached inodes ~1 KB of unswappable kernel slab, so a 10M-file tree needs ~12 GB of
metadata cache that a 4–16 GB CI runner cannot hold.
The snapshot is not an optimization there; it is the only warm path the environment
permits.

### Where the Current Engine Spends Its Time

The realtree baseline — exp-000 in the
[experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md) (59,654
entries, M1 Pro, warm cache; raw run artifacts are machine-local and gitignored) —
decomposes as: cold-scan-producer 385.8 ms (~6.5 µs/entry — walk + stat),
cold-scan-index 514.5 ms (index apply adds ~2.1 µs/entry on one thread), warm-revalidate
472.3 ms, warm-snapshot-load 219.5 ms (~3.7 µs/record), cold-snapshot-save 36.9 ms.
Against the model: the warm sweep is ~82% syscall/walk and ~18% index work — but the
index share is pure userland constant, and both shares have large known reductions.

The line-level review found the losses concentrated in six places:

1. **The exclusive reconcile path re-derives what it already holds.**
   `ReconcileTarget::Direct` still routes child expectations through
   `collect_child_states` (`scan.rs:290`), which per child joins a `PathBuf` and calls
   `index.expectation()` — two full root-relative descents, each through a `normalize()`
   that heap-allocates one `OsString` per path component.
   The shared-handle twin (`collect_child_expectations`, `index.rs:1183`) already reads
   state directly off the child `EntryId` with zero path work; commit `abeb377` applied
   it only to the shared path.
   Roughly 13 of the ~15 heap allocations per *unchanged* entry exist to re-derive an
   `EntryId` the iterator already had.
   An equivalence test (`index.rs:1292`) already locks the two paths together, so the
   fix is a dispatch change.
2. **The reconcile sweep is serial.** The new parallel walker is wired into `scan()`
   only; `reconcile_target_inner`’s queue (`scan.rs:922`) still runs one thread, one
   outstanding stat at a time.
   Every metadata-cache miss stalls the only worker — this multiplies the cold and
   over-capacity cases below.
3. **Snapshot load replays the delta contract record by record.** `parse_stream`
   (`snapshot.rs:380-388`) performs per record: a `path_of` parent walk, a `lookup`, a
   one-op `Observation` through `apply_baseline` (which re-normalizes, re-descends from
   the root via `ensure_dir_chain`, merges reducers with per-ancestor `String` clones,
   and journal-clones the applied delta), then a third `lookup` — ~5 root descents and
   ~25–30 allocations per record, for a format that already stores `parent_slot` in
   pre-order. A direct arena fill using the existing `ids[]` vector, with one
   reverse-pass roll-up, removes all of it; expect ~5–10×, which is ~28% of today’s
   warm-start wall.
4. **Extension tallies allocate at every ancestor for every file.** `RollUp::merge`
   clones the extension `String` into a `BTreeMap` per level (`index.rs:123`), and
   `contribution()` builds a fresh one-node map plus a `derive_ext` string per file.
   On the realtree corpus (52k files, mean depth ~10, 119 extensions) that is ~523k
   `String` clones and ~523k string-keyed B-tree descents per cold scan — the dominant
   share of the 2.1 µs/entry apply cost.
   Interning extensions to a `u32` id turns the merge into integer adds.
5. **Paths are normalized and descended repeatedly per op.** `normalize()`
   (`index.rs:1204`) allocates a `Vec<OsString>` plus one `OsString` per component and
   is called 2–3 times per upsert (validate, arbitrate, apply); `apply` then re-descends
   from the root for the ancestor chain of every entry (`ensure_dir_chain`) even during
   a bulk scan whose walker just visited the parent.
   `Path::components()` yields borrowed `&OsStr` and needs no allocation; passing the
   parent `EntryId` down from the walker removes the descent.
6. **The memory layout is ~15–20× over target, measured flat at ~490–520 B/entry.**
   Every entry is a separate `Box<Entry>` behind a 24-byte arena tag (pointer-chasing,
   allocator overhead); every name is stored twice (`Entry.name` and the parent’s
   `BTreeMap` key — two heap copies of the same bytes); a 64-byte `RollUp` sits inline
   on every *file* where it is permanently zero; `EntryId` has no niche so
   `Option<EntryId>` costs 24 B; two `u64` revision counters spend 16 B/entry on ABA
   protection. An arena-packed entry (u32 parent, name as offset into a shared byte
   arena, roll-ups in a directory-only side vector, interned extension tallies) lands at
   ~40–60 B/entry; reaching the 25–32 B target additionally requires packing timestamps
   and dropping per-entry `dev`.

**What the live experiment loop has since established (exp-000 through exp-005).** The
loop has already tested parts of this analysis, and its verdicts sharpen it:

- **exp-001 (H1, accepted, `a0cc981`):** a bounded parallel producer halved cold scan
  (cold-scan-index wall 627 → 311 ms at 4 threads, digests identical).
  Finding 2’s cold-path half is landed.
  The cold critical path is now the *consumer*: cold-scan-index component (197 ms) sits
  almost exactly on cold-scan-producer component (192 ms), so further producer threads
  cannot help until apply gets cheaper or parallel.
- **exp-002 (H9 attempt, rejected):** parallelizing the *revalidation* sweep gained only
  2.6% at 60k entries on a warm-steady cache.
  This is consistent with the cost model, not against it: with the metadata cache warm
  there is no miss latency to hide (blocked time ~6 ms of ~800), and the per-entry
  expectation machinery in the single consumer dominates.
  The rejection is state- and scale-specific: the parallel sweep’s predicted wins are in
  purge-cold, over-capacity (the knee), and network-storage states the loop does not yet
  visit. Sequence flip: make the consumer O(changes) first (below), then re-test sweep
  parallelism where there is latency to hide.
- **exp-003 (H8, rejected):** removing ~120k bootstrap path clones changed nothing
  measurable. This corrects finding 3’s emphasis: individual small allocations are nearly
  free on this allocator at this scale; the paying work is *descents and extra passes*,
  not clone counts. exp-004/005 confirm from the other side — they removed structural
  work and won.
- **exp-004 (H5, accepted, `bf7a05a`):** borrowed path components — warm-revalidate
  −9.4% wall, snapshot load −17.8%, user CPU −18.6%. Finding 5’s normalize half is
  landed; parent-id passing remains open.
- **exp-005 (H10 partial, accepted, `954d27b`):** resolving each snapshot record through
  its parent id cut load component −31% (to ~165 ms ≈ 2.8 µs/record at 60k). Finding 3
  is partially landed; the remaining headroom is the single-pass CRC+parse, eliminating
  the per-record `Observation`, persisted roll-ups, and eventually the block format.
- **The loop’s headline defect (its H9) is architectural:** after exp-001,
  warm-revalidate (~790–820 ms) costs ~2.6× cold-scan-index (~310 ms) on the same tree —
  the cache currently costs more than it saves.
  The highest-order leverage section below is organized around closing exactly this.
- **The loop’s profile also hardened two blocked hypotheses:** after exp-001, `open` is
  28% and `fstatat` 19% of cold self-time — the two costs that only the dirfd/openat and
  bulk-stat work (its H2/H3) can remove.
  Both are blocked on a dependency-policy decision, addressed in Recommendations.

One scope caution when reading loop verdicts: every experiment so far is one 60k-entry
tree, one M1 Pro, warm-steady cache, unchanged-tree revalidation.
The knee at 500k+, cold-cache behavior, churned warm runs, network storage, and Linux
are all currently invisible to the loop, and at least one rejection (exp-002) is
predicted to flip in states it has not visited.

**The superlinear knee is probably capacity, not algorithm.** The exploratory curve (72
ms / 725 ms / 8.2 s / 62.9 s at 10k / 100k / 500k / 1M) has a local scaling exponent of
exactly 1.00 across the first decade, 1.51 to 500k, and 2.94 from 500k to 1M — while
peak RSS stays exactly linear at ~500 B/entry.
An O(n·depth) or O(n log n) algorithm cannot be perfectly linear for a decade and then
turn cubic over a doubling; a capacity being crossed at a fixed absolute size can, and
the corpus depth barely grows (8-ary tree: depth 4.5 at 100k, 5.6 at 1M). The prime
suspect is the macOS vnode/metadata cache — `kern.maxvnodes` on this host class is
~250–350k, squarely between the last linear point and the knee — with the serial sweep
converting every miss into a full stall of the only thread.
The review found no accumulating set, sort, or repeated full-tree pass in the reconcile
path (genuine algorithmic superlinearity is the last-ranked hypothesis), and the harness
records the discriminators cheaply: run the 100k/500k/1M points with `--repeat 2`
in-process, record `minor_faults`, `blocked_ns`, and `kern.maxvnodes`. If iteration 2 is
near-linear while iteration 1 keels over, the fix is the parallel sweep and syscall
batching — not index micro-optimization.
This experiment costs an afternoon and directs everything in Wave 3; it should run
first.

One evidence correction that belongs on the record: the
[reconciliation fast-path note](research-2026-08-09-reconciliation-index-fast-path.md)
states both reductions were kept, but `git show abeb377` shows the
capture-from-iteration optimization reached only the shared-handle path, not the
exclusive path the probe measures (finding 1 above).
The measured −18% improvement is real; the description of where it applies is not.

### What the Fastest Walkers Do — New Findings Beyond the Survey

The attic review of bfs, dut, pdu, diskus, and jwalk added detail the original survey
did not have, and three structural lessons:

**Output shape drives internal shortcuts — fdu must get the same effect at query time.**
dut never builds the full tree: per-thread top-N heaps reject entries *before
allocating* when they cannot beat the heap minimum, so memory is O(open dirs + N), never
O(files). pdu folds subtree sizes into the parent the moment depth exceeds `--max-depth`
and drops the children.
fdu retains the full index by design — so top-N, depth truncation, and friends must be
bounded *output stages* over pre-computed roll-ups, which is exactly what the
per-directory reducer state already enables.
The corollary: those tools’ headline numbers are for a smaller job, and the benchmark
capability matrix must keep saying so.

**Scheduling details that transfer directly:**

- bfs runs **one io_uring per worker thread** (a single shared ring was not
  competitive), SQ depth 64, with `SUBMIT_ALL`, `SINGLE_ISSUER` (via `R_DISABLED` +
  enable-on-worker), `DEFER_TASKRUN`, and `ATTACH_WQ` sharing one kernel io-wq pool
  capped at the thread count; unsupported opcodes fall back per-entry to synchronous
  calls on the same worker.
  Its MPMC queue uses unconditional `fetch_add` with skip-tagged slots and a 64-monitor
  futex pool — a documented, reimplementable design (`ioq.c:4-119`).
- bfs’s famous “8-thread cap” is **its own coordinator bottleneck, not a filesystem
  law** — the author says so; XFS scales metadata work near-linearly to at least 8
  threads while ext4 plateaus earlier.
  fdu should not inherit the constant.
- dut pushes a whole sibling batch with **one CAS** and wakes exactly
  `min(children, blocked)` workers; its hardlink tables live per-subtree near the leaves
  and merge upward (happy path: one CAS installing the child’s table into the parent),
  yielding a per-directory shared-vs-owned split almost for free — a good fit for fdu’s
  per-directory roll-up records, and a richer answer than a global seen-set.
  (GPL: reimplement from this description.)
- Thread-count policy has exactly two empirical data points in the wild: diskus defaults
  to **3× cores capped at 64** ("for cold disk caches, more threads help the IO
  scheduler plan ahead; for warm caches, too many add synchronization overhead") and pdu
  drops to **1 thread on rotational media**. Nobody adapts to measured conditions; both
  numbers are wrong on APFS (below).
- jwalk demonstrates the *cost* of strict streaming order under parallelism: a
  busy-waiting consumer and per-directory `IndexPath` Vec clones.
  Ordering belongs at output time, never in the walker — bfs’s unordered-by-default
  queues are the model.

**Two classic optimizations are unclaimed by every tool surveyed:** none uses macOS
`getattrlistbulk`, and none sorts entries by inode before statting.
Both are the top platform levers below; fdu can be first.

**Maintainer testimony on scheduling, collected from primary sources (annotated list in
the References), adds four warnings the surveys alone would miss:**

- **A mutex-guarded work queue caps scaling at zero.** ripgrep’s `ignore` walker sat
  behind an `Arc<Mutex<Vec<_>>>` and measured *flat* (~580 ms on Chromium) at 1, 6, 12,
  and 24 threads; replacing it with a crossbeam work-stealing deque gave 5.5× at 24
  threads (ripgrep PR #2591 — the core of fd v9’s 6–13× claim, with fd’s result-batching
  PR #1422 as the other half).
  fdu’s current parallel producer uses exactly the mutex-queue shape (`DirectoryQueue`,
  exp-001); fine at 4 threads, but the thread-curve experiments must expect this wall
  and the crossbeam-deque escalation is the known fix.
- **Naive `par_bridge` is a trap, and ordering costs memory.** Tavian Barnes measured a
  graph traversal at 34 s sequential, **16 m 44 s** under naive `par_bridge`, and 1.36 s
  with proper thief-splitting; BurntSushi diagnosed ripgrep’s 1 GB memory peak as
  *breadth-first order itself* (per-directory matcher state allocated for the whole
  frontier before any file completes, ripgrep #1550). Queue-vs-stack choice controls
  both ordering and peak resident state — one more reason fdu keeps ordering out of the
  walker.
- **Warm traversal parallelism saturates early even on Linux.** BurntSushi’s own
  measurement (ripgrep discussion #2472): raw warm-cache traversal of Chromium was
  fastest at 4 threads and *degraded* at 8; the optimum moves higher only when CPU-bound
  work (glob matching) rides along.
  Consistent with this document’s policy stance that worker count is a per-platform,
  per-workload measurement, not a constant.
- **Redundant syscalls can beat serialization.** Byron (dua, gitoxide) runs gitoxide’s
  untracked-files dirwalk *concurrently* with the index-to-worktree check, deliberately
  re-paying lstats that git’s serial `CE_UPTODATE` path avoids, and beats `git status`
  by 1.44× on WebKit: “with today’s machines, it’s often faster to just perform them
  anyway if it helps to run more in parallel.”
  He also flags M1 efficiency cores as a real skew in work-stealing pools — the QoS
  guidance above, observed independently.

### macOS: Batch Syscalls, Don’t Add Threads

**`getattrlistbulk(2)` is the single largest cold+warm lever on APFS.** One syscall
returns names plus attributes for as many entries as fit the buffer (hundreds at 64–256
KB), and the attribute set covers fdu’s entire fingerprint and both size metrics:
`ATTR_CMN_OBJTYPE`, `ATTR_CMN_FILEID`, `ATTR_CMN_MODTIME`, `ATTR_CMN_CHGTIME`,
`ATTR_FILE_TOTALSIZE` (logical) and `ATTR_FILE_ALLOCSIZE` (allocated bytes).
Evidence: Tempelmann’s benchmarks show it losing to bare `readdir` for names-only but
decisively beating `readdir`+`lstat` once attributes are needed — and fdu always needs
attributes; dumac (getattrlistbulk + 64-way bounded concurrency) measured **521 ms over
409,500 files vs 3,330 ms for Apple `du`** (6.4×) and 1,342 ms for diskus, with 91% of
remaining time in the kernel.
One dissent to design the spike around: Tempelmann’s filesystem-dev follow-up found
APFS’s `getattrlistbulk` implementation less efficient than HFS+’s (with `fts` fastest
for local names-only walks in his 2019 data, pre-Apple-Silicon), while dumac’s 2025
M-series numbers show bulk winning decisively — so the H26 spike must A/B bulk against
`fts`/readdir+`fstatat` on modern APFS rather than assume, and the per-filesystem API
choice belongs in the policy layer either way.
Apple’s own modern `FileManager` rewrite in swift-foundation uses `fts` instead — i.e.
even Apple’s current tooling leaves this on the table.
Implementation cautions, all documented in the field: buffers ≥ 64 KB (Apple DTS calls
small buffers a bug factory), per-entry `ATTR_CMN_RETURNED_ATTRS` checking, the
ERANGE-after-exactly-full-buffer quirk, drain-one-directory-fully-then-descend (DTS
guidance; also avoids a Sequoia SMB relisting loop), and process-wide
`setiopolicy_np(IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES, …, OFF)` so iCloud dataless
placeholders are never materialized by a scan.

**APFS parallelism plateaus early; batching does not.** APFS metadata operations funnel
through per-volume B-tree locking (Szorc measured parallel walks *adding* kernel time in
2018; Apple engineers acknowledged high-core-count contention concerns as recently as
2025). dumac found goroutine-per-directory parallelism *slower* than batched syscalls
and settled on a bounded semaphore of 64 in-flight directory operations.
Design consequence: on macOS, worker count is a second-order tunable (≈ P-core count,
tens of in-flight directories), and syscall batching is first-order — the mirror image
of Linux.

**The FSEvents persistent journal can make warm runs O(changes) instead of O(n).**
fseventsd journals directory-level change events to disk, surviving reboots; a run can
persist its last `FSEventStreamEventId` plus the volume UUID and, on the next open,
replay “which directories changed since event X” instead of sweeping a million stats.
A source-level read of the two production precedents (below) sharpens the claim:
Watchman’s `fsevents_try_resync` proves the *mechanics* — resume from a recorded event
ID guarded by a `FSEventsCopyUUIDForDevice` equality check and an `EventIdsWrapped` veto
— but uses them only for **in-process** recovery after dropped events, off by default
(briefly defaulted on, then reverted in December 2021 “due to possible correctness
issues”); across restarts both Watchman and git’s fsmonitor daemon start at `SinceNow`
and force a fresh crawl through their own logical clocks.
Cross-restart replay is therefore Apple-documented and API-supported but unproven in
major production tools — fdu would be pioneering it, which makes the backstop
non-negotiable rather than merely prudent.

Implementation findings from the follow-up spec work (2026-08-10), recorded here so the
platform picture stays in one place: the run-loop scheduling every older example uses
(`FSEventStreamScheduleWithRunLoop`, including `notify` 8.2’s backend) is deprecated
since macOS 13; the supported path is `FSEventStreamSetDispatchQueue` (available since
10.6), which also suits one-shot historical replay better — no parked run-loop thread,
no cross-thread stop.
`fsevent-sys 4.1.0` is already in `Cargo.lock` via `notify`, so a journal-resume
implementation adds zero new crates; the two functions it leaves undeclared
(`FSEventStreamSetDispatchQueue`, `FSEventsCopyUUIDForDevice`) are self-declared
externs, with the generated `objc2-core-services` bindings verified complete as the
fallback route. H43 is now specced with a validation spike as its first phase:
[plan-2026-08-10-fdu-fsevents-scoped-revalidation.md](../specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
(beads `fdu-2cdv` → `fdu-hs10`). The validation ladder is exactly fdu’s existing
escalation shape: UUID mismatch, event ID regression,
`kFSEventStreamEventFlagEventIdsWrapped`, or `MustScanSubDirs`/dropped-event flags each
map to `InvalidateSubtree` (scoped or root) and the sweep runs as the backstop; Apple
documents the journal as advisory, so a periodic paranoia sweep remains.
Nothing in the du-tool space does this; it converts fdu’s warm-open story on macOS from
“fast sweep” to “no sweep,” and it reuses the delta contract unchanged.
Linux has no equivalent (confirmed below), which means the sweep must be fast there
regardless — the two investments are complements, not alternatives.

**Scheduling and measurement details:** Rust threads get `QOS_CLASS_DEFAULT` (P-core
eligible — fine), but a CLI wanting max throughput should set worker QoS to
`USER_INITIATED`, and a resident watcher to `UTILITY`; never `BACKGROUND`, which pins to
E-cores and throttles I/O. `sudo purge` approximates a cold filesystem cache but does
not fully drop vnode/name caches; remounting a dedicated APFS test volume is the strict
cold protocol, and the evidence records should distinguish purge-cold from remount-cold.
Accounting wrinkles to encode in oracles rather than discover in bug reports: APFS
clones make allocated-size roll-ups non-additive (pnpm/uv trees can overstate 10–100×
naively), sparse files and decmpfs compression make logical and allocated diverge in
both directions — carrying both sizes per entry (already the design) is correct, and the
docs should say per-directory allocated sums can exceed `df` truth under cloning.
Finally, **APFS fast directory sizing** (`dirstat_np`, `INODE_MAINTAIN_DIR_STATS`) is
confirmed dead in practice — Apple never wired it up, Finder declined to adopt it, and
it cannot be enabled retroactively on existing directories.
fdu’s persistent roll-up cache is precisely the feature Apple abandoned; there is no
kernel shortcut to steal.

### FSEvents From Rust: What notify Can and Cannot Carry

A dedicated source review (notify 8.2.0 and 9.0.0-rc.4 under `attic/notify`, fsevent-sys
4.1.0, objc2-core-services 0.3.2, Watchman’s and git fsmonitor’s FSEvents watchers)
settles how the journal-resume module should be built:

- **notify — fdu’s pinned live-watch backend — cannot express journal resume, in any
  version.** The 8.2.0 backend receives per-event IDs and discards them (`_event_ids`,
  `fsevent.rs:532`); `since_when` is a private field hardcoded to
  `kFSEventStreamEventIdSinceNow` (`fsevent.rs:66,299`); `HistoryDone` is recognized and
  swallowed without emitting anything (`fsevent.rs:108-110`); `EventIdsWrapped` is
  declared but never checked; scheduling still uses the deprecated
  `FSEventStreamScheduleWithRunLoop` (with an open initialization-deadlock report,
  notify #942). The 9.0 RC line migrates to objc2 bindings (PR #726) but changes none of
  this, and upstream has *never discussed* exposing event IDs or `sinceWhen` — no
  rejected proposal, no in-flight work.
  What notify does surface — `MustScanSubDirs`+dropped flags as `Flag::Rescan` — is
  exactly what fdu’s watch layer already consumes, so notify remains the right live
  backend; resume is simply outside its model.
- **One live hazard at the seam:** notify’s shutdown path calls
  `FSEventsPurgeEventsForDeviceUpToEventId` (8.2.0 `fsevent.rs:487-489`), which
  truncates the device’s **on-disk journal**. A resume token persisted *after* the live
  watcher stops may point at purged history.
  (Apple documents the purge call as root-only, so the hazard is sharpest for
  root/daemon processes — but the ordering rule is cheap insurance regardless.)
  Ordering rule: persist the resume token before stopping the watcher, and treat a
  failed resume as an ordinary fall-back-to-sweep, never an error.
- **The binding for a first-party resume module is `objc2-core-services`** (features
  `FSEvents`, `libc`, `dispatch2`). It is the only current binding with the complete
  surface: `FSEventsCopyUUIDForDevice` (the UUID validation call — literally commented
  out of fsevent-sys, which also lacks `FSEventStreamSetDispatchQueue` and has
  deprecated itself in objc2’s favor), `FSEventStreamCreateRelativeToDevice`
  (volume-scoped streams, matching the per-volume sharding the whole-drive design
  already requires), and the non-deprecated dispatch-queue scheduling.
  notify 9.0 itself migrated to this stack, so fdu would converge on one binding family.
  One generator gap to handle locally: the extended-data dictionary keys (`"path"`,
  `"fileID"`) are C string macros the bindings don’t emit — define them in the module.
  The dependency addition (objc2-core-services, objc2-core-foundation, dispatch2;
  macOS-only, unsafe confined to generated externs) goes through the supply-chain
  process and cool-off like any other.
- **Reference configurations from production:** Watchman and git both run
  `NoDefer | WatchRoot | FileEvents` with plain path arrays; git hardcodes latency 0.001
  s (empirically tuned against event drops — 0.1 s dropped events under a 100k-file
  storm), Watchman defaults 0.01 s configurable; git switched to
  `FSEventStreamSetDispatchQueue` in v2.40 precisely because the runloop API is
  deprecated; `FSEventStreamSetExclusionPaths` is capped at 8 paths by the API (Watchman
  uses it; git filters in the callback).
  `kFSEventStreamCreateFlagUseExtendedData` — which delivers the file **inode** with
  each event, joining replay events directly against fdu’s inode-bearing fingerprints —
  is used by neither and is a genuine improvement available to fdu.
- **The resume module’s shape** (macOS-only, feature-gated, ~one file): at snapshot
  save, persist `(volume UUID via FSEventsCopyUUIDForDevice, event ID)` per volume; at
  open, re-derive the device, re-check the UUID (mismatch or null ⇒ full sweep), create
  a device-relative stream at the persisted ID with
  `FileEvents | NoDefer | UseCFTypes | UseExtendedData | FullHistory`, schedule on a
  private serial dispatch queue (batch ordering is what makes token persistence sound),
  collect `(path, inode, flags, id)` until `HistoryDone` (detected by flag — its
  accompanying path is garbage), with a bounded wait; any dropped/`MustScanSubDirs` flag
  scopes an `InvalidateSubtree` (the dropped flags always accompany `MustScanSubDirs`,
  so checking it alone suffices), `EventIdsWrapped` abandons replay entirely, and the
  new resume token — the max of the per-event IDs captured *inside* the callback, not a
  cross-thread `GetLatestEventId` read — is persisted only after the corresponding
  deltas are applied. `FullHistory` (macOS 10.15+) is load-bearing, not optional: Apple’s
  header documents that without it, events near the sinceWhen boundary can be **silently
  skipped** because history is stored in coalesced chunks; with it, replay is
  overlapping and at-least-once, which fdu’s idempotent deltas absorb by design.
  Journal availability caveats to encode in the validation ladder: a NULL UUID means no
  history exists (read-only volumes); a volume can opt out entirely via
  `/.fseventsd/no_log`; FAT32/exFAT journals are unreliable; retention is bounded and
  unspecified (days to weeks on busy volumes, flushed at major OS upgrades) — the UUID
  match plus successful replay-through-HistoryDone is the only proof the journal served
  the request. `IgnoreSelf`/`MarkSelf` have no effect on historical events, so replay
  includes fdu’s own past writes (harmless: the cache lives outside scanned roots).
  TCC nuance: events can arrive for paths the process cannot stat — reconciling a
  flagged directory under `~/Library` etc.
  surfaces as ordinary partial errors without Full Disk Access, and the behavior is not
  Apple-documented, so it belongs in per-macOS-release tests.
  Threading rules copied from notify’s proven pattern: context via `Box::into_raw` freed
  in the stream’s release callback, the raw stream pointer in a Send wrapper, stop →
  invalidate → release in that order and never from the callback thread, no panics
  across the FFI boundary, and `OsStr::from_bytes` for paths (notify 8.2 panics on
  non-UTF-8 — a bug not to copy).

### Linux and the Cloud: Hide Latency, Order the Work, Trust Nothing Optional

**The planned `getdents64` + dirfd-relative `statx` layer stays the backbone** — nothing
found changes that — with `STATX_BASIC_STATS` as the exact right mask (size, mtime,
ctime, ino, dev, blocks all included; bfs’s extra `STATX_BTIME` is not needed for the
fingerprint) and `AT_STATX_DONT_SYNC` added for network filesystems.

**io_uring is a bonus, never the plan.** Verified against current mainline: there is
still no getdents/readdir opcode (three patch series died over `f_pos` locking), so
enumeration stays synchronous even on the io_uring path; only openat/statx/close batch.
And availability in fdu’s stated cloud targets is mostly *no*: Docker ≥ 25 and
containerd ≥ 2.0 block the io_uring syscalls in their default seccomp profiles
(motivated by Google’s finding that 60% of 2022 kernel-exploit bounties targeted
io_uring), gVisor (Cloud Run, GKE Sandbox) disables it, RHEL 9 ships `io_uring_disabled`
by default, and kernel 6.6 added the global kill switch.
Raw EC2/GCE VMs and self-hosted runners allow it.
So: runtime probe, per-opcode fallback (bfs’s exact pattern), and the thread-pool path
is the one that must be fast.
This retires the Phase 1 open question: io_uring is a post-Phase-1 accelerator behind a
feature flag (`fdu-ktka`), correctly sequenced.

**Inode-ordered statting is the highest-ROI cold-cache technique, and it helps network
storage most.** ext4 returns dirents in htree hash order, deliberately uncorrelated with
inode-table layout; statting in that order makes every inode lookup a random block read.
Sorting a directory’s entries by `d_ino` before statting turns N random inode-table
reads into ~N/16 mostly-sequential ones (256-byte inodes, 16 per 4 KiB block) — measured
at 4–6× cold in prior art (mutt’s maildir code, borg’s issue tracker, LKML guidance from
Ted Ts’o). The reason this matters for fdu’s *targets* specifically: on local NVMe the
win shrinks (latency is small and parallelism hides it), but on network block storage
every avoided read is a saved ~0.5 ms round trip against a 3000-IOPS budget — access
*ordering* is how you go fast on EBS, because you cannot buy latency down.
It costs one sort per directory, needs no privileges, and is gated cleanly per
filesystem via `statfs` `f_type`: valuable on ext4, mostly redundant on XFS (its readdir
does internal readahead and correlates with inode order), meaningless on overlayfs
(synthetic inode numbers) and on APFS (where getattrlistbulk removes the per-entry stat
entirely). There is no userspace readahead for directory metadata —
`posix_fadvise`/`readahead` apply to file data only — so ordering plus concurrency are
the *only* cold-cache levers; this is the complete answer to “can we read filesystems in
a more optimized order.”

**Queue depth is storage arithmetic, not core count.** Cold metadata scanning is
small-random-read I/O: EBS gp3 (~0.5 ms, 3000 IOPS baseline) saturates at ~4–8 in-flight
operations — beyond that you are IOPS-capped and extra threads only add contention;
local NVMe serves 16–64; a cgroup-limited container must size its pool from the CPU
quota (`std::thread::available_parallelism` already accounts for cgroup quotas;
`num_cpus` does not) or the whole pool bursts and then sleeps out the CFS period.
A single huge directory cannot be enumerated in parallel (the dirent stream serializes
on `f_pos`, and ext4 offsets are opaque hash cookies), but enumeration and stat pipeline
naturally: one getdents drainer feeding inode-sorted stat batches to the pool.
Benchmark hygiene from the same physics: record volume type and burst-credit state
(gp2’s 30-minute credit bucket makes fresh volumes measure the bucket, not the disk),
prefer gp3, and run the matrix on both ext4 and XFS since their parallel-metadata
scaling diverges.

**Watching at scale has a real answer now, but it is privileged.** fanotify with
`FAN_MARK_FILESYSTEM` + `FAN_REPORT_DFID_NAME` (kernel ≥ 5.9; `FAN_RENAME` with both
names ≥ 5.17) watches an entire mount with one mark and reports create/delete/rename
*with parent handle and entry name* — the events fdu’s delta contract wants — but
requires `CAP_SYS_ADMIN` (unprivileged fanotify cannot take mount/filesystem marks), and
resolving handles needs `CAP_DAC_READ_SEARCH`. inotify remains the unprivileged path and
its costs at scale are now quantified: ~1 KB kernel memory per watch, a default
`max_user_watches` ceiling of ~1M that a 10M-file tree’s ~700k directories approaches,
and a reported ~35 s recursive arming crawl at that size.
Overflow in either backend maps to the existing `InvalidateSubtree` escalation.
And the structural fact, now confirmed: **Linux has no persistent change journal** — no
USN, no fseventsd; fanotify is live-only, and the only changed-since query anywhere is
btrfs’s root-only `find-new` (generation numbers), which misses deletions.
fdu’s snapshot + revalidation design is therefore not a workaround on Linux; it is the
only possible architecture, and the parallel sweep is its hot path.

**Why no Linux journal exists, and the partial analogs that do.** The absence is policy,
not oversight: a persistent change journal taxes every write so occasional readers can
replay history — NTFS and APFS accepted that tax at the OS level; the Linux answer to
“changed since” has consistently been *live* delivery (the superblock-watch patches
became fanotify’s filesystem marks), and ext4’s jbd2 journal is crash-consistency
machinery with seconds of retention and no API. So the “no journal” statement is
precisely an ext4/XFS statement, and four analogs deserve the record:

- **CoW snapshot diff (btrfs, ZFS) — the one real candidate accelerator.** `zfs diff`
  and `btrfs send --no-data -p old new` yield a *complete* change set — creates,
  deletes, and renames included, unlike `find-new`, which misses deletions.
  The cursor design maps exactly onto the FSEvents plan’s shape: at snapshot save,
  create a cheap read-only CoW snapshot (the snapshot *is* the cursor); at open, diff
  against it, revalidate only named paths, rotate the snapshot; same
  gate-and-fall-back-to-sweep structure.
  Caveats that keep it an opt-in niche: requires btrfs/ZFS (some desktops and NAS,
  rarely CI), privileges for send/diff, and subvolume-scoped roots (H47).
- **fdu’s own delta journal is the honest general answer.** No production tool resumes
  Linux watches across restarts because there is nothing to resume from — Watchman
  recrawls on restart here too.
  But fdu’s planned append-only journal of applied deltas (cache option B) *is* a
  persistent change log — valid for exactly the periods a watcher was alive, degrading
  to the sweep for the gaps.
  On Linux, rung 2 is something fdu builds rather than something the OS provides.
- **The pattern is proven where vendors control the whole stack.** Lustre’s persistent
  ChangeLog feeds Robinhood Policy Engine — scan once into a database, then consume the
  changelog forever, which is fdu’s architecture at HPC scale — and NetApp SnapDiff /
  OneFS changelists are the NAS equivalents.
  Useful precedent; none of it reaches local ext4.
- **`STATX_CHANGE_COOKIE` (kernel ≥ 6.6)** is a per-inode change counter: it cannot beat
  the sweep (still one statx per entry) but hardens fingerprints against timestamp
  forgery and granularity races at zero cost inside the existing mask.

Rejected for the record: dm-era/LVM thin-snapshot deltas (block-level; mapping blocks to
files requires parsing the filesystem), auditd or eBPF write-logging (a resident
privileged daemon with heavier overhead — strictly dominated by fdu’s own watcher), and
the ext4 journal itself (no retention, no API).

**Overlayfs (CI rootfs) is a distinct worst case:** readdir re-merges every layer per
open directory stream, lookups probe layers top-down, `st_ino`/`d_ino` are synthetic
unless xino applies (weakening both inode sorting and inode-based identity — record
`st_dev` transitions carefully), and copy-up changes file identity behind the walker.
Workspace checkouts are usually bind mounts (no penalty); it is scans of the container
rootfs that pay. Treat overlayfs as: portable walker semantics, ordering disabled,
fingerprints conservative.

### Traversal as Policy: Adapting to Filesystem, Storage, and Cache State

The findings above converge on a small decision layer rather than per-platform walkers
scattered through the code.
Mechanism (three walker backends: getattrlistbulk on macOS; getdents64+statx on Linux;
portable std elsewhere and as fallback) stays separate from **policy** — a per-run,
per-device selection of:

- **Stat ordering:** inode-sorted on ext4/btrfs; native order on XFS/APFS/overlayfs.
- **In-flight depth:** from storage class — network block storage ~4–8, NVMe 16–64, APFS
  a bounded few-dozen directory operations, rotational 1 (safe fallback only, not an
  optimization target).
- **Worker count:** from effective CPU quota and platform (Linux scales with threads;
  APFS does not).
- **Traversal order:** DFS for warm locality, breadth-biased fan-out for cold queue
  depth — dut’s README documents the trade; no surveyed tool adapts it.
- **QoS/priority:** platform-appropriate (macOS QoS classes above).

**Robust detection: probe behavior, then identify, then measure — in that order.** The
way to get platform wins without hyper-customization is a three-tier detection design
where each tier is used only when the one before it cannot answer:

1. **Probe the operation, not the platform.** Every optional primitive is detected by
   attempting it once at open and caching the verdict for the run: one `getattrlistbulk`
   call on the root directory (ENOTSUP/EINVAL → getdents or portable backend), one
   `statx` (ENOSYS → `fstatat`), one io_uring setup (EPERM/ENOSYS — the common seccomp
   case — → thread pool), one fanotify mark (EPERM → inotify).
   bfs’s per-opcode probing with per-entry synchronous fallback is the proven model, and
   fdu already has in-repo precedent: `corpus_cache.py` runs a live
   copy-on-write-independence probe for `clonefile`/`FICLONE` instead of trusting the
   filesystem name. Behavior probes cannot be wrong about the thing that matters, survive
   containers and seccomp filters that lie about kernel versions, and cost microseconds.
2. **Identify the filesystem only for the small residue behavior cannot cheaply
   reveal**, per mount: `statfs` `f_type` on Linux, `f_fstypename` plus `getattrlist`
   volume capabilities on macOS. Exactly three decisions hang off the name table, all
   fail-safe: inode-sort gating (on for ext4/btrfs; off for XFS, APFS, overlayfs, and
   *unknown*), network-filesystem handling (NFS/SMB/FUSE → `AT_STATX_DONT_SYNC`, polling
   watch backend, low concurrency — the policy metabrowser already ships in
   `watch_backends.py`), and overlayfs conservatism (synthetic-inode identity, ordering
   off). An unrecognized filesystem gets portable defaults; identification failure is
   never an error.
3. **Measure the hardware instead of classifying it.** Static classification is where
   hyper-customization creep starts, and it does not even work: on EC2 Nitro, EBS and
   local instance store both appear as non-rotational NVMe devices, distinguishable only
   by model-string sniffing (“Amazon Elastic Block Store”) — exactly the kind of vendor
   table to refuse. Instead, calibrate: time the first K metadata operations of the
   actual walk; the observed latency distribution sets in-flight depth by Little’s-law
   arithmetic, and it simultaneously classifies cache state (µs-scale reads mean warm
   cache, tens of µs mean cold local SSD, ~ms means cold network storage) — one probe
   answers both axes, replacing the static depth table with a measured prior.
   The static table (network ~4–8, NVMe 16–64, APFS few-dozen) remains only as the
   starting point and the fallback; an optional adaptive controller (AIMD-style: grow
   in-flight depth while marginal throughput improves) is the fully general version, at
   the cost of harder-to-reproduce benchmarks — the experiment below measures whether
   the simple one-shot calibration captures most of the win.

Policy is resolved **per mount**, re-evaluated at `st_dev` boundary crossings the walker
already tracks — one tree spanning APFS, an SMB mount, and a FUSE volume gets three
policies, not one compromise.
Probes run once per run, sub-millisecond, never per directory; the snapshot (already
keyed by root) may carry the previous run’s calibration as a warm prior, never as truth.
Anti-patterns, named so they stay out: kernel-version sniffing (containers and backports
make it a liar), filesystem allowlists beyond the three-decision table, vendor/model
string matching, and any persistent hardware-keyed tuning database.

No surveyed tool does runtime calibration; the closest prior art is diskus’s static
3×-cores oversubscription and pdu’s static rotational check — both are the kind of
host-tuned constant the measured approach replaces.

This is also where the composable-surface work connects to performance, in one rule:
**scope and view parameters must never select a different engine variant.**
Tag-don’t-prune keeps one index valid across all scopes; depth/top-N are output
truncations over roll-ups; formats are renderers over one result model.
The benchmark matrix then scales with jobs × states, not × product surface.

### Change Propagation: What Filesystems Refuse to Tell You

The entire cache design follows from one fact about every mainstream filesystem (ext4,
XFS, btrfs, APFS, NTFS — POSIX semantics throughout): **change information propagates
upward at most one level, and for the most common change, zero levels.** Precisely:

- A **namespace operation** — creating, deleting, or renaming an entry — updates the
  mtime/ctime of exactly the containing directory.
  Grandparents observe nothing.
- A **content edit** updates only the file’s own mtime/ctime — *not even its immediate
  parent’s*. Directory mtime describes the name list, not the files the names point to.
  (Verified empirically on this host’s APFS during the FSEvents spec work: an in-place
  append changed no ancestor directory’s mtime; a create changed exactly one level;
  recorded as refuted hypothesis H47 in the loop guide.)
- A **metadata change** (chmod, chown) updates only the file’s ctime.

No mainstream filesystem maintains recursive modification times or recursive sizes.
The one that tried — APFS fast directory sizing — never worked and was abandoned by
Apple (see the macOS findings above).
This single fact has five design consequences, and together they *are* the cache
architecture:

1. **A directory fingerprint can prove membership, never content.** A matching (mtime,
   ctime) on a directory proves no entry was added, removed, or renamed in it.
   It proves nothing about any child’s bytes or metadata, one level down or fifty.
   This is guardrail G1 stated as physics rather than policy: any cache that revalidates
   by statting only directories silently misses every in-place edit — the classic naive
   du-cache bug.
2. **Therefore the trustworthy warm floor is one stat per entry, not per directory** —
   and beating that floor requires information the filesystem’s namespace does not
   carry. There are exactly three sources: brute force (the parallel sweep, rung 1), an
   OS change journal (FSEvents/USN/btrfs generations, rung 2), or a live watcher (rung
   3). This is why the warm ladder has the shape it has, and why on Linux/ext4 — which
   has no journal — the sweep speed is not optional.
3. **What a matching directory fingerprint does buy is skipping *enumeration*, and the
   cache should be organized to exploit it.** The snapshot already knows the child
   names; if the directory fingerprint matches, the sweep can stat each known child
   directly and skip `read_dir` entirely (git’s untracked-cache design — H15). Two
   riders: on macOS this is worthless once getattrlistbulk lands, because bulk
   enumeration *is* the stat pass; and the skip must be gated per-filesystem
   (coarse-timestamp filesystems, network mounts, and overlayfs get conservative
   treatment).
4. **The index is the recursive layer the filesystem refuses to be.** fdu’s
   per-directory roll-ups and clocked deltas reconstruct exactly the upward propagation
   the kernel does not perform — which is why they must be *persisted* (H33):
   recomputing them on every load re-pays the cost the filesystem never waives, while
   storing them makes an unchanged subtree free forever.
5. **Timestamp trust has known failure modes with known fixes.** Equal-second
   granularity on coarse filesystems (FAT: 2 s; HFS+: 1 s; NFS: server-dependent) and
   same-instant modifications make a fingerprint *racily clean*: a file changed within
   the same timestamp tick as the cached stat is indistinguishable from unchanged.
   Git’s rule transfers directly: a fingerprint whose mtime equals the snapshot’s
   capture time at the filesystem’s granularity is treated as suspect and re-verified,
   and the snapshot must record the per-filesystem timestamp granularity it was captured
   under. mtime is also user-settable (`touch -t`), which is why the fingerprint includes
   kernel-controlled ctime and the inode (borg/restic’s lesson, already the design).

**Verification floors by metric tier.** Working the same fact through each class of
roll-up gives a clean rule, because different reducers depend on different filesystem
state:

| Reducer tier | Depends on | Cheapest sound verification | Dir-fingerprint pruning? |
| --- | --- | --- | --- |
| Name tier — file/dir counts, extension tallies, tree shape | Namespace only | Stat **every directory** (D stats; no file stats) | Yes — per directory, exact |
| Stat tier — sizes, mtimes, allocated bytes | Per-file inode attributes | Stat **every entry** (N stats) | No — in-place edits are invisible to every directory fingerprint |
| Content tier — words, lines, hashes | File bytes | N stats, then re-**read** only entries whose fingerprints changed | No for the sweep; yes for the expensive part — unchanged files are never re-read |

Three corrections to the intuitive reading, each load-bearing:

- Name-tier pruning is per-*directory*, never per-subtree: a deep namespace change bumps
  only its immediate parent, so every directory in the subtree must still be
  fingerprint-checked (D stats, not 1). Fingerprint checks do not compose upward; only
  journal or watcher information (rungs 2–3) can skip whole subtrees.
- Size roll-ups are precisely the ones directory fingerprints can *not* protect — the
  most common change in a working tree (an in-place edit) changes a file’s size and no
  directory’s mtime.
- The cache shortcut is *strongest* at the content tier, not absent: the stat
  fingerprint is the identity proxy for content, so revalidation costs the same N-stat
  sweep as the stat tier plus re-derivation for changed files only.
  What no tier can do is drop below its stat floor without rung-2/3 information.

A corollary that prevents a whole class of tempting non-optimizations: **within a tier,
attributes are free; only crossing a tier boundary changes the cost.** One `statx`/bulk
record returns size, mtime, ctime, and inode together, so a sizes-only view (plain disk
usage) verifies at exactly the same cost as sizes-plus-newest-mtime — dropping timestamp
metrics from a stat-tier view saves nothing, because sizes alone already force the
N-stat sweep. The discrete jumps are: name-tier only → D stats; any stat-tier metric → N
stats; any content-tier metric → N stats plus changed-file reads.
To make a pure disk-usage view faster than N stats, the only sound routes are rung 2–3
change information (skip unchanged subtrees entirely) or the labeled H44 mode.

The design consequence for the reducer registry (`fdu-a6dz`): **each reducer declares
its dependency tier, and the engine derives the cheapest sound verification from the
reducer set a view actually uses.** A counts-only query runs the D-stat sweep; a size
query runs the N-stat sweep; a content query adds changed-file reads.
This is the deepest connection between the composable-views surface and the cost model:
view selection changes verification cost by integer factors while remaining exact, with
no trust label needed.
A *labeled* mode (H44) is then only required when a caller wants stat-tier answers at
name-tier cost — serving possibly-stale sizes under an explicit staleness label (Goal 7:
fast-but-wrong is a non-goal; fast-and-labeled is a feature).

### Warm Runs: A Ladder With Three Rungs

1. **Parallel stat sweep (the floor, all platforms).** Load snapshot, sweep
   fingerprints, emit deltas.
   Floor is pool 1 arithmetic: ~0.2–0.4 s per million entries on Linux at 8 threads; on
   macOS the sweep should itself use getattrlistbulk (batch-verify a whole directory in
   one syscall against the snapshot’s children — which the directory-at-a-time reconcile
   structure already matches).
   Requires findings 1–2 fixed (allocation-free expectations, parallel sweep) to reach
   the floor.
2. **Journal-assisted revalidation (macOS today; Windows later; never Linux).** FSEvents
   `sinceWhen` resume reduces the sweep to changed directories plus the validation
   ladder; quiet trees revalidate in milliseconds regardless of size.
   The snapshot format needs two new fields (event ID, volume UUID) reserved now so the
   format does not break when this lands.
   Why this rung must be an *operation log* and can never be a timestamp query:
   filesystems index by name, not time (`find -mmin` is itself a full N-stat walk), and
   even a hypothetical mtime-since-T index could not work — deletions have no mtime, so
   “changed since T” (modified ∪ created ∪ deleted) is answerable only from a log that
   records operations, which is what FSEvents and USN are.
   (macOS Spotlight can serve time-indexed queries but its coverage exclusions and lag
   make it at best an unverified hint source; the FSEvents journal strictly dominates
   it.) Calibration: with a warm metadata cache the rung-1 parallel sweep at 1M entries
   is already ~0.2–0.4 s, so the journal’s transformative cases are cold caches (cloud
   hosts that cannot hold the inodes in RAM — minutes on network storage) and very large
   N, where it is minutes versus milliseconds.
3. **Resident watch mode (all platforms, already built).** The index stays perpetually
   fresh; the performance frontier moves to per-event constants and escalation rarity —
   next section.

The production magnitude of this ladder is now well documented by git’s builtin
fsmonitor (the same inversion: OS names the changes, verify only those): on Chromium
(393k files) `git status` fell from 17.6 s to 0.827 s, the lstat sweep alone ~500×, and
a synthetic 2M-file repo went 85.1 s → 0.75 s. Equally instructive is the crossover Ben
Peart measured in the original series: at 3k files warm, event-driven invalidation was a
*regression* (0.05 s → 0.24 s — the hook/daemon round trip cost more than the stats it
saved). Rung selection is size-dependent, and small trees should just sweep.

The rungs degrade gracefully into each other, and every rung ends at the same place:
correctness never depends on anything above rung 1.

### Content-Tier Metrics: Where the Cache Pays Most

The tier table shows content metrics (lines of code, words, paragraphs — including
custom plugin analyzers) have the same N-stat verification floor as sizes, but their
*re-derivation* cost is reading and parsing file bytes — gigabytes and minutes for a
large repository. That inverts the cache’s economics: for stat-tier roll-ups the cache
saves syscalls (decisive when cold, at drive scale, or via journal resume; modest on a
warm laptop), while for content-tier roll-ups it saves nearly everything on every rerun.
A repository summary that took minutes cold takes seconds warm: N stats plus re-reading
only the files whose fingerprints changed.
The fingerprint rule is git’s: size + mtime + ctime + inode unchanged ⇒ content presumed
unchanged, with the racily-clean guard (G5) — and this is unclaimed competitive ground,
since scc and tokei, the best-in-class content counters, cache nothing and recompute
every run.

Design consequences, extending the original research’s
`(fingerprint, analyzer id, analyzer version)` cache key:

- **The derived-data cache is a separate, additive layer, not part of the core
  snapshot.** The core snapshot must stay small and fast to open; per-analyzer results
  load lazily, accumulate as new analyzers run (“every run with more extensive roll-ups
  saves that data”), and are independently purgeable and size-bounded.
  Tens of bytes per file per analyzer: a 1M-file tree with several analyzers costs tens
  of MB.
- **Per-directory content roll-ups persist per analyzer too**, so an unchanged subtree
  contributes its cached aggregate without touching per-file records — the same duc
  lesson applied one tier up.
- **Analyzer identity is part of the key.** A plugin analyzer’s version (or rule-set
  hash) invalidates only its own column, never the tree truth; analyzers must be pure
  functions of content for the memoization to be sound (flowmark’s cache-lifecycle
  discipline).
- **Hardlinks derive once for free** (same inode ⇒ same fingerprint ⇒ shared cache
  entry); APFS clones (different inodes, same extents) are missed sharing and merely
  cost a duplicate derivation.
- **Content work parallelizes embarrassingly** — unlike the stat sweep it is
  CPU-and-read bound (scc/tokei’s pipelines), so E-cores add real throughput and the
  APFS metadata-lock ceiling does not apply.

This is backlog item H46: a spike with one cheap analyzer (line counts) over a real
repository, measuring cold derive, fingerprint-cached rerun, and 1%-churn rerun against
scc/tokei cold-every-time as references — validating the derived-store shape before the
reducer registry (`fdu-a6dz`) freezes interfaces.

### The Motivating Use Case: Whole-Drive Usage on a Mac

The scenario that exercises every rung at once, and the clearest product win available:
full-disk usage of a large internal drive (~2–5M entries), where dust takes 30–60
minutes and therefore cannot be part of anyone’s working loop.
Target shape: first scan in single-digit minutes (cold, unavoidable), every subsequent
check in seconds — because the FSEvents journal names what changed in the intervening
hour and everything else is served from persisted roll-ups.

Why the incumbent is as slow as it is, and what a correct whole-drive walk must do
differently, are mostly the same list:

- **One filesystem at a time, deliberately.** `/Volumes` network mounts and external
  disks should never be swept into an internal-drive scan; device-boundary stops are
  already the design. The System/Data firmlink split makes parts of the tree reachable
  twice — dedupe by `(dev, ino)` and treat firmlink crossings as boundaries, or the
  drive is partially double-counted and double-walked.
- **Never materialize dataless files.** iCloud/App Store placeholders must stay
  placeholders (`setiopolicy_np(… MATERIALIZE_DATALESS … OFF)`); a naive walker can
  trigger downloads, which is both slow and destructive of the user’s intent.
- **Expect permission structure.** Whole-drive scans need Full Disk Access to see
  `~/Library`, Mail, and friends; SIP-protected paths error regardless.
  Partial-error handling is a first-class outcome at drive scope (already the contract),
  and the errored subtree count belongs in the output.
- **Report both size metrics and label the caveats.** At drive scope, APFS clones
  (allocated sums can exceed physical truth), sparse files, compressed system files, and
  purgeable space make `du`-style totals diverge from `df` by design; carrying logical
  and allocated per entry (already the design) plus a one-line caveat beats silently
  confusing output.
- **Shard the cache by volume — correctness requires it anyway.** FSEvents journals,
  event IDs, and volume UUIDs are all per-volume, and one APFS container holds several
  volumes. So the whole-drive cache is naturally one snapshot shard per volume, each with
  its own resume token, validation ladder, and incremental journal+compaction lifecycle;
  a changed volume rewrites only its shard.
  “More than one file” falls out of the platform’s own structure.
- **The second-run path composes three things this document already sequences:**
  per-volume journal resume (H43) → re-bulk-scan of the named changed directories (H26)
  → render from persisted roll-ups without loading unchanged blocks (H33/H35/H16). Each
  is independently testable in the loop; the composition is the product.

This composition is backlog item H45: a whole-drive spike on a real Mac, measuring cold
first-scan, journal-resume second-scan at realistic churn (an hour and a day of normal
use), shard-update cost after a large change (an Xcode install), and the correctness
ladder under a purged journal — with dust and dumac as the calibration references on the
same drive.

### Watch-Path Hot Spots

The watch layer’s architecture is sound (bounded coalescer, one stat per coalesced path,
sticky overflow).
Three costs will dominate real usage and deserve attention before watch
hardening (`fdu-lka2`) is called done:

1. **Unpaired renames currently invalidate the root.** On FSEvents — macOS, the primary
   interactive platform — renames are *routinely* unpaired, so a single `mv` can cost a
   full-tree reconcile (currently seconds to minutes at scale).
   The known fix is already in the research: stitch by file identity (`(dev, inode)`
   cache, notify-debouncer-full’s design), and escalate to the *containing subtree*, not
   the root, when stitching fails.
2. **Directory-creation bursts.** `relist_new_dirs` emits an `InvalidateSubtree` per new
   directory; a `git clone` or `npm install` becomes a storm of subtree reconciles, with
   `take_invalidation_roots` collapsing them in O(k²) (`scan.rs:1050`). Batch-collapse
   before reconcile, and make the collapse O(k log k).
3. **Escalation cost is the product of watch quality and sweep speed.** Every overflow
   or contention fallback pays one reconcile — so rung 1’s parallel sweep is also the
   watch layer’s insurance premium.
   This coupling is worth stating in the plan: optimizing the sweep is not “the
   non-watch path”; it is what makes watch escalations affordable.

For incremental apply itself, the delta path inherits findings 4–5 (interned extensions,
allocation-free normalize): O(depth) apply with those fixed is sub-microsecond,
comfortably absorbing coalesced event batches.

### Windows: Deferred, Not Designed Out

Second priority by decision, but two facts should shape interfaces now.
First, Windows is the one target with a true persistent change journal (NTFS USN): rung
2 of the warm ladder generalizes there directly, so the snapshot’s resume-token fields
should be platform-tagged rather than FSEvents-shaped.
Second, the enumerate-with-attributes primitive exists there too
(`FindFirstFileEx`/`NtQueryDirectoryFile` return attributes during enumeration — with
the already-handled caveat that NTFS enumeration attributes can be stale, per the
portable-direntry research).
The walker-backend trait should assume “enumeration may yield attributes of varying
trust” as the general shape; macOS bulk, Windows enumeration, and Linux getdents+statx
all fit it.

### Sequencing: The Optimization Ladder

Implementation should climb four rungs in order, because each rung makes the next one
measurable and each is justified by the residual gap the previous one leaves — never by
anticipation:

1. **Portable, benefit-everywhere fixes.** The `ReconcileTarget::Direct` dispatch fix,
   allocation-free apply and normalize, parent-id passing, snapshot bulk load, parallel
   traversal and sweep with plain `std` threads, and record packing.
   These are ordinary Rust with no platform surface, they carry no silly-error-on-one-OS
   risk, and the cost model says they recover most of the gap at mid-scale on every
   platform. They are also a *precondition* for honest platform measurement: while
   userland wastes 2–6 µs/entry, a platform backend’s benefit is confounded by noise it
   did not cause — dumac’s 91%-kernel flamegraph is what a *clean* baseline looks like,
   and only against one does a syscall-layer change show its true size.
2. **Platform-conditional policy with fail-safe defaults.** Thread count, in-flight
   depth, inode-sort gating, `AT_STATX_DONT_SYNC`, QoS classes.
   A few lines each, all behind the policy layer, all degrading to portable behavior
   when detection is uncertain.
   These are “basic, obvious customizations” in the same sense multithreading is: low
   code, low risk, reversible.
3. **Platform mechanism backends.** getattrlistbulk on macOS; getdents64 + statx on
   Linux. Real code with real platform surface — built only where the measured gap
   between the rung-1 portable engine and the platform floor remains large.
   The cost model already predicts both will be justified (the portable walker cannot
   reach either platform’s floor: it makes one metadata syscall per entry on macOS where
   the platform offers one per ~300, and it cannot express narrow statx masks or dirfd
   discipline on Linux) — but the *decision* should cite the rung-1 baseline, not this
   prediction.
4. **Opportunistic accelerators.** io_uring, fanotify, FSEvents journal resume —
   probe-gated features that some environments will never grant.
   Last, because every one of them needs the lower rungs as its fallback path anyway.

The experiments below do not follow this order, and deliberately so: experiments are
cheap *measurements*, and the platform spikes (2, 4) exist precisely to price rung 3
before anyone commits to it.
Measure early, implement in ladder order.

## Key Insights

1. **The engine is a few× from its floor at mid-scale and ~100× at 1M entries, and the
   two gaps have different causes.** The mid-scale gap is userland constants — confirmed
   and now partly recovered by the live loop (exp-001/004/005), whose own data locates
   the remaining bound in the single index consumer on both paths.
   The 100× tail is almost certainly the OS metadata cache being exceeded with zero
   latency hiding; the knee experiment (H36) decides this for a day’s work and must run
   before the tail is optimized on a guess.
2. **Each platform has one dominant lever, and they differ.** macOS: syscall count
   (getattrlistbulk, ~hundreds of entries per syscall; parallelism plateaus on the
   volume lock). Linux warm: parallelism (near-linear to ≥8 threads on XFS). Cloud cold:
   latency hiding and access ordering under an IOPS budget.
   Userland: allocation discipline.
   A single tuned-for-one-host walker cannot win all three; a policy layer over three
   mechanism backends can.
3. **fdu can be first to two proven-but-unclaimed techniques.** No surveyed tool uses
   getattrlistbulk (6.4× measured by the one experiment that tried, dumac) or
   inode-ordered statting (4–6× cold on ext4 in prior art).
   Both are privilege-free and low-risk.
4. **Warm runs have a platform-split destiny.** On Linux the parallel sweep *is* the
   product (no journal exists; cloud RAM cannot hold the metadata cache, so the snapshot
   is the only warm state).
   On macOS the FSEvents journal can eliminate the sweep for quiet trees entirely —
   O(changes) opens, Watchman-proven, expressible in the existing delta/escalation
   contract. Build both; they back each other up.
5. **io_uring is correctly a footnote.** No getdents opcode, and seccomp-blocked in most
   of fdu’s own cloud targets.
   Probe-and-fallback later; never load-bearing.
6. **Memory is the quiet fifth workload.** ~490 B/entry versus a realistic ~50: boxed
   entries, duplicated names, inline roll-ups on files, and niche-less IDs.
   Packing is not (only) about fitting big trees — it is cache locality for every
   descent the index performs, and it compounds with every other fix.
7. **The snapshot loader is cache infrastructure, not a delta consumer.** Replaying the
   mutation contract per record spends ~28% of warm-start wall re-deriving structure the
   format already encodes.
   Trusted-bulk-load with a checksum is the design; the delta contract governs
   *changes*, not *deserialization*.
8. **Escalation rarity × sweep speed is the watch layer’s real performance metric.**
   Rename stitching and subtree-scoped escalation bound the numerator; the parallel
   sweep bounds the denominator.

## The Highest-Order Leverage Points

Before the itemized backlog, the five moves that change the *shape* of the curves rather
than shaving constants.
The loop’s discipline (profile → hypothesis → smallest change → paired measurement) is
right; the risk of any such loop is climbing a local hill.
These are the hills worth being on:

1. **Make warm cost O(changes), not O(entries): producer-side no-op elision.** This is
   the architectural answer to the loop’s H9 (warm 2.6× cold) and the correct reading of
   exp-002’s rejection.
   Today every entry — changed or not — flows through the single-threaded consumer’s
   expectation/arbitration machinery; exp-002 parallelized the walk but still funneled
   60k no-ops through one thread.
   Restructure: the parallel producers hold a clock-stable read-only baseline (the
   loaded index or the snapshot itself), compare fingerprints *in the workers*, and
   forward only mismatches plus per-directory verified-unchanged summaries.
   Consumer work becomes proportional to churn — typically zero.
   Predicted end state: warm-revalidate wall converges on parallel producer time (~190
   ms at 60k today, less once bulk stat lands) and finally drops *below*
   cold-scan-index; on an unchanged tree the consumer applies nothing.
   The delta contract is untouched — producers emit fewer, richer observations; no new
   mutation path.
2. **Serve queries from persisted roll-ups; stop rebuilding what the snapshot knew.**
   The snapshot stores raw entries and recomputes every reducer through apply on each
   load. Persist per-directory reducer state (duc’s lesson, ncdu2’s format shape) and two
   things follow: load skips all merge work, and a one-shot CLI query against an
   unchanged tree can be answered from the snapshot’s directory records in O(depth +
   output) *without materializing the index at all*. End state worth naming: warm `fdu`
   CLI = open, validate freshness, print — milliseconds at any tree size, which no tool
   in the survey achieves.
   Sequence: persisted roll-ups → single-pass load → lazy block format (`fdu-xihx`),
   each independently measurable in the loop’s `warm-snapshot-load` and a new
   `warm-query` job.
3. **Continue through the now-unblocked syscall rung.** exp-022 completed the dependency
   review and put direct `libc` plus the sole unsafe call behind a macOS-only module.
   `getattrlistbulk` removed per-entry `fstatat` from the cold profile and improved 720k
   producer wall 41.60%; directory `open` is now the largest residue at 33.86% of cold
   samples. Test H24 next inside the same audited boundary, then carry the bulk reader
   into reconciliation.
   Linux `statx`/`getdents64` still needs its own binding and host evidence rather than
   inheriting the macOS verdict.
4. **Parallelize index construction by subtree merge, not a faster funnel.** exp-001/002
   establish the single consumer as both paths’ ceiling (cold component 197 ms vs
   producer 192 ms). Cheaper apply (H6/H7, backlog below) raises the ceiling; the
   structural move removes it: workers build disjoint subtree indexes in their own
   arenas, and completion merges child into parent structurally — an arena splice plus
   one roll-up merge, refcount-triggered, dut’s design generalized — so there is no
   streaming consumer at all and cold-scan-index converges on producer time.
   This is what `fdu-gdrv`/`fdu-aky1` are really for; packing (H19–H22) is what makes
   the splice cheap.
5. **Keep moving the loop beyond one warm 60k APFS tree before trusting global
   verdicts.** The adaptive-depth and bulk-metadata work now includes 120k boundary and
   720k cache-pressure subjects, but controlled-cold caches, churned warm runs, network
   storage, and Linux are still invisible.
   exp-002 is predicted to flip in several of those states.
   The extensions are cheap because the generated-corpus harness already exists (recipes
   for 100k–1M, churn transitions, `--purge`); they are backlog items H36–H39, and they
   should interleave with code experiments rather than wait.

## Candidate Experiment Backlog (Loop-Ready)

Numbered to continue the loop’s registry (H12+); each is falsifiable in the loop’s
format: hypothesis, predicted signal (job, metric, direction), and prerequisites.
Experimentation is cheap — the intent is to walk through essentially all of these, in
roughly this order within each group, letting the profile re-rank between rounds.
Verdicts at 60k-warm generalize only to 60k-warm; anything marked *scale/state* needs
the loop extensions in H36–H39 to be trusted globally.

**Warm path (the H9 family):**

| # | Hypothesis | Predicted signal | Prereq |
| --- | --- | --- | --- |
| H12 | Producer-side fingerprint comparison against a clock-stable baseline makes consumer work O(changes); the expectation machinery per unchanged entry is the warm bound (exp-002’s residue) | `warm-revalidate` wall −40% or more at 60k; falls below `cold-scan-index`; `user_cpu_ns` collapses | — |
| H13 | Applying per *directory* (accumulate children locally, one ancestor merge per directory) cuts upward merges ~7× (7.3k dirs vs 52k files on the reference tree) | `user_cpu_ns` down on `cold-scan-index` and `warm-revalidate` | — |
| H14 | Routing `ReconcileTarget::Direct` through `collect_child_expectations` (deleting `collect_child_states`) removes ~13 allocations and ~10 descents per unchanged entry; equivalence already test-locked | `warm-revalidate` `user_cpu_ns`, `minor_faults` down | — |
| H15 | A directory whose stat fingerprint matches the snapshot can skip `read_dir` *membership discovery* (git untracked-cache trick; plocate’s updatedb ships exactly this contract — “it won’t readdir() it. It will stat() it, though”); child stats still run | `system_cpu_ns` down modestly on unchanged trees, most on wide dirs | guardrail G1 |
| H16 | On an unchanged tree, a one-shot CLI query can be served from snapshot roll-ups without building the index | new `warm-query` job wall → tens of ms | leverage 2, H33 |
| H17 | Replacing the transient per-directory `BTreeMap<OsString, PathExpectation>` (~152 B/child) with a sorted merge-join against the parent’s existing children removes a build-and-tear-down map per directory (extends the loop’s H11) | `warm-revalidate` `user_cpu_ns`, `minor_faults` down | — |
| H44 | A *labeled* structure-only revalidation (stat directories only; sizes of edited-in-place files may be stale, and the output says so) serves shape-tolerant queries at ~1/8 the stats | new labeled job ~8× fewer stats; never the default | change-propagation findings; G1 |

**Index and allocation (extends the loop’s H6/H7):**

| # | Hypothesis | Predicted signal | Prereq |
| --- | --- | --- | --- |
| H18 | Interning extensions to `u32` ids (side table; `by_ext` as id-keyed small vec) removes ~523k `String` clones + B-tree descents per 60k scan — the largest single apply cost | `cold-scan-index` `user_cpu_ns` down; `peak_rss` down | — |
| H19 | Storing `Entry` inline in the arena slot (un-boxing) converts a pointer chase per touch into sequential access | `user_cpu_ns`, `minor_faults`, `peak_rss` down ~10–15% | — |
| H20 | Storing each name once (parent-map key only, or a shared name arena) removes a duplicate heap string per entry | `peak_rss` −15–20%; locality gains | H19 helps |
| H21 | Moving `RollUp` (64 B) out of file entries into a directory-only side vector removes dead weight from ~88% of entries | `peak_rss` down ~55 B × file share | — |
| H22 | `EntryId` with a niche (`u32::MAX` sentinel) and `u32` revisions shrinks parent links from 24 B to 8 B and halves ABA overhead | `peak_rss` down ~24–32 B/entry | — |
| H23 | Carrying the parent `EntryId` in the op (walker and loader both know it) makes `ensure_dir_chain` O(1) instead of a root descent per entry | `cold-scan-index` `user_cpu_ns` down | — |

**Syscall and in-flight rung (macOS binding established by exp-022 — leverage 3):**

| # | Hypothesis | Predicted signal | Prereq |
| --- | --- | --- | --- |
| H24 | `openat` relative to a retained dirfd removes repeated path-prefix resolution (`open` = 33.86% of post-H26 cold self-time) | `system_cpu_ns` down, most on deep trees | **Ready on macOS**; exp-022 boundary |
| H25 | Linux `statx` with `STATX_BASIC_STATS` only, `AT_STATX_DONT_SYNC` on network mounts | `system_cpu_ns` down modestly; NFS dramatically | rustix |
| H26 | macOS `getattrlistbulk` (64 KiB buffers, drain-then-descend) replaces one `fstatat` per entry with one syscall per many entries | **Confirmed for cold scans (exp-022):** 720k producer wall −41.60%, system CPU −61.40%; 60k producer wall −9.25%. Reconciliation integration remains open. | landed cold backend |
| H27 | Raw `getdents64` with a 256 KB–1 MB per-thread buffer beats libc’s 32 KB `readdir` batching on wide directories | `system_cpu_ns` down on Linux; neutral macOS | rustix |
| H28 | Statting in `d_ino` order on ext4 turns random inode-table reads ~N/16 sequential | drop_caches-cold wall 2–6× down on ext4; neutral warm; neutral XFS | rustix; Linux host |
| H29 | An LRU of ancestor dirfds sized from `RLIMIT_NOFILE` keeps H24 effective at depth | `system_cpu_ns` flat vs depth | H24 |
| H30 | Worker QoS `USER_INITIATED` on macOS protects throughput under background load | wall variance down under load; neutral idle | — |
| H31 | In-flight depth from measured first-K operation latency (Little’s law) beats any fixed thread count across storage classes | **Confirmed by exp-015–021 on portable chunk timing:** 720k cold-index wall −5.31% and producer wall −10.09%; 120k wall and resources neutral | — |

**Snapshot (sequenced; leverage 2):**

| # | Hypothesis | Predicted signal | Prereq |
| --- | --- | --- | --- |
| H32 | Folding the CRC pass into parsing (one pass over the image, not two) | `warm-snapshot-load` `component_ns` −15–25% | — |
| H33 | Persisting per-directory reducer state eliminates all merge recomputation on load and enables H16 | `warm-snapshot-load` `user_cpu_ns` down; format v3 | — |
| H34 | A bulk arena fill (no per-record `Observation`/apply at all; reverse-pass roll-up only if H33 absent) reaches ~0.3–0.5 µs/record | `warm-snapshot-load` component → ~20–30 ms at 60k | — |
| H35 | Block format with tail index and lazy decompression makes open O(1) at any scale | open-time flat vs tree size | `fdu-1vd0`, H33/H34 |

**Scale and state coverage (loop extensions, mostly no code):**

| # | Hypothesis | Predicted signal | Prereq |
| --- | --- | --- | --- |
| H36 | The 500k→1M knee is metadata-cache capacity, not algorithm: iteration 2 of `--repeat 2` is near-linear while iteration 1 keels over | per-iteration `component_ns`, `minor_faults`, `blocked_ns`, `kern.maxvnodes` | corpus harness |
| H37 | exp-002 (parallel sweep) flips to accept at 500k+ and/or purge-cold, where misses exist to hide | `warm-revalidate` wall down ×threads in those states | H36 states |
| H38 | With H12 landed, warm-revalidate on a 1%-churn tree scales with churn, not size | churn-state `warm-revalidate` wall ∝ changes | H12 |
| H39 | exp-001..005 verdicts replicate on Linux/ext4 within 2×; divergences (B-tree vs hash, thread scaling) identify platform-conditional policy | full ledger re-run on one Linux host | Linux runner |

**Watch path and calibration:**

| # | Hypothesis | Predicted signal | Prereq |
| --- | --- | --- | --- |
| H40 | Stitching unpaired renames by `(dev, inode)` turns a `mv` from a full-root reconcile into an O(1) move | resident-mode `mv` cost at 60k/1M | — |
| H41 | Collapsing invalidation roots in O(k log k) and batching subtree reconciles bounds `git clone`-burst cost | burst reconcile count and wall down | — |
| H42 | A first-K-operations calibration probe classifies cache state and storage latency well enough to set order/depth at runtime | misclassification rate; cold wall on gp3 vs static defaults | — |
| H43 | FSEvents `sinceWhen` journal resume revalidates a quiet tree in tens of ms at any scale, with the sweep as backstop (validation ladder: UUID, ID regression, drop flags); cross-restart replay is Apple-documented but unproven in production tools, so the spike must prove it | macOS warm open O(changes); correctness identical to sweep every trial | snapshot fields; first-party objc2-core-services module (notify cannot express resume) |
| H45 | Whole-drive macOS spike: per-volume shards + journal resume + persisted roll-ups turn a 30–60 min dust-class drive scan into a seconds-scale recheck at realistic churn | cold first-scan minutes; hour/day-churn recheck seconds; shard rewrite bounded by changed volume | H26, H33, H43 |
| H46 | A fingerprint-keyed derived-data cache (line counts over a real repo) turns minutes-cold content summarization into seconds-warm: N stats + re-derive changed files only | cached rerun ~10–100× faster than cold; 1%-churn rerun ∝ churn; scc/tokei as cold-every-time references | tier findings; G5 |
| H47 | On btrfs/ZFS, a CoW snapshot held as the cache cursor and diffed at open (`btrfs send --no-data -p` / `zfs diff`) yields a complete change set — deletes and renames included — making Linux warm runs O(changes) with the same gate-and-fallback shape as FSEvents resume | quiet-tree warm revalidate in seconds→ms on btrfs/ZFS roots; sweep-identical digests; privilege and subvolume-scope caveats recorded | niche/privileged; fsevents-plan gate pattern |

**Guardrails, so a fast-looking result cannot be a wrong one:**

- **G1:** a matching directory fingerprint may skip only *membership discovery*, never
  child stats — in-place edits do not change the parent directory’s mtime.
  Any experiment that gets fast by trusting parent mtimes for child state is broken by
  construction, whatever its numbers say.
- **G2:** producer-side elision (H12) reads a clock-stable baseline and emits fewer
  observations; it must not write the index from workers.
  The delta contract stays the only mutation path.
- **G3:** the oracle digest must hold at every thread count and in every state — already
  the loop’s rule; it matters most for H12/H26, which change *what* is compared, not
  just how fast.
- **G4:** prefer deleting work over adding machinery when both test the same hypothesis
  — exp-003 vs exp-004 is the loop’s own demonstration.
- **G5:** fingerprints equal to the snapshot’s capture time at the filesystem’s
  timestamp granularity are *racily clean* — treat them as changed and re-verify (git’s
  rule). Any revalidation speedup must preserve this, and the snapshot must record the
  granularity it was captured under.
  Two production mechanisms to copy: git *smudges* racily-clean entries by truncating
  the cached size so a future mismatch is guaranteed (racy-git.txt measures the cost of
  getting this wrong: 2.22 s vs 0.14 s on 20k files), and borg deliberately excludes
  from its files cache any entry whose timestamp equals the newest in the archive — same
  race, closed from the other side.
  Fingerprint components also need per-filesystem trust: restic and borg both document
  inode-unstable filesystems (FUSE, pCloud, mergerfs) where inode-bearing fingerprints
  cause 100% false rescans and ctime churns on hardlink farms — the policy layer’s
  conservative-per-filesystem degradation applies to the fingerprint itself, not only to
  ordering.

Explicit non-experiments, to bound scope: FIEMAP/HDD ordering (off-target hardware),
io_uring before the H24–H27 walker exists (and expected-unavailable in containers), NUMA
pinning (no evidence at this scale), any SQL-backed persistence (measured 10–17× against
in the original research), and micro-tuning `readdir` batch sizes on the portable walker
(measured no stable effect in the portable-direntry research).

## Recommendations

1. **Implement in ladder order: portable fixes and plain multithreading first, fail-safe
   policy second, platform backends third, probe-gated accelerators last** — which the
   loop is already doing (exp-001/004/005 are rung 1). Next in that order: H14 (the
   dispatch fix — proven equivalent by an existing test), then the H9-family
   architecture (H12/H13) before any further parallelism, since exp-001/002 show the
   consumer is the ceiling on both paths.
2. **Adopt the three-backend walker shape** (bulk / getdents+statx / portable) behind
   one trait whose contract is “enumeration may yield attributes of varying trust,” with
   the policy layer (ordering, depth, threads, QoS) selected by the three-tier detection
   design (probe behavior, identify the residue, measure the hardware) and degrading to
   portable defaults whenever detection is uncertain.
   This also keeps Windows adoptable later without reshaping anything.
3. **Treat the parallel sweep as the product’s spine on Linux and the backstop
   everywhere**; treat FSEvents journal resume as the macOS warm-open accelerator and
   reserve its snapshot fields (event ID, volume UUID, platform tag) in the block format
   now so the format survives its arrival.
4. **Finish the snapshot ladder in sequence** — single-pass parse (H32), persisted
   roll-ups (H33), bulk arena fill (H34) — ahead of the block format, which then
   inherits a loader worth having; exp-005 was the first step of this ladder.
   Persisted roll-ups also unlock the index-free warm CLI query (H16), the strongest
   product-latency result available on any platform.
5. **Reuse the established platform boundary deliberately.** exp-022 chose an exact,
   macOS-only `libc` dependency already present in the lockfile, passed the supply-chain
   review, and confines unsafe code to one bounds-audited module.
   H24 and the reconciliation half of H26 should extend that boundary rather than adding
   a second binding abstraction without measured need.
   Linux remains a separate review.
6. **Extend the loop’s states and scales (H36–H39) in parallel with code experiments:**
   generated-corpus scale points, `--purge` runs, a churn job, a `warm-query` job, and
   one Linux host — so rejections and acceptances generalize beyond one warm 60k APFS
   tree.
7. **Fold the platform findings into the benchmark protocol:** label purge-cold vs
   remount-cold on macOS; record volume type and burst state on cloud runners; run ext4
   and XFS; record `kern.maxvnodes` and fault counters with every revalidation curve;
   and add dumac alongside dut and gdu as the macOS comparator, since it is the only
   tool exercising the same platform lever fdu will use.
8. **Keep the composability rule enforced:** scope, view, depth, and format never select
   engine variants; they filter and render one index.
   That is what keeps this document’s benchmark matrix — and the cache — stable while
   the CLI surface grows.
9. **Cache-write policy: default-on everywhere, gated on warm beating cold — not
   OS-gated.** The platform changes the cache’s benefit *magnitude*, never its sign,
   while writing costs almost nothing (36.9 ms at 60k, the ledger’s cheapest job;
   skip-rewrite-if-unchanged makes quiet reruns a resume-token update).
   Even without a change journal, the cache buys enumeration skip (H15), bulk load over
   rebuild (H34), inode-sorted cold sweeps *derived from cached inodes* before any
   readdir (H28’s ordering without its enumeration cost), and labeled
   stale-while-revalidate answers — the last mattering most on cold cloud runners where
   caches are otherwise weakest.
   The honest gate: with today’s engine, warm loses to cold (H9), so until the loop
   demonstrates `warm-revalidate ≤ cold-scan` at stat tier, default-on applies only
   where the cache already pays — macOS once journal resume lands, and any run with
   content-tier reducers.
   Hygiene either way: skip rewrite when nothing changed; `--no-cache` opt-out; degrade
   silently on read-only cache dirs; a purge command; user-cache location with
   owner-only permissions, since a snapshot is a complete file listing.

## Open Questions

1. Can cache state be detected reliably enough to drive policy (H42), or does
   misclassification cost force a conservative static default?
2. Where exactly does APFS bulk-enumeration concurrency plateau across M-series
   generations, and is the plateau lock-bound (fixed) or bandwidth-bound (scales with
   hardware)?
3. Does the FSEvents journal’s directory-level granularity hold under
   `kFSEventStreamCreateFlagFileEvents`-free operation for all the change classes the
   fingerprint needs (in-place edits change directory events how?), or does rung 2 need
   per-directory re-stat of children rather than fingerprint-only checks?
   (The sweep-as-backstop makes this a performance question, not a correctness one.)
4. How should the single-writer index absorb an 8-way producer once apply drops below 1
   µs/entry — batch application per directory, or sharded staging with ordered commit?
   (`fdu-r27g` owns the measurement.)
5. What is the right stat-batch size for the pipelined enumerate→sort→stat design on
   each storage class? (dua-core uses 4; dut streams; nobody has measured the sweep spot
   for network storage.)

## Next Steps

- Hand the H12+ backlog to the performance loop; merge it into the guide’s hypothesis
  registry so ledger verdicts accumulate against one numbering.
  Suggested first round: H14, H12, H18, H13 (warm architecture and apply cost), with H36
  run alongside to establish the scale states.
- Test H24 inside exp-022’s macOS boundary, then reuse the bulk reader for full and
  journal-scoped reconciliation; evaluate Linux bindings separately for H25/H27–H29.
- Extend the loop with the H36–H39 states/scales and a `warm-query` job.
- Correct the applies-to-which-path wording in the reconciliation fast-path research
  note.
- Reserve the journal-resume fields (event ID, volume UUID, platform tag) in the
  snapshot format design (`fdu-xihx`).
- Add dumac to the comparator list and the macOS purge-vs-remount cold protocol to the
  benchmark plan.
- Track the strategic items as beads where no bead exists: producer-side elision (H12),
  persisted roll-ups + `warm-query` (H16/H33), the calibration probe (H42), and the
  FSEvents journal resume spike (H43).

## References

Checked out under `attic/` for this research: bfs `4af45dc`; dut `68d4ba2` (GPL — ideas
only); parallel-disk-usage (pdu) `c30e46f`; diskus `90196e9`; jwalk v0.9.0.

macOS:

- [getattrlistbulk(2) man page](https://www.manpagez.com/man/2/getattrlistbulk/osx-10.12.6.php)
  ·
  [Apple DTS on bulk enumeration buffers and SMB looping](https://developer.apple.com/forums/thread/766035)
  · [ERANGE quirk](https://developer.apple.com/forums/thread/98262)
- [Tempelmann, directory-read performance on macOS](http://blog.tempel.org/2019/04/dir-read-performance.html)
  ·
  [dumac — getattrlistbulk du, with measurements](https://healeycodes.com/maybe-the-fastest-disk-usage-program-on-macos)
- [Szorc, global kernel locks in APFS](https://gregoryszorc.com/blog/2018/10/29/global-kernel-locks-in-apfs/)
  ·
  [Apple forums: APFS lock contention, 2025](https://origin-devforums.apple.com/forums/thread/800906)
- [FSEvents Programming Guide (persistent event IDs)](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html)
  ·
  [Watchman fsevents resync](https://facebook.github.io/watchman/docs/troubleshooting.html)
  · [git fsmonitor daemon](https://git-scm.com/docs/git-fsmonitor--daemon)
- [What happened to APFS fast directory sizing](https://mjtsai.com/blog/2025/01/13/what-happened-to-apfs-fast-directory-sizing/)
  ·
  [QoS and core types (eclecticlight)](https://eclecticlight.co/2024/12/17/tune-for-performance-core-types/)
  ·
  [setiopolicy_np(3) — dataless files](https://keith.github.io/xcode-man-pages/setiopolicy_np.3.html)
- FSEvents from Rust:
  [notify fsevent backend](https://github.com/notify-rs/notify/blob/main/notify/src/fsevent.rs)
  (read at tags `notify-8.2.0` and `notify-9.0.0-rc.4` under `attic/notify`) ·
  [notify PR #726 — objc2 migration](https://github.com/notify-rs/notify/pull/726) ·
  [notify #942 — runloop init deadlock](https://github.com/notify-rs/notify/issues/942)
  ·
  [objc2-core-services](https://docs.rs/objc2-core-services/latest/objc2_core_services/)
  ·
  [Watchman fsevents watcher](https://github.com/facebook/watchman/blob/main/watchman/watcher/fsevents.cpp)
  ·
  [git fsm-listen-darwin.c](https://github.com/git/git/blob/master/compat/fsmonitor/fsm-listen-darwin.c)
  and
  [git b0226007 — dispatch-queue migration](https://github.com/git/git/commit/b0226007f0aa)

Linux and cloud:

- [bfs 3.0: io_uring and parallelism](https://tavianator.com/2023/bfs_3.0.html) ·
  [io_uring getdents series (unmerged)](https://lwn.net/Articles/937900/)
- [moby seccomp: block io_uring](https://github.com/moby/moby/pull/46762) ·
  [containerd equivalent](https://github.com/containerd/containerd/pull/9320) ·
  [io_uring_disabled sysctl](https://lwn.net/Articles/937013/)
- [Ts’o on inode-ordered stats (LKML)](https://lkml.iu.edu/hypermail/linux/kernel/0804.3/1616.html)
  · [borg: sort by inode](https://github.com/borgbackup/borg/issues/905) ·
  [Kołaczkowski, disk access ordering](https://pkolaczk.github.io/disk-access-ordering/)
- [ext4 directory hashing](https://docs.kernel.org/filesystems/ext4/directory.html) ·
  [LWN: XFS metadata scaling](https://lwn.net/Articles/476263/)
- [fanotify(7)](https://man7.org/linux/man-pages/man7/fanotify.7.html) ·
  [FAN_RENAME](https://lwn.net/Articles/874378/) ·
  [inotify limits](https://watchexec.github.io/docs/inotify-limits.html)
- [overlayfs kernel doc](https://docs.kernel.org/filesystems/overlayfs.html) ·
  [cgroup throttling (Luu)](https://danluu.com/cgroup-throttling/) ·
  [EBS gp2/gp3 characteristics](https://docs.aws.amazon.com/ebs/latest/userguide/general-purpose.html)
- [zfs diff](https://openzfs.github.io/openzfs-docs/man/master/8/zfs-diff.8.html) ·
  [btrfs send](https://btrfs.readthedocs.io/en/latest/btrfs-send.html) ·
  [Robinhood Policy Engine (Lustre changelog consumer)](https://github.com/cea-hpc/robinhood)
  · [statx STATX_CHANGE_COOKIE](https://man7.org/linux/man-pages/man2/statx.2.html)
- [btrfs find-new](https://btrfs.readthedocs.io/en/latest/btrfs-subvolume.html) ·
  [dentry cache sizing incident](https://access.redhat.com/solutions/55818)

### Primary Sources: Maintainer Deep Dives (Annotated)

In-depth performance writing by the maintainers and kernel developers in this space,
collected 2026-08-10; each annotation carries the load-bearing insight.

**Kernel and VFS:**

- [Chinner, “XFS: Adventures in Metadata Scalability” (LCA 2012 slides)](https://xfs.org/images/d/d1/Xfs-scalability-lca2012.pdf)
  — the raw data behind the LWN piece: 8-thread create/traverse of 25M files/thread
  stays near-flat on XFS while ext4 and btrfs collapse ~4–7×; parallel stat scaling is
  filesystem-dependent, not just VFS-dependent.
- [Corbet, “Introducing lockrefs” (LWN 2013)](https://lwn.net/Articles/565734/) —
  packing spinlock+refcount into one cmpxchg word gave 6× on path-heavy workloads; why
  hot parent dentries stopped serializing walkers.
- [Brown, “Pathname lookup in Linux”](https://lwn.net/Articles/649115/) and
  [“RCU-walk”](https://lwn.net/Articles/649729/) — the per-component cost inventory of
  REF-walk and the write-nothing RCU-walk that makes warm lookups nearly free for
  concurrent readers.
- [Howells, statx RFC (2016)](https://lwn.net/Articles/685519/) — the mask and DONT_SYNC
  design rationale from the syscall’s author.

**Meta (Watchman / EdenFS / Sapling):**

- [“Scaling Mercurial at Facebook” (2014)](https://engineering.fb.com/2014/01/07/core-infra/scaling-mercurial-at-facebook/)
  — the canonical stop-walking-subscribe result (status >5× via Watchman), plus the
  rollout discipline: months of shadow-comparing watch answers against real rescans.
- [“Sapling” (2022)](https://engineering.fb.com/2022/11/15/open-source/sapling-source-control-scalable/)
  and
  [EdenFS Inodes.md](https://github.com/facebook/sapling/blob/main/eden/fs/docs/Inodes.md)
  — scale with the working set, not the repo; a non-materialized directory still
  carrying its source-control object ID is skipped wholesale during status — a persisted
  unchanged-subtree shortcut, the inverse of fdu’s invalidation model.

**Git developers:**

- [racy-git.txt](https://git-scm.com/docs/racy-git) — the canonical racily-clean
  analysis; smudging (truncate cached size) guarantees a future mismatch; 2.22 s vs 0.14
  s on 20k files when handled wrong.
- [Hostetler, builtin-fsmonitor RFC cover letter](https://lore.kernel.org/git/pull.923.git.1617291666.gitgitgadget@gmail.com/)
  and
  [GitHub blog write-up](https://github.blog/engineering/infrastructure/improve-git-monorepo-performance-with-a-file-system-monitor/)
  — daemon design rationale and the shipped numbers (Chromium status 17.6 s → 0.827 s;
  lstat sweep ~500×; dropped events → fresh token → one slow rescan).
- [Nguyen, untracked-cache cover letter (2014)](https://lore.kernel.org/git/1399474320-6840-1-git-send-email-pclouds@gmail.com/)
  — dir-mtime-keyed directory-listing cache (~80% off read_directory warm) with the
  honest caveats that led to `--test-untracked-cache`: dir-mtime semantics vary by
  filesystem, and racy timestamps disable the cache.
- [Peart, fsmonitor-hook v1 series (2017)](https://public-inbox.org/git/20170515191347.1892-1-benpeart@microsoft.com/)
  — 3M-file status 421 s → 18.6 s cold, and the crossover: at 3k files warm the hook is
  a regression.
- [index-format: UNTR/FSMN extensions](https://git-scm.com/docs/index-format) — how
  validity bitmaps + tokens are serialized rather than re-derived on load.

**Tavian Barnes (bfs):**

- [“bfs from the ground up, part 1”](https://tavianator.com/2016/bfs_1.html) — d_type
  stat-skipping (34k stats vs find’s 101k), BFS 2× slower cold / 15–25% faster warm, and
  the refcount-priority dircache.
- [“Parallelizing graph search with Rayon”](https://tavianator.com/2022/parallel_graph_search.html)
  — sequential 34 s, naive `par_bridge` **16 m 44 s**, thief-splitting 1.36 s.
- [“You could have invented futexes”](https://tavianator.com/2023/futex.html) — the
  blocking/wakeup primitives under any custom work queue.
- [“Bug hunting in Btrfs”](https://tavianator.com/2024/btrfs_bug.html) — parallel stat
  as a filesystem race detector; a kernel UPTODATE race surfaced only under bfs’s thread
  pool.
- [tailfin](https://github.com/tavianator/tailfin) — his benchmark stabilizer:
  turbo/SMT/ASLR off, frequency pinning, CPU/NUMA pinning; the checklist for claim-grade
  runs.
- [ripgrep PR #2591](https://github.com/BurntSushi/ripgrep/pull/2591) (+
  [#2642](https://github.com/BurntSushi/ripgrep/pull/2642)) and
  [fd PR #1422](https://github.com/sharkdp/fd/pull/1422) — the fd v9 6–13× mechanism:
  mutex queue → crossbeam deque (flat → 5.5× at j24) plus batched result channels (2523
  ms → 246 ms on 2.1M files).

**Andrew Gallant (ripgrep / ignore / walkdir):**

- [the ripgrep post](https://burntsushi.net/ripgrep/) — gitignore cost is matching, not
  walking; compile all globs into one batch matcher.
- [PR #223 (WalkParallel)](https://github.com/BurntSushi/ripgrep/pull/223) —
  closure-per-worker so per-thread ignore state needs no locking.
- [issue #1550](https://github.com/BurntSushi/ripgrep/issues/1550) — breadth-first order
  itself caused a ~1 GB peak: per-directory state × frontier width.
- [discussion #2472](https://github.com/BurntSushi/ripgrep/discussions/2472) — warm
  traversal fastest at 4 threads, degrading at 8; optimum rises only with CPU-bound work
  alongside.
- [walkdir README](https://github.com/BurntSushi/walkdir/blob/master/README.md) — the
  serial baseline’s claims and the `max_open` fd-budget tradeoff.

**macOS and du-family maintainers:**

- [Oakley, APFS fast directory sizing](https://eclecticlight.co/2019/02/06/how-big-is-that-folder-what-happened-to-apfs-fast-directory-sizing/)
  — the on-disk format carries the recursive `total_size` fdu computes; no API reads it.
  [His Spotlight-indexing instrumentation](https://eclecticlight.co/2025/08/04/a-deeper-dive-into-spotlight-indexing-and-local-search/)
  bounds macOS’s own index freshness at ~7–8 s after a write.
- [Tempelmann, directory-read benchmarks](http://blog.tempel.org/2019/04/dir-read-performance.html)
  and
  [his filesystem-dev post](https://www.mail-archive.com/filesystem-dev@lists.apple.com/msg00263.html)
  — the per-filesystem API verdicts, including the APFS-bulk-slower-than-HFS+ dissent
  the H26 spike must test.
- [dust #375](https://github.com/bootandy/dust/issues/375) — rayon locks in a traversal
  order the maintainer can’t change; 1 thread nearly matches `du` on HDD.
- [Byron, dua #92](https://github.com/Byron/dua-cli/issues/92) — “anything dua does
  pales in comparison to the fs syscalls”; M1 E-cores skew work-stealing pools.
- [Byron, gitoxide discussion #1326](https://github.com/GitoxideLabs/gitoxide/discussions/1326)
  — dirwalk concurrent with index check beats `git status` 1.44× by *re-paying* lstats:
  redundant syscalls beat serialization on modern SSDs.
- [pdu announcement](https://gist.github.com/KSXGitHub/b8dacc7753ae56ae51cd599e779014c1)
  — author’s admission that his parallel walker beats `du` in CI but loses locally: wins
  are environment-dependent.
- [gdu discussion #114](https://github.com/dundee/gdu/discussions/114) — adaptive GC
  against free-memory pressure; the in-memory-index vs persistent-store tension.

**plocate and backup tools (fingerprints and index I/O):**

- Gunderson’s plocate posts (originals offline; full text preserved via Planet Debian
  snapshots:
  [2020-10-12 capture](https://web.archive.org/web/20201012165143/https://planet.debian.org/),
  [2020-12-06 capture](https://web.archive.org/web/20201206182416/https://planet.debian.org/))
  — trigram index: 26M-file query 20.9 s → 0.008 s; io_uring gather-reads for posting
  lists (cold query 200–400 ms → 40–60 ms), with the honesty note that drop_caches
  “doesn’t actually always drop all the caches”; async `statx` to pre-warm the dentry
  cache before `access()`; and updatedb’s merge contract — stat every directory, readdir
  only the changed ones (H15’s production precedent).
  [plocate NEWS](https://sources.debian.org/src/plocate/latest/NEWS/): a shared zstd
  dictionary made the database 7% smaller *and* linear scans 20% faster.
- [restic file-change detection](https://restic.readthedocs.io/en/stable/040_backup.html#file-change-detection),
  [issue #2179](https://github.com/restic/restic/issues/2179) and
  [PR #2212](https://github.com/restic/restic/pull/2212) — why ctime joined the
  fingerprint (Debian tools restore mtime after content changes), why it is coupled to
  inode identity, and the `--ignore-inode`/`--ignore-ctime` escape hatches for
  inode-unstable filesystems and hardlink farms.
- [borg FAQ on the files cache](https://borgbackup.readthedocs.io/en/stable/faq.html#it-always-chunks-all-my-files-even-unchanged-ones)
  and
  [internals](https://borgbackup.readthedocs.io/en/stable/internals/data-structures.html)
  — every way (ctime, size, inode) fingerprints produce spurious rescans in practice;
  the newest-timestamp exclusion that closes the racily-clean window; and the budget
  anchor: ~240 B per file of cache state.

fdu internals referenced: `crates/fdu/src/scan.rs`, `index.rs`, `snapshot.rs`,
`watch.rs`, `types.rs`, `examples/perf_probe.rs`; the
[experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md) and its
records under `docs/project/experiments/`; and the prior research and plan documents
linked above.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
