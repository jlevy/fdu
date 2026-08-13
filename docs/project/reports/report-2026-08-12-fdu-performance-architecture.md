# fdu Performance Architecture: Evidence From Forty-Seven Experiments

**Date:** 2026-08-12 (updated 2026-08-13)

**Status:** Technical white paper

## Abstract

FDU builds a reusable index of an entire file tree while computing size, count, recency,
and file-type roll-ups for every directory.
That product is more demanding than a scalar `du -sh` total, so speed cannot come from
discarding the index or weakening its correctness contract.
It has instead come from changing where work happens: parallelize the latency-bound
producer, reduce kernel transitions with platform bulk metadata, remove repeated tree
work from the single-writer consumer, and compare immutable state in parallel before
sending only real changes through the mutation contract.

Forty-seven measured experiments produced a 54.5% improvement in snapshot-absent indexed
wall time and a 52.0% improvement in verified warm-open wall time against the original
implementation on the primary 60,067-entry subject.
On a heterogeneous 1,007,659-entry workspace, the integrated branch was 31.3% faster
than merged `origin/main` with exact oracle parity.
A separate 901,963-entry product comparison measured a 3.324-second FDU median, versus
6.016 seconds for dust, 5.657 for pdu, and 6.782 for Go gdu on an M1 Pro MacBook with a
local APFS SSD.

The largest gains came from architectural changes, not constant tuning.
Bounded parallel traversal, integer-keyed roll-ups, service-time-adaptive concurrency,
macOS `getattrlistbulk`, bulk reconciliation, and producer-side no-op elimination all
paid.
Larger buffers, allocation reuse, extra workers, path clone removal, and descriptor
frontiers did not. The main remaining cost is the size and construction of the reusable
index: it used about 398 MiB at near-million-entry scale.
The only comparator with a lower scalar-arm median was dumac, which retained only a
selected total and inode set; its paired interval crossed zero, so neither tool had a
statistically established wall-time lead.
The first requirement-derived plan now closes most of that retained-state cost for the
existing rich summary composition: exp-040 avoids index retention when no later consumer
can use it, improving wall 14.56% and cutting RSS 95.28% on the heterogeneous acceptance
tree without changing report bytes.
Exact-final-binary runs on two more uniform 720,805- and 901,963-entry trees reproduced
the roughly 3× user-CPU and 23–30× memory advantage, while wall improved only 1.8–2.8%.
That contrast localizes the remaining floor to filesystem work and topology rather than
index construction. Exp-041 through exp-044 then removed progressively more summary
representation work, including a selected-total scanner matched to dumac.
They cut user CPU and memory but failed the wall-time gate, confirming that directory
opens and bulk system calls now own the warm-APFS floor.
Exp-045 and exp-046 are the open-ahead follow-ups: exact-binary profiles localize both
FDU and dumac to `open` plus `getattrlistbulk`; exp-045’s pairwise helper was superseded
by exp-046’s shared pool, which remains unretained pending quiet-host confirmation.

All M1/APFS numbers in this paper use a repeated-workload warm-steady operating-system
filesystem cache, established by a complete independent fingerprint and explicit
warmups. “Cold” job names refer to an absent FDU snapshot, not an evicted OS cache.
Warm-steady is not a full-residency claim: the near-million-entry subject exceeds the
host’s vnode target.
Controlled-cold Linux and dedicated-volume macOS results remain separate future evidence
because cache state can change both absolute latency and the size of a relative
advantage.

## The Product Being Optimized

FDU’s cold path produces one complete inventory from which a caller can ask multiple
questions without another filesystem walk.
Every entry has exact stat-tier metadata; every directory has precomputed roll-ups;
stable identities support snapshots, queries, changes, and progressive results.
The index is not temporary CLI formatting state.

That defines the optimization boundary:

- enumerate every in-scope entry and preserve partial/error boundaries
- retain exact metadata needed by cache validation and every supported view
- keep one mutation authority so snapshots, queries, and change feeds cannot diverge
- make a fresh process with `--cache off` fast without relying on persisted state
- treat FSEvents or another journal as a future scope reducer, not as a substitute for a
  fast non-cached scan

