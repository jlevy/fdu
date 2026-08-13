# fdu Performance Architecture: Evidence From Forty-One Experiments

**Date:** 2026-08-12

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

Forty-one measured experiments produced a 54.5% improvement in snapshot-absent indexed
wall time and a 52.0% improvement in verified warm-open wall time against the original
implementation on the primary 60,067-entry subject.
On a heterogeneous 1,007,659-entry workspace, the integrated branch was 31.3% faster
than merged `origin/main` with exact oracle parity.
A separate 976,295-entry product comparison measured a 4.237-second FDU median, versus
7.546 seconds for dust, 6.684 for pdu, and 8.315 for gdu on an M1 Pro MacBook with a
local APFS SSD.

The largest gains came from architectural changes, not constant tuning.
Bounded parallel traversal, integer-keyed roll-ups, service-time-adaptive concurrency,
macOS `getattrlistbulk`, bulk reconciliation, and producer-side no-op elimination all
paid.
Larger buffers, allocation reuse, extra workers, path clone removal, and descriptor
frontiers did not. The main remaining cost is the size and construction of the reusable
index: it used about 585 MiB at million-entry scale, and the only faster comparator was
dumac, which retained only a scalar total and an inode set.
The first requirement-derived plan now closes most of that retained-state cost for the
existing rich summary composition: exp-040 avoids index retention when no later consumer
can use it, improving wall 14.56% and cutting RSS 95.28% on the heterogeneous acceptance
tree without changing report bytes.
Exact-final-binary runs on two more uniform 720,805- and 901,963-entry trees reproduced
the roughly 3× user-CPU and 23–30× memory advantage, while wall improved only 1.8–2.8%.
That contrast localizes the remaining floor to filesystem work and topology rather than
index construction.

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
six.
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

The final fresh-process comparison used a later 976,295-entry fingerprint of that
workspace. It ran twelve adjacent interleaved pairs per competitor after three warmups,
with FDU’s persisted cache disabled:

| Tool | Product | Median wall |
| --- | --- | ---: |
| **fdu** | complete reusable index and 10-row tree | **4.237 s** |
| pdu | depth-one rendered tree | 6.684 s |
| dust | depth-one 10-row rendered tree | 7.546 s |
| gdu | depth-one 10-row rendered tree | 8.315 s |
| ncdu | retained index, UI disabled | 28.576 s |
| dumac | scalar allocated-byte total | **3.566 s** |
| diskus | scalar total | 7.064 s |
| dua | scalar total | 8.352 s |

FDU beat every tree or index product and every scalar tool except dumac.
Dumac’s 17.6% paired advantage is an upper-bound signal: it requests fewer attributes
and retains only a total and hard-link inode set.
FDU’s current index used about 585 MiB at this scale, versus dumac’s 45 MiB. The next
material opportunity is therefore compact index construction, not more APFS worker
threads.

Full method, intervals, semantic caveats, and source analysis are in the
[live tool comparison](report-2026-08-12-fdu-live-tool-comparison.md), with exact facts
in its [reproduction manifest](fdu-live-tool-comparison-manifest-v1.json).

## The Most Useful Negative Results

Forty-one experiments are valuable partly because they close plausible but unproductive
paths:

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

## Next Performance Frontier

The experiment queue is ordered by potential impact and by design risk:

1. **Worker-local derived summaries (H62, `fdu-hly4`).** Exp-040 proved the execution
   plan; now aggregate inside existing workers so files need no relative path, `Op`,
   observation batch, or single summary consumer.
2. **Report-derived macOS metadata (H63, `fdu-vpow`).** Use a separate strict
   `getattrlistbulk` record for the derived summary, omitting index-only fields and
   allocating names only for directories.
   Re-screen 64 versus 128 KiB for that narrower record; do not resurrect the rejected
   256 KiB full-record change.
3. **Selected-total matched challenge (H64, `fdu-8nfx`).** Derive only the requested
   apparent or allocated total for a fair dumac-class workload.
   Keep rich `summary` unchanged and publish hard-link/symlink accounting differences
   beside any claim.
4. **Reduction-only worker depth (H65, `fdu-i076`).** Re-run the 6/8/10/12/16 curve
   after H62 removes the consumer; the indexed path retains its accepted policy.
5. **Compact reusable entries (H19–H22, `fdu-prph`).** Measure the entry layout, remove
   duplicate name storage, move directory-only state out of files, and compact IDs and
   revisions one arm at a time.
   Million-entry RSS is the clearest current defect.
6. **Worker-local subtree construction (H60, `fdu-weey`).** Build disjoint local arenas
   and splice them with one roll-up at region completion, reducing path and channel work
   without bypassing the index contract.
7. **Dense immutable base plus sparse overlay (H61, `fdu-f67r`).** After the layout
   floor is known, test whether bootstrap state can remain dense while later mutations
   use a bounded overlay and compaction cycle.
8. **Portable wide-directory stat chunks (H58, `fdu-r9he`).** On Linux or the portable
   backend, test dua-style small stealable metadata jobs.
   This is not expected to help the macOS bulk path.
9. **Journal scoping.** FSEvents can turn a quiet warm run from O(entries) into
   O(changed regions), but the same fast full scan remains the fallback and the basis
   for first use, invalid journals, and explicit cache-off runs.

## Linux Evidence Still Needed

The current headline is deliberately limited to one M1 Pro and local APFS. Linux will
exercise the portable per-entry metadata path and may produce a different ranking.
A future report should repeat the exact comparator matrix on a controlled local-SSD
Linux host with:

- the same immutable tree or a generated and verified million-entry equivalent
- verified-warm and per-sample controlled-cold regimes reported separately
- `sync` and `/proc/sys/vm/drop_caches` preparation for the cold regime, following the
  useful part of the diskus method
- exact binary, compiler, kernel, filesystem, mount, CPU, memory, and storage provenance
- the FDU oracle, pre/post fingerprint, work classes, paired schedule, raw resource
  metrics, and confidence intervals used on macOS
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
The remaining scalar-only gap and memory footprint point to one frontier: construct and
store the same complete index more densely, rather than doing less work and calling it
the same result.

The complete numerical record, including every rejection, remains in the
[experiment ledger](report-2026-08-10-fdu-performance-experiments.md).
The measurement and acceptance protocol is the
[performance loop](../guides/performance-loop.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