Comparisons therefore use three work classes.
An **indexed tree** retains browseable state, a **rendered tree** produces roll-ups and
bounded human output, and a **scalar total** reduces the scan to one number.
The classes share traversal work but are not semantically interchangeable.

## Integration Boundary for Performance PR #8

The PR is the performance layer on top of the merged composable CLI in #5. Current
`main` already contains region-scheduled breadth-first traversal, the explicit path
requirement, the safe bare-help behavior, the compact default tree, and the public
composition rules. The performance branch retains those semantics and adds five
production mechanisms:

1. service-time-adaptive reserve workers for a slow fresh scan;
2. macOS bulk metadata for fresh scans;
3. reuse of that bulk reader during full reconciliation;
4. bounded parallel immutable-baseline reconciliation with producer-side no-op
   elimination; and
5. an internal exact transient-summary plan for an unfiltered cache-off summary-only
   request.

It also fixes two reconciliation correctness defects found during integration review: an
enumerated entry whose metadata lookup fails is no longer mistaken for a deletion, and a
late deferred-change overflow resumes at the first unapplied wave instead of retrying
completed work and double-counting its statistics.
The evidence review fixed two harness defects as well: the component probe now emits
newest regular-file mtime as required by the real-tree oracle, and the real-tree
allocated-byte aggregate uses apparent size on non-POSIX hosts, matching both FDU and
the oracle’s own engine digest when native block counts are unavailable.

Against freshly rebuilt merged `origin/main`, the integrated branch measured:

| Immutable APFS subject | Job | Wall-time change |
| --- | --- | ---: |
| 60,067 entries | snapshot-absent indexed scan | -5.1% |
| 60,067 entries | verified warm open | **-42.3%** |
| 720,805 entries | snapshot-absent indexed scan | **-30.5%** |
| 720,805 entries | verified warm open | **-70.9%** |
| 1,007,659 entries | cache-off indexed scan | **-31.3%** |
| 1,007,659 entries | scan producer | **-36.6%** |

These are the PR merge deltas.
The 54.5% snapshot-absent index and 52.0% verified-warm results elsewhere in this paper
are cumulative campaign comparisons with the older pre-campaign `b565882` binary, not
the incremental difference from current `main`.

### Four Different Batching Boundaries

The implementation has four batching layers with different effects and failure modes:

1. **Kernel record batches:** the new macOS reader fills one reusable 64 KiB buffer with
   the children and stat-tier attributes of one open directory.
   A wide directory takes several calls; the API does not batch across directories.
2. **Observation batches:** workers publish up to 1,024 ordinary operations to the
   consumer. This pre-existing mutation boundary is unchanged; an earlier sweep found no
   stable benefit from increasing it.
3. **Directory claims:** a worker takes four directories from the region scheduler at a
   time. This pre-existing queue-lock amortization is small enough not to hoard a whole
   region.
4. **Reconciliation waves:** up to 1,024 directories share one immutable index baseline
   and four comparison workers.
   Effective changes are applied only after the wave joins, through the ordinary
   mutation authority and observation batches.

The fresh-scan 30.1–41.6% gain in exp-022 comes primarily from kernel record batching.
Reusing it during warm reconciliation produced the 34.4% exp-026 gain.
Immutable comparison waves then removed the unchanged-entry mutation funnel and produced
the additional 59.5% large-tree warm gain in exp-030. Those experiments use successive
controls, so their percentages compose in the code but must not be added arithmetically.

## Cost Model

A complete scan has four dominant costs:

1. **Filesystem metadata work.** Directory enumeration and stat-tier attributes are the
   irreducible kernel work.
   On a portable path this is approximately one metadata call per entry; macOS bulk
   attributes amortize many entries per call.
2. **Latency overlap.** A serial producer idles during cache misses.
   Too little concurrency exposes latency; too much creates kernel contention, scheduler
   work, and memory pressure.
3. **Index construction.** Paths, names, identities, extension tallies, and ancestor
   roll-ups consume user CPU and allocations after metadata arrives.
4. **Output.** A bounded tree report should be a small query over retained roll-ups, not
   a second scan.

Warm revalidation adds snapshot loading and an equality problem.
An unchanged entry should be proved unchanged in a worker, then discarded.
Sending every no-op through the single-writer mutation path repeats index lookup,
arbitration, and batching work without changing the answer.

This model predicts the campaign’s main result: optimizations that remove a whole class
of kernel or index work compose; optimizations that merely rearrange allocations or
increase queue depth after the bottleneck has moved tend not to matter.

## Architectural Changes That Paid

### Bounded Parallel Production

The original walker was latency-bound and largely serial.
A bounded directory producer feeding one index consumer cut cold indexed wall time 50.0%
in exp-001. It raised total CPU, as expected when waiting is replaced by useful parallel
work, but retained a clear ownership boundary: producers observe the filesystem and only
the index consumer commits state.

This was the foundational change.
The original gap to dust was not that FDU performed intrinsically more expensive
userland work; it was leaving cores idle while metadata requests waited.

### Cheaper Index and Snapshot Constants

Several measured changes removed repeated work inside the retained-state product:

- borrowed path components improved warm revalidation 9.4% (exp-004)
- resolving snapshot records through their known parent improved snapshot load 18.6%
  (exp-005)
- reading direct reconcile expectations from entry IDs improved warm revalidation 7.1%
  (exp-007)
- interning extension keys as integers improved cold indexed wall 15.7% (exp-008)
- folding checksum calculation into snapshot parsing improved the snapshot-load
  component 12.4% (exp-009)

These are related wins: preserve identity already known at a layer boundary instead of
reconstructing it from a root-relative path, and use compact stable keys for roll-ups
that are merged at every ancestor.

### Region-Scheduled Breadth-First Traversal

A single global FIFO made breadth-first progress visible but widened the live frontier.
The shipped scheduler groups work by top-level region, rotates ready regions, and keeps
local work LIFO within each region.
That retained shallow, balanced progressive coverage while recovering the memory
locality of depth-first traversal (exp-013).

This decision was retested after the tree and CLI changed.
On the million-entry subject, switching back to depth-first made indexed wall time 3.57%
worse (exp-037). Breadth-first is therefore both a progressive-result semantic and the
faster measured operating point for the current heterogeneous tree.

### Service-Time-Adaptive Concurrency

One fixed worker count cannot represent both a small warm tree and a large tree under
metadata-cache pressure.
Six workers were sufficient at the ordinary operating point; sixteen helped the earlier
high-latency portable path.
The accepted policy measures initial filesystem service time and activates reserve
workers only when observed latency justifies them.
It improved 720,805-entry indexed wall time 5.31% and producer wall time 10.09% without
changing the smaller-tree resource profile (exp-021).

The important design is feedback, not the number sixteen.
After bulk metadata removed per-entry waits, sixteen workers regressed indexed wall time
19.2% and roughly doubled CPU (exp-025). The service-time trigger correctly remained at
six. The calibration intentionally decides once from the initial sample.
It does not re-evaluate if a later subtree or mount becomes slower; repeated calibration
would add coordination to every directory release for a case not established in the
measured local-tree workload.
A future mixed-latency or network-filesystem experiment must measure that trade rather
than assume the opening sample represents every subtree.
On the later million-entry tree, eight workers bought only 1.30% for 33.5% more CPU,
while twelve and sixteen were slower (exp-036).

### macOS Bulk Metadata

The largest platform-specific gain came from replacing separate directory enumeration
and per-entry stat calls with `getattrlistbulk`. The reader requests all attributes FDU
needs for its stat-tier contract, validates record bounds and returned masks, and falls
back for the complete directory on malformed data, unsupported filesystems, mount
points, or firmlinks.

At 720,805 entries this improved cold indexed wall time 30.1%, producer wall time 41.6%,
and system CPU 46.6% (exp-022). Reusing the same audited reader for full reconciliation
improved warm-open wall time 34.4% and cut system CPU 54.0% at that scale (exp-026).

The source reviews explain the cross-tool result.
Dust, gdu, and pdu use portable recursive parallelism and consumed 35–43 aggregate
CPU-seconds on the final workspace.
FDU’s bulk path built its full index and report in 19.7 CPU-seconds.
Fewer kernel transitions, rather than skipped entries, account for the difference; exact
FDU oracle checks and immutable tree fingerprints guard completeness.

### Parallel Immutable Comparison and No-Op Elimination

The first attempt to parallelize reconciliation sent every observed entry through one
consumer and improved wall time only 2.6%, below the acceptance threshold (exp-002). The
successful design moved equality work to bounded worker waves over an immutable
baseline.
Workers compare complete directory observations, discard exact no-ops, and send
only effective changes through the existing delta authority.

That distinction was decisive.
The final design improved warm-open wall time 30.3% at 60,067 entries and 59.5% at
720,805 entries; reconciliation component time fell 50.3% and 72.6%, with exact parity
and bounded memory (exp-030). Parallelism helped only after the serialized boundary
stopped receiving work that could be proved irrelevant before it.

## Cumulative and Product-Level Evidence

Against the pre-campaign binary, the final integrated stack produced these paired
results on the primary 60,067-entry APFS subject (exp-032):

| Job | Wall-time change | 95% interval |
| --- | ---: | ---: |
| Snapshot-absent indexed scan | **-54.5%** | -55.3% to -53.7% |
| Snapshot-absent producer | **-60.0%** | -62.2% to -58.7% |
| Scan and snapshot save | **-52.4%** | -56.6% to -51.0% |
| Verified compatible-snapshot open | **-52.0%** | -54.1% to -50.1% |
| Snapshot load only | **-35.7%** | -36.2% to -35.3% |

The composable CLI merge was not assumed performance-neutral.
Rebuilt current binaries were compared directly.
The performance branch retained exact output semantics and was 5.1% faster for cold
indexing and 42.3% faster for warm revalidation at 60,067 entries (exp-033), then 30.5%
faster for cold indexing and 70.9% faster for warm revalidation on the 720,805-entry
pressure tree (exp-034). On the then-1,007,659-entry workspace it was 31.3% faster for
cold indexing and 36.6% faster for the producer (exp-035).

The final fresh-process comparison used the self-contained 901,963-entry benchmark tree.
It ran twelve adjacent interleaved pairs per competitor after three warmups, with FDU’s
persisted cache disabled.
Tree/index and scalar work classes ran as separate matrices:

| Tool | Product | Median wall |
| --- | --- | ---: |
| **fdu** | complete reusable index and 10-row tree | **3.324 s** |
| **fdu** | exact five-tally summary | **3.125 s** |
| dumac | scalar allocated-byte total | 2.980 s (statistical tie) |
| pdu | depth-one rendered tree | 5.657 s |
| dust | depth-one 10-row rendered tree | 6.016 s |
| gdu | depth-one 10-row rendered tree | 6.782 s |
| ncdu | retained index, UI disabled | 20.550 s |
| diskus | scalar total | 5.708 s |
| dua | scalar total | 5.459 s |

FDU beat every tree or index product and every scalar tool other than the statistically
tied dumac comparison.
Dumac’s 2.2% lower paired point estimate is an upper-bound signal: it requests fewer
attributes and retains only a total and hard-link inode set.
The interval [−5.7%, +1.7%] crosses zero, and dumac nevertheless used 85.4% more
aggregate CPU and 224.5% more peak RSS than FDU’s richer summary.
H67 then replayed both the current and published exact binary pairs under a busier host
regime. Dumac led current FDU by 16.19% in twelve pairs and the published FDU binary by
11.1% in a five-pair diagnostic.
The intervening reconciliation-only change was therefore not a live-scan regression.
FDU sustained 3.46 aggregate core-equivalents and dumac 5.64, while exact process
samples put 96.10% and 94.21% of worker tops, respectively, in `open` or
`getattrlistbulk`. The relative wall result is sensitive to host pressure and
concurrency; the original quiet-host matrix remains the published typical comparison.

The next material tree-product opportunity is still compact index construction, not a
static increase in APFS worker threads.
H69 separately tests whether open-only helpers can overlap the two synchronous syscall
phases under one shared concurrency budget.

Full method, intervals, semantic caveats, and source analysis are in the
[live tool comparison](report-2026-08-13-fdu-live-tool-comparison.md), with exact facts
in its [reproduction manifest](fdu-live-tool-comparison-manifest-v2.json).

## The Most Useful Negative Results

Completed experiments are valuable partly because they close plausible but unproductive
paths; exp-045 records the superseded pairwise mechanism and exp-046 the unretained
shared-pool follow-up:

- **Parallelism without moving the serialization boundary:** the first parallel
  reconcile funnel gained only 2.6%; immutable producer-side comparison later gained
  30–60% (exp-002 versus exp-030)
- **More threads after syscall batching:** sixteen workers changed from helpful on the
  latency-bound portable path to a 19.2% regression after bulk metadata; queue depth
  must follow the current bottleneck (exp-015 versus exp-025)
- **Allocation folklore:** moving producer paths, reusing directory staging vectors, and
  reducing bootstrap journaling changed wall time by less than the acceptance bar or
  regressed resources (exp-003, exp-016, exp-028)
- **Bigger buffers:** increasing the macOS bulk buffer from 64 KiB to 256 KiB did not
  help at 60,067 entries and changed million-entry wall time by +2.22% with an interval
  crossing zero (exp-029 and exp-039)
- **Descriptor-relative opening:** one retained root descriptor and a bounded
  parent-relative frontier were both neutral; repeated prefix resolution was no longer a
  dominant cost on the bulk path (exp-024 and exp-038)
- **Locally sensible index batching:** accumulating same-parent roll-ups gained 2.5%
  after extension interning, below the gate.
  The two ideas competed for the same cost, and the simpler integer-key change captured
  the useful share (exp-011)
- **Changing traversal order on intuition:** depth-first was measurably slower on the
  final heterogeneous tree, despite its reputation for locality (exp-037)
- **Removing summary representation after the syscall floor:** worker-local reduction,
  narrower macOS records, and a selected-total in-buffer fold cut user CPU by 36–52% and
  RSS by 35–40% without clearing the 3% wall gate (exp-041, exp-042, exp-044)

These failures support a general rule: profile the current architecture, because a
successful change moves the bottleneck and invalidates earlier tuning.

## Benchmarking Lessons and the Diskus Protocol

The
[current diskus benchmark](https://github.com/sharkdp/diskus/blob/90196e950017d25b2940e8e0fda51a321ca66e1a/README.md#benchmark)
uses Hyperfine on a 15 GB Linux tree with roughly 100,000 directories and 400,000 files.
Its warm regime uses five warmups.
Its cold regime runs `sync` and writes `3` to `/proc/sys/vm/drop_caches` before every
timed invocation. It also used Hyperfine’s parameter scan to choose tin-summer’s worker
count. These are good practices: cold and warm cache states are separate products,
per-sample cache preparation is stronger than a one-time purge, and a concurrency claim
should include a measured sweep.

FDU’s protocol adds controls needed for an index whose correctness and output surface
are richer:

- adjacent paired scheduling with alternating order and bootstrap confidence intervals
- exact executable hashes, versions, commands, host, filesystem, and tree fingerprints
- pre-run and post-run mutation detection
- an independent exact oracle for FDU experiment output
- retained invalid samples, stable-output digests, hard-link prevalence, and process
  resource metrics
- explicit indexed-tree, rendered-tree, and scalar-total work classes

The protocols are complementary.
FDU already performs broader worker-depth experiments and stronger provenance and
semantic validation.
A future Linux run should add diskus’s per-sample `sync` plus `drop_caches` preparation
as a **controlled-cold** regime while retaining a separate warm-steady regime and all
FDU validity checks.
A cache-preparation command succeeding is not enough by itself; the result should record
the command, privilege boundary, kernel state or observable proof, and failures per
sample.

Diskus reports arithmetic means, standard deviations, minima, maxima, and relative
ratios. FDU should retain its paired median and bootstrap interval as the decision
statistic because filesystem timings are skewed and drift over long runs, while also
keeping raw samples so means or Hyperfine-compatible summaries can be reconstructed.

### What dut changes in the Linux plan

A source refresh at current upstream commit
[`68d4ba2`](https://codeberg.org/201984/dut/commit/68d4ba2d66211e7ca93a2312bb12f5879d0179e1)
confirms that dut is the most interesting Linux rendered-tree comparator, but not a
like-for-like full-index oracle.
It retains directories plus bounded top-N files and runs allocated bytes, apparent
bytes, or file counts as separate modes.
Its transferable mechanisms are a reused 1 MiB `getdents64` buffer, dirfd-relative
`statx`, one-CAS sibling-batch publication, demand-sized worker wakeups, early top-N
rejection, and last-child bottom-up roll-up.
Because the source is GPL, FDU uses these descriptions and measurements for independent
experiments only; no implementation is copied or linked.

The refresh also makes the validity bar stricter.
Recent dut fixes addressed entries lost across a full directory buffer, unbounded growth
after `EINVAL`, and a hard-link-table resize error.
The adapter must therefore pass wide-directory, hard-link-growth,
sparse/preallocated-size-ordering, partial-error, symlink, mount-boundary, and non-UTF-8
fixtures before its timing counts.
That is especially important because dut emits human output and can warn about partial
traversal while returning success.

dut’s published warm comparison uses warmups, but its SSD/HDD preparation writes `1` to
`/proc/sys/vm/drop_caches` without a preceding `sync`. Under the
[Linux kernel contract](https://docs.kernel.org/admin-guide/sysctl/vm.html#drop-caches),
that targets page cache but does not request dentry/inode slab reclamation.
The future report will call this `pagecache-drop-only`, not cold, and publish it beside
verified warm and per-sample `sync` plus `echo 3` controlled cold.

## Next Performance Frontier

The experiment queue is ordered by potential impact and by design risk.
Three proposed representation layers have now been measured and removed: H62 cut user
CPU 36.23% but improved wall only 1.38% (exp-041), and the H62 plus H63 composition
changed wall by +1.86% [−1.96%, +4.56%] (exp-042). That evidence says another
rich-summary allocation optimization is unlikely to move the warm APFS syscall floor.
Exp-043 also retained six workers: eight looked promising on the 901k screen but changed
720k wall by +0.67% while raising CPU 40.66%; deeper pools were neutral or slower.
H64’s complete selected-total specialization then changed wall by only −1.15%
[−2.24%, +0.44%] and did not beat dumac despite halving user CPU (exp-044).

1. **Confirm a shared directory-opener pool (H70, `fdu-druf`).** H67 found six FDU
   workers and ten dumac workers at the same synchronous `open` and `getattrlistbulk`
   boundary. H69’s pairwise handoff was inconclusive.
   H70’s shared two-opener screen improved wall 3.98% [0.70%, 9.87%], but doubled
   involuntary context switches; its count sweep and direct dumac run suffered extreme
   host outliers. Run twelve quiet adjacent pairs, then replicate any qualifying gain on
   an independent large topology.
   The opener and adaptive-worker policies must share one budget: the first prototype
   accidentally activated both and created eighteen threads.
2. **Directory-only transient tree (H66, `fdu-sk7v`).** For an unfiltered cache-off
   tree-only request, test folding file observations into exact directory roll-ups and
   retaining only directory topology.
   Require byte-identical output at 60k and near-million scale; fall closed to the full
   index for cache, filters, composed views, watch, or reusable-index requests.
3. **Compact reusable entries (H19–H22, `fdu-prph`).** Measure the entry layout, remove
   duplicate name storage, move directory-only state out of files, and compact IDs and
   revisions one arm at a time.
   Million-entry RSS is the clearest current defect.
4. **Worker-local subtree construction (H60, `fdu-weey`).** Build disjoint local arenas
   and splice them with one roll-up at region completion, reducing path and channel work
   without bypassing the index contract.
5. **Dense immutable base plus sparse overlay (H61, `fdu-f67r`).** After the layout
   floor is known, test whether bootstrap state can remain dense while later mutations
   use a bounded overlay and compaction cycle.
6. **Portable wide-directory stat chunks (H58, `fdu-r9he`).** On Linux or the portable
   backend, test dua-style small stealable metadata jobs.
   This is not expected to help the macOS bulk path.
7. **Journal scoping.** FSEvents can turn a quiet warm run from O(entries) into
   O(changed regions), but the same fast full scan remains the fallback and the basis
   for first use, invalid journals, and explicit cache-off runs.

## Linux Evidence Still Needed

The current headline is deliberately limited to one M1 Pro and local APFS. Linux will
exercise the portable per-entry metadata path and may produce a different ranking.

A first paired scouting pass on a virtualized Linux host now exists in
[the Linux first measurements](../research/research-2026-08-13-linux-first-measurements.md).
It is not release evidence — the rig is a VM whose guest-cold reads can still be
host-cached, and its subject was generated for that session — but it ranks the work: the
enumeration layer is already at parity with dut and diskus in syscalls issued, the index
consumer accounts for the tree-class gap, and the verified warm open runs slower than a
cold scan. The claim-grade matrix below remains outstanding, and the scouting numbers do
not substitute for any part of it.

A future report should repeat the exact comparator matrix on a controlled local-SSD
Linux host with:

- the same immutable tree or a generated and verified million-entry equivalent
- verified-warm, dut-compatible pagecache-drop-only, and per-sample controlled-cold
  regimes reported separately
- `sync` plus `echo 3 > /proc/sys/vm/drop_caches` for controlled cold, following the
  useful part of the diskus method and recording failures per sample
- ext4 and XFS results or an explicit record that only one filesystem was available,
  with a worker-count sweep rather than an inherited thread constant
- exact binary, compiler, kernel, filesystem, mount, CPU, memory, and storage provenance
- the FDU oracle, pre/post fingerprint, work classes, paired schedule, raw resource
  metrics, and confidence intervals used on macOS
- dut-specific wide-directory, hard-link-growth, selected-size-ordering, and partial
  error postconditions before comparator timing is accepted
- profile evidence before considering `statx`, raw `getdents64`, io_uring, or a
  different queue design

Linux numbers should be added beside the macOS table, not averaged with it.
Platform backends change the syscall count and concurrency optimum; one combined number
would hide the mechanism the benchmark is meant to expose.

## Conclusion

FDU’s performance campaign has validated a coherent architecture.
A bounded parallel producer hides metadata latency.
Region scheduling makes breadth-first progress cheap.
Adaptive concurrency responds to observed service time.
Platform bulk attributes remove per-entry kernel transitions on macOS. Compact identity
and roll-up keys reduce retained state work.
Immutable parallel comparison keeps no-ops away from the single mutation authority.

Together these changes more than halved both cold indexed scans and verified warm opens
without changing query or cache semantics.
The million-scale product comparison now places FDU ahead of the established tree
renderers on the measured M1/APFS host.
The small scalar-only gap is now experimentally resistant to representation changes; the
reusable tree’s memory footprint remains the larger defect.
The next frontier is to construct and store the same complete index more densely, while
journal scoping changes the amount of filesystem work only when trusted OS history
proves that is safe.

The complete numerical record, including every rejection, remains in the
[experiment ledger](report-2026-08-10-fdu-performance-experiments.md).
The measurement and acceptance protocol is the
[performance loop](../guides/performance-loop.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
