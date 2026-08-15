# Deleting Work: Performance Research with Soft Schemas

**Date:** 2026-08-14

**Author:** fdu project

**Status:** Current

**Interactive companion:**
[Explore the experiment map and cross-platform graphics](performance-research/index.html).

## Abstract

Deleting unnecessary work produced most of fdu’s durable speedups.
The successful changes reused identity already present in snapshots and traversals,
collapsed metadata calls, rejected unchanged entries before serialized mutation, and
avoided retained state that a one-shot query could never consume.
Plausible alternatives such as more threads, larger buffers, relative directory opens,
inode ordering, inline content analysis, and `io_uring` batching were neutral or slower.

The current cumulative Linux comparison measures 9.1% lower latency for cold indexed
scanning, 25.3% for warm revalidation, and 31.4% for snapshot loading.
On macOS, the cumulative transfer test measures 15.7% lower warm-revalidation latency;
cold indexed scanning remains statistically neutral.
An equal-weight index over the comparable Linux and macOS cold and warm cells gives
12.7% lower latency, equivalent to 14.6% more throughput for perfectly serial work.
The index combines separate measurements for scenario analysis; it is not a benchmark
result.

The evidence comes from 56 experiment artifacts: 27 accepted, 24 rejected, two
baselines, two superseded, and one still in progress.
An accepted artifact is not necessarily an independent speedup: the set includes
behavior choices, instrumentation checks, cumulative controls, and cross-platform
validations. Each artifact pairs machine-readable measurements with rationale,
interpretation, and provenance.

Forty-four terminal index experiments preserve a decision-state checkpoint: 41 on macOS
and three on Linux. The history is useful but incomplete.
macOS has absolute cold wall, CPU, and RSS values for 36 checkpoints, warm wall for 27,
and snapshot-load wall for 12; Linux has three cold vectors, two warm values, and no
snapshot-load checkpoint.
Only 21 of the 44 checkpoints can be tied to an exact source revision from the surviving
records. The report shows those gaps instead of manufacturing a continuous trend.

Low-overhead counters, caller-tree profiles, falsifiable hypotheses, interleaved trial
pairs, exact correctness oracles, and a predeclared acceptance rule turned each attempt
into durable evidence.
Recording failed experiments with the same care as wins narrowed the next search and
kept attractive dead ends from becoming folklore.

The [interactive experiment map](performance-research/index.html) plots absolute
kept-state history separately from local paired effects, organized by platform and
mechanism. Hover and keyboard details expose each setup, and the aggregate controls
expose every weight.
Negative percentages mean lower latency.
A *clear* result has its entire 95% paired bootstrap interval on one side of zero; an
ordinary production speedup must also clear the 3% acceptance threshold, preserve
correctness, and justify its complexity.

## 1. One Scan, Three Different Workloads

fdu walks a directory tree, collects size and metadata, and renders a tree, file list,
extension tally, or summary.
The words *directory scan* hide three workloads with different costs and useful outputs.

### Retained State Determines the Job

| Tier | What it retains | Main costs | Typical question |
| --- | --- | --- | --- |
| Aggregate | running totals, not individual paths | directory enumeration and metadata | How large is this tree? |
| Index | one durable entry per filesystem object | scan plus identity, insertion, roll-up, and snapshot work | Which files and directories account for the space? |
| Content | the index plus selected file contents and derived analysis | classification, reads, decoding, analysis, and cache behavior | What is inside the files? |

A change that helps one tier may not reach another.
For example, the exact-summary path in exp-040 cuts peak memory 95.3% by avoiding an
index that a one-shot request cannot consume.
That is a decisive aggregate-tier win and says nothing about queries that need the
index.

### Cold and Warm Paths Pay for Different Work

A *cold* run enumerates the filesystem and constructs current state.
A *warm* run loads a previous snapshot and revalidates it against the filesystem.
The paths share types and correctness rules but not cost shape:

- cold scanning is dominated by directory and metadata system calls plus index
  insertion;
- snapshot load is dominated by parsing, allocation, path reconstruction, and tree
  insertion; and
- warm revalidation adds filesystem checks and reconciliation against retained state.

The cumulative results differ enough across these jobs that one “fdu got faster” number
would obscure both mechanism and product value.

### Parallel Observation Ends at One Mutation Authority

Workers enumerate directories and produce observations concurrently.
A single mutation authority applies those observations under the delta contract so
snapshots, queries, and change feeds cannot disagree.
The boundary explains several results:

- bounded scan concurrency pays when workers overlap metadata latency;
- adding workers stops paying once the bulk macOS path removes much of that latency;
- parallel reconciliation fails when it merely moves the same work across a channel; and
- parallel reconciliation succeeds when workers prove unchanged entries are no-ops and
  keep them out of the serial mutation boundary.

Correctness is part of performance, not a later gate.
Every candidate must preserve exact output and the stable engine digest.
A faster wrong answer is an invalid sample, not an optimization.

## 2. Measured Results by Platform

### Linux: All Three Measured Jobs Improved

The current cumulative Linux comparison uses a 450,463-entry tree and 18 interleaved
trial pairs per variant.
The control is the campaign branch point and the candidate is its tip.

| Job | Control | Candidate | Change | 95% interval | Interpretation |
| --- | ---: | ---: | ---: | ---: | --- |
| Warm snapshot load | 1897.2 ms | 1303.8 ms | **−31.4%** | [−32.0%, −30.8%] | Clear gain |
| Warm revalidation | 2317.6 ms | 1726.6 ms | **−25.3%** | [−26.4%, −23.8%] | Clear gain |
| Cold indexed scan | 2107.8 ms | 1909.2 ms | **−9.1%** | [−13.2%, −7.6%] | Clear gain |
| Cold producer probe | 2381.3 ms | 2200.2 ms | −7.3% | [−8.9%, −6.1%] | Wall moved, component did not |

The producer row does not establish a producer optimization.
Its isolated component measured 345.3 ms versus 346.8 ms, and no campaign change
targeted that component.
The wall movement is therefore most plausibly environmental or outside the measured
component.

Component measurements localize the real work:

- snapshot loading fell from 939.7 ms to 390.0 ms, a 58.5% component reduction; and
- index building fell from 979.3 ms to 797.3 ms, an 18.6% component reduction.

The cumulative changes reversed the startup tradeoff: warm opening is now 22.6% faster
than cold scanning on Linux; at the start, warm was 69% slower.

### macOS: Warm Gains Transferred

Exp-054 measured the Linux campaign cumulatively on macOS to test whether portable
source changes produced portable effects.

| Job | Control | Candidate | Change | 95% interval | Interpretation |
| --- | ---: | ---: | ---: | ---: | --- |
| Cold indexed scan | 298.127 ms | 306.243 ms | +1.393% | [−0.186%, +3.895%] | Neutral; interval crosses zero |
| Warm revalidation | 392.991 ms | 335.747 ms | **−15.682%** | [−16.286%, −13.993%] | Clear gain |

The warm result supports retaining the campaign, but Linux’s cold-scan gain remains
unproven on macOS.

### Aggregate Scenarios Without False Precision

A *derived decision index* combines four comparable cells: Linux cold, Linux warm, macOS
cold, and macOS warm.
It uses the weighted geometric mean of latency ratios:

\[ R = 1 - \exp\left(\sum_i w_i \log(1 + c_i / 100)\right) \]

where `c_i` is a measured percentage change and the non-negative weights sum to one.
With equal platform weight and equal cold/warm weight, `R` is **12.7% lower latency**.
The reciprocal latency ratio corresponds to **14.6% throughput-equivalent speedup**.

The index supports scenario comparison but not a benchmark claim:

- its cells were measured on different hosts and trees;
- it combines unlike work according to chosen weights;
- it is not the elapsed time of a real mixed workload; and
- a different product mix should use different weights.

The interactive controls expose those assumptions.
Raw milliseconds are never averaged across machines.

## 3. The Experimental Loop

Each candidate passes through the same loop, producing either a shipped change or a
recorded constraint on future work.

1. **Instrument the boundaries.** Count work by layer and sample process truth without
   leaving the instrument permanently active.
2. **Profile before proposing a change.** Use caller trees to distinguish an expensive
   function from the caller that made it expensive.
3. **Write a falsifiable hypothesis.** Name the platform, tier, start state, metric,
   predicted mechanism, and expected signal.
4. **Change one thing.** Keep causal attribution possible.
5. **Prove equivalence first.** Compare exact output, stable digests, and tree state.
6. **Measure paired and interleaved.** Alternate control and candidate so host drift
   reaches both arms.
7. **Apply the acceptance rule.** Require the magnitude, interval, valid oracle, and
   complexity trade to pass together.
8. **Settle the state.** Commit the accepted production change or restore the rejected
   control, then resolve the exact source revision that remains.
9. **Record the verdict and checkpoint.** Store the complete profile, kept arm, source
   revision, and rejection evidence.
10. **Re-screen the queue.** A landed change may remove the headroom behind the next
    hypothesis.

### Paired, Interleaved Trials Control Host Drift

Running all control samples and then all candidate samples confounds the candidate with
temperature, page cache, background work, and host load.
An alternating schedule pairs nearby samples so slow periods affect both variants.
The harness uses at least 12 pairs for claim-grade work and more when variance or the
decision boundary requires it.

Surprising results require both orderings.
If A:B and B:A disagree on the sign, the result is position bias or noise rather than a
product claim. Shared-runner CI omits timing because it would benchmark the runner.

### The Acceptance Rule Separates Evidence from Desire

For a normal speed change, all of the following must be true:

- paired median latency improves by at least 3%;
- the whole 95% paired bootstrap interval lies below zero;
- the independent oracle rejects no candidate sample;
- the subject tree did not mutate during measurement; and
- dependencies, unsafe code, failure modes, CPU, and memory remain worth the gain.

The rule is one-sided.
`passes_acceptance: false` does not mean “unchanged”; it can mean unclear, too small, or
a clear regression. The artifact records effect direction separately from the product
decision.

Not every accepted experiment is a speed change.
Exp-012 accepted breadth-first order for progressive shallow results despite neutral
wall time and a small memory cost.
Exp-052 and exp-053 accepted instrumentation because their purpose was to bound
measurement overhead.
Exp-055 accepted correctness and safety fixes because neither macOS interval detected a
wall regression.

### Local Effects and Kept-State History Answer Different Questions

The paired comparison decides one experiment.
The absolute checkpoint records what remained after that decision.
Accepted experiments keep the candidate; rejected experiments keep the control.
A rejected candidate therefore cannot make the cumulative line slower, and a favorable
percentage against an old control cannot masquerade as the next point in a time series.

The `index-core-v1` profile runs cold indexed scan, cold producer, snapshot save, warm
revalidation, and snapshot load, retaining every process metric for every job.
The interactive history selects five absolute dimensions from that matrix: cold wall,
warm wall, load wall, cold CPU, and cold peak RSS. Each dimension has its own scale and
unit.
Lines connect only adjacent checkpoints with the same tree, host class, filesystem,
virtualization, operating-system cache state, job, and metric.

Git revision replay audits that history independently.
It archives a declared sequence of commits, builds each with one locked current
toolchain, and measures every binary in one interleaved run against one frozen corpus.
Running the same resolved commit list on macOS and Linux produces two platform histories
without averaging raw milliseconds across machines.
The source revision is the experimental variable; the current toolchain, profile, and
local corpus are controlled conditions.

### Three Instrumentation Tiers Answer Different Questions

| Tier | Source | Approximate cost | What it answers |
| --- | --- | --- | --- |
| Application | counters at semantic call sites | about 1 ns per event | Which layer requested the work? |
| Process | kernel process counters sampled by phase | one small system read | What did the process do? |
| External | `strace -c`, `perf`, callgrind, or platform profiler | roughly 2×–50× | Which system calls and caller paths dominate? |

Application counters ship in the binary but stay off unless `FDU_COUNTERS=1` enables
them. Exp-053 measured the idle and recording paths separately: idle −1.26%
[−2.96%, +1.40%], recording +0.64% [−0.68%, +2.13%]. Both span zero, so the claim is
bounded overhead rather than zero overhead.

No tier is sufficient alone.
Application counters saw one directory-read operation per directory while `strace`
showed two `getdents64` calls: one carrying entries and one returning zero at EOF.
Conversely, the kernel can count calls but cannot say which engine layer requested them.

## 4. Soft Schemas Preserve the Research

An experiment artifact needs narrative context and stable machine-readable fields.
A soft schema supplies both without forcing rationale, anomalies, and judgment into
rigid database columns.

Each experiment is Markdown with a small authoritative YAML frontmatter envelope:

```yaml
---
title: Bounded parallel directory producer
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-001
  date: "2026-08-10"
  hypotheses: [H1]
  subject: { ... }
  method: { ... }
  results: [ ... ]
  checkpoint:
    profile: index-core-v1
    kept_variant: candidate
    source_revision: 0123456789abcdef0123456789abcdef01234567
  verdict: { ... }
  complexity: { ... }
---
```

The body explains rationale, observations, and interpretation.
Any value consumed by code lives in frontmatter and is validated against the compiled
schema and semantic model.
Tools never scrape a prose sentence or Markdown table for a number.

### What Belongs in the Contract

Promote a value into the envelope only when a later step consumes it.
The current contract captures:

- stable identity, date, and hypothesis references;
- subject tree identity, scale, platform, filesystem, cache state, and virtualization;
- control and candidate definitions, binaries, toolchain, warmups, trials, and
  interleaving;
- per-job metrics, paired changes, intervals, effect direction, and invalid samples;
- decision, primary metric, threshold, reason, and landing commit;
- the versioned checkpoint profile, surviving arm, and exact source revision; and
- complexity: production lines, dependencies, unsafe blocks, failure modes, and notes.

Exploration, judgment, profiler interpretation, and future ideas stay prose until a tool
needs them. This avoids a hard schema for thought while preserving a hard boundary for
evidence.

### Why One Artifact per Experiment Matters

One artifact per experiment makes the research durable:

- a failed idea is searchable by mechanism and hypothesis;
- each decision travels with its exact regime and correctness evidence;
- schema changes produce visible validation failures rather than silent table drift;
- generated ledgers and graphics can be rebuilt from source; and
- Git history shows when a conclusion changed.

The generated ledger and interactive companion are projections.
The experiment directory remains authoritative.

### Projecting Any Soft-Schema Directory into a Table

A schema-directed directory projector turns any one-contract artifact directory into
stable table rows:

1. discover Markdown artifacts matching a caller-supplied pattern;
2. require one shared soft-schema contract, envelope, and compiled schema;
3. read only the authoritative envelope;
4. resolve caller-selected dotted field paths through the compiled JSON Schema;
5. derive labels, types, descriptions, and requiredness from that schema;
6. emit stable rows containing typed cell values, source links, and the original
   structured payload; and
7. let a renderer handle search, filters, column visibility, and a detail disclosure.

Explicit column selection is intentional.
Automatically flattening every nested object and repeated result would create dozens of
unreadable columns and confuse absence with summary.
The schema describes what a field means; a view profile decides which fields answer the
current reader’s question.

The same projector could support a Metabrowser plugin that discovers any one-contract
artifact directory and mounts the generic table and record inspector through
Metabrowser’s public plugin SDK. Because the projector consumes only the contract, the
plugin would need neither experiment-specific parsing nor changes to the static report.

## 5. All 56 Experiments

The ledger divides 56 experiments into seven chronological phases.
The primary percentage is the artifact’s decision metric; it may measure a component or
resource rather than wall time, and it may not be the reason an experiment was accepted.
The [generated ledger](report-2026-08-10-fdu-performance-experiments.md) contains every
metric and verdict sentence, while the
[interactive explorer](performance-research/index.html#experiments) exposes each setup
and structured payload.

### Phase 0: Establish the Baseline and Oracle

| ID | Decision | Primary result | What it established |
| --- | --- | ---: | --- |
| exp-000 | Baseline | 0.0% | Real 60k-entry reference, paired schedule, stable digest, and output oracle |

### Phase 1: Parallel Scan and Index Constants

| ID | Decision | Primary result | Resolution |
| --- | --- | ---: | --- |
| exp-001 | Accepted | −50.0% wall | Bounded producer concurrency overlaps metadata latency; CPU rose 58% |
| exp-002 | Rejected | −2.6% wall | Parallel revalidation was real but below the 3% bar for about 180 lines |
| exp-003 | Rejected | +1.0% wall | Skipping bootstrap journaling removed clones but not elapsed work |
| exp-004 | Accepted | −9.4% wall | Borrowed path components deleted repeated allocation and parsing |
| exp-005 | Accepted | −18.6% wall | Snapshot load used the parent ID already present in the format |
| exp-006 | Accepted control | −48.9% wall | Cumulative anchor proved the early wins composed against baseline |
| exp-007 | Accepted | −7.1% wall | Reconciliation read expectations directly from entry IDs |
| exp-008 | Accepted | −15.7% wall | Integer extension IDs removed repeated strings and comparisons |
| exp-009 | Accepted | −12.4% component | Single-pass checksum and parse moved the preregistered loader signal |
| exp-010 | Rejected | −0.03% wall | Deferred joins had no remaining headroom after direct entry-ID reads |
| exp-011 | Rejected | −2.5% wall | Fewer ancestor merges amortized work already made cheap by interning |

### Phase 2: Traversal Order and Adaptive Concurrency

| ID | Decision | Primary result | Resolution |
| --- | --- | ---: | --- |
| exp-012 | Accepted behavior | −0.6% wall, neutral | Breadth-first order preserved progressive shallow results for a small cost |
| exp-013 | Accepted | −3.8% peak RSS | Region scheduling recovered BFS’s memory cost without changing wall time |
| exp-014 | Baseline | −3.0% producer wall | Same-binary control measured BFS’s shipped cost; warm serial path regressed 2.7% |
| exp-015 | Accepted | −11.7% wall | More workers paid only on a 720k tree under metadata-cache pressure |
| exp-016 | Rejected | −0.4% wall | Moving paths instead of cloning left time neutral and worsened RSS/faults |
| exp-017 | Rejected | +2.0% wall | Dormant workers added CPU, faults, and memory before activation |
| exp-018 | Superseded | −4.0% wall | Entry-count activation worked at 720k but fired too early near 120k |
| exp-019 | Rejected | +1.2% wall | The first scale crossing gained no time and added resource cost |
| exp-020 | Rejected | −1.7% wall | A safer capacity trigger was under the bar and unclear |
| exp-021 | Accepted | −5.3% wall | Initial filesystem service time selected concurrency without the boundary cost |

### Phase 3: macOS Bulk Metadata and Warm Reconciliation

| ID | Decision | Primary result | Resolution |
| --- | --- | ---: | --- |
| exp-022 | Accepted | −30.1% wall | `getattrlistbulk` changed the syscall shape; producer wall fell 41.6% |
| exp-023 | Accepted control | −53.5% wall | Cumulative anchor through adaptive scanning and bulk metadata |
| exp-024 | Rejected | −0.1% wall | One retained root descriptor did not move indexed wall or system CPU |
| exp-025 | Rejected | +19.2% wall | Sixteen workers after bulk metadata doubled CPU and added one-third RSS |
| exp-026 | Accepted | −34.4% wall | Warm reconciliation reused bulk records; system CPU fell 54.0% |
| exp-027 | Accepted control | −52.8% wall | Cumulative anchor through bulk reconciliation |
| exp-028 | Rejected | +0.2% wall | Reused staging allocations missed CPU/fault signals and hurt producer resources |
| exp-029 | Rejected | −1.8% wall, unclear | A 256 KiB bulk buffer did not corroborate in system CPU and raised cold resources |
| exp-030 | Accepted | −59.5% wall | Workers proved no-ops before bounded parallel reconciliation waves |
| exp-031 | Rejected | +1.6% wall | Larger waves did not amortize startup and worsened the component point estimate |
| exp-032 | Accepted control | −54.5% wall | Final early-campaign cumulative anchor with exact oracle parity |

### Phase 4: Integration and Million-Entry Validation

| ID | Decision | Primary result | Resolution |
| --- | --- | ---: | --- |
| exp-033 | Accepted control | −42.3% warm wall | Post-CLI integration retained the measured gains against current main |
| exp-034 | Accepted control | −30.5% cold wall | Pressure-tree validation reproduced cold and warm gains |
| exp-035 | Accepted control | −31.3% cold wall | One-million-entry validation retained speed but exposed +44.3% RSS |
| exp-036 | Rejected | −1.3% wall | Eight workers missed the bar for +33.5% CPU; 12 and 16 regressed |
| exp-037 | Rejected | +3.6% wall | Depth-first was clearly slower; region-scheduled BFS remained preferred |
| exp-038 | Rejected | −0.7% wall, unclear | Parent-relative `openat` did not justify descriptor-lifetime machinery |
| exp-039 | Rejected | +2.2% wall, unclear | The larger macOS buffer again failed on the live million-entry tree |

### Phase 5: Retained-State Tiers and Content Work

| ID | Decision | Primary result | Resolution |
| --- | --- | ---: | --- |
| exp-040 | Accepted | −14.6% wall | Exact rich summary avoided a reusable index and cut RSS 95.3% |
| exp-041 | Rejected | −1.4% wall | Worker-local reduction cut CPU 36% and RSS 35% but missed the wall bar |
| exp-042 | Rejected | +1.9% wall, unclear | macOS summary records reduced resources but not user-visible time |
| exp-043 | Rejected | +0.7% wall, unclear | Eight summary workers failed independent confirmation and added 40.7% CPU |
| exp-044 | Rejected | −1.1% wall, unclear | Selected-size specialization required a second unsafe parser and lost the trade |
| exp-045 | Superseded | −4.5% wall, very unclear | Per-worker open pipelining had a [−31%, +34%] interval; shared pool superseded it |
| exp-046 | In progress | −4.0% short screen | Shared macOS opener pool needs quiet claim-grade and topology confirmation |
| exp-047 | Rejected | +66.3% wall | Inline content analysis destroyed useful parallel I/O |
| exp-048 | Rejected | +1.5% wall, unclear | Prose-collector gating did not move SLOC, basic, or cache-hit jobs |
| exp-049 | Rejected | −3.5% wall, unclear | Markdown reserve had an interval from −14.5% to +7.4% and was reverted |
| exp-050 | Accepted | −12.0% wall | Complete UTF-8 chunks decoded in place; CPU and RSS also fell |

### Phase 6: Linux Structure, Instrumentation, and Platform Transfer

| ID | Decision | Primary result | Resolution |
| --- | --- | ---: | --- |
| exp-051 | Accepted | −7.35% cold wall | Previous-parent memo removed 89% of `normalize` instructions |
| exp-052 | Accepted instrument | +0.03% cold wall, neutral | Per-layer counters were below the measurement’s visible cost |
| exp-053 | Accepted instrument | −1.26% idle wall, neutral | Runtime toggle separated compiled, idle, and recording costs |
| exp-054 | Accepted validation | −15.68% warm wall | Linux warm gains transferred to macOS; the cold effect remained neutral |
| exp-055 | Accepted validation | −0.95% cold wall, neutral | Correctness and safety review fixes showed no detected macOS regression |

## 6. Wins by Mechanism

Individual percentages use different controls and regimes, so they cannot be summed.
Cumulative experiments measure composition directly.

| Mechanism | Representative evidence | Why it worked | Cost or boundary |
| --- | --- | --- | --- |
| Overlap unavoidable metadata latency | exp-001: −50.0% cold wall | Four bounded producers waited concurrently on filesystem metadata | CPU +58%; the right depth depends on latency and scale |
| Stop rebuilding paths | exp-004: −9.4% warm; exp-005: −18.6% load; known-parent study: −51.9% load | Caller already held components or parent identity | Requires explicit identity-preserving APIs |
| Replace repeated strings with IDs | exp-008: −15.7% cold | Extensions became compact integer identity | Helps retained index work, not raw enumeration |
| Fuse passes over the same bytes | exp-009: −12.4% loader component | Checksum and parse shared one read | Wall was diluted by probe/oracle overhead |
| Adapt concurrency to observed service time | exp-021: −5.3% large-tree cold | Initial latency predicted whether more workers could pay | Avoided an entry-count threshold that fired too early |
| Change the system-call shape | exp-022: −30.1% cold index, −41.6% producer | `getattrlistbulk` returned directory entries and metadata together on macOS | Platform-specific unsafe parser and tuning evidence |
| Reuse bulk observations across phases | exp-026: −34.4% warm | Revalidation stopped repeating metadata system calls | macOS-specific mechanism |
| Eliminate work before serialization | exp-030: −59.5% large warm | Workers discarded unchanged entries before the mutation authority | Parallelism paid only after the no-op proof moved outward |
| Retain only consumable state | exp-040: −14.6% wall, −95.3% RSS | One-shot summary stopped building a reusable index | Does not replace indexed queries |
| Remove a decode copy | exp-050: −12.0% Markdown wall | Complete UTF-8 chunks decoded in the input buffer | Specific to content work; other content jobs stayed neutral |
| Reuse the previous parent | exp-051: −7.35% Linux cold wall, −16.6% component | Directory children arrive consecutively, so parent identity has strong locality | Warm revalidation was unchanged |

Three supporting studies fall outside the 56-row ledger and appear separately in the
interactive companion:

- skipping a byte-identical snapshot rewrite improved Linux warm-tree wall 20.6% and
  reduced peak RSS from 411.2 MiB to 194.7 MiB;
- loading snapshot children beneath the already-known parent improved the loader 51.9%
  and complete warm open 41.9%; and
- hash lookup for content-cache candidates improved warm content open 3.48% and the
  cache-only path 3.03%.

Future campaigns should either record such studies in the main experiment contract or
give supporting studies a sibling contract with the same regime and verdict fields.
Their prose evidence remains useful but is harder to aggregate automatically.

## 7. Rejected Ideas and Their Limits

The 24 rejected artifacts cluster into mechanism families that constrain future work
more usefully than a flat list of red percentages.

### More Concurrency Failed After Latency Fell

Exp-015 showed that extra scan workers helped a large pressure tree before macOS bulk
metadata. After exp-022 removed much of the metadata latency, exp-025’s sixteen workers
regressed wall 19.2%, roughly doubled CPU, and added about one-third peak RSS. At one
million entries, exp-036 found only 1.3% wall improvement from eight workers for 33.5%
more CPU; 12 and 16 workers became slower.

Useful concurrency depends on the current service-time regime.
A structural I/O win can invalidate the worker count that preceded it.

### Rearranging Descriptors Did Not Delete System Calls

Root-relative opens (exp-024) and a parent-relative `openat` frontier (exp-038) changed
descriptor arrangement but not the number of expensive directory/metadata operations.
Both were neutral and added descriptor-lifetime or unsafe-boundary complexity.
Linux raw `readdir`, direct `statx`, and inode-ordering screens reached the same
conclusion: the standard library was already close to the useful kernel shape.

Batching `statx` through `io_uring` regressed warm wall 327% and a virtualized cold
screen 77.6% because queue setup, submission, completion, and `io-wq` overhead
rearranged the same cached work rather than removing it.

### Local Allocation Tweaks Stayed Below the Filesystem Floor

Skipping journaling (exp-003), moving rather than cloning producer paths (exp-016),
reusing bulk staging (exp-028), and increasing the bulk buffer (exp-029 and exp-039)
were all plausible local improvements.
None produced a claim-grade wall gain, and several worsened memory or faults.

Allocation still mattered when a change deleted an entire allocation family: the
known-parent and byte-identical-rewrite studies both won, while allocator substitution
exposed a cross-thread-free pattern.
Small local allocation counts, however, did not predict wall time on the I/O path.

### Lower CPU and Memory Did Not Guarantee Lower Latency

Exp-041 cut user CPU 36% and RSS 35% but improved wall only 1.4%. Exp-042 likewise
reduced resource signals without improving elapsed time.
Exp-044’s specialized selected-size path gained only 1.1%, did not beat the reference
tool, and required a second unsafe parser plus a public view.

Together, these results expose a remaining filesystem and directory-open floor.
Lower internal work did not justify a second implementation when user-visible latency
stayed flat.

### Content Optimizations Missed Until the Byte Boundary Moved

Inline analysis in exp-047 regressed wall 66.3% because the existing worker pool was
performing useful parallel reads.
Collector gating and source reserve in exp-048 and exp-049 were neutral or too
uncertain.
Exp-050 moved the byte boundary by removing a complete UTF-8 decode copy while
preserving the parallel I/O architecture.

### An Isolated Benchmark Win Can Still Lose the Product Decision

The Linux mimalloc screen improved aggregate wall 23.0%, but index and snapshot-load
intervals crossed zero, aggregate peak RSS rose 139%, the dependency builds C, and macOS
was unmeasured. It was not adopted.

Changing allocator policy helped where local allocation-count reductions had not.
That contrast points toward cross-thread allocation and free behavior as a mechanism to
investigate without adopting a global allocator dependency.

## 8. Why the Loop Converged

### Caller Trees Exposed Redundant Work

The strongest wins removed callers’ demand for hot helpers.
Caller trees showed a parent already in a local variable being reconstructed from a
path, or the same bytes being traversed twice.
Removing that demand produced the large effects.

Flat profiles repeatedly overvalued hot callees.
A content-cache map looked large until the caller tree showed it was only 0.9% of
instructions.
The eventual hash-map change still cleared 3%, but by an order of magnitude
less than a flat view suggested.

### Cumulative Anchors Measured Composition Directly

Exp-006, exp-023, exp-027, exp-032, and the integration/scale validations compared a
current candidate with an earlier real binary.
They answered whether individually accepted changes still composed after code and regime
changes. Without those controls, adding headline percentages would overstate the result
and hide interactions.

The absolute checkpoints expose the same history without converting local percentages
into a fictional cumulative line.
In the first exact macOS regime, cold indexed wall fell from 627.5 ms at exp-000 to
320.9 ms at exp-006, while warm wall moved from 795.2 ms to 688.0 ms.
In a later exact regime, exp-023, exp-027, and exp-032 measured cold wall at 295.5,
275.4, and 289.6 ms, while warm wall fell from 631.6 to 536.5 to 441.6 ms.

The cold series is not monotonic because the campaign did not optimize one score.
Several accepted changes targeted warm reconciliation, snapshot loading, behavior, or
instrumentation; independent run medians also retain host noise.
A checkpoint reports the observed surviving state across every dimension.
Only the paired comparison beside it attributes a movement to that experiment.

### Separate Jobs Localized Each Mechanism

The harness names whole-wall and component jobs separately.
That prevented the Linux producer wall movement from being misreported as producer work,
and it let exp-009 pass on its preregistered loader-component signal despite a wall
interval diluted by harness and oracle work.

### Exact Oracles Expanded the Safe Search Space

Every trial records stable tree identity and compares exact results.
This made unsafe platform parsing, traversal-order changes, parallel reconciliation, and
specialized summary paths experimentally tractable.

### Rejections Narrowed the Next Search

The 24 rejected artifacts preserve the conditions under which each idea failed.
Exp-011 records a dependency between experiments: fewer ancestor merges should help in
isolation, but extension interning had already made each merge cheap.
The rejection records that dependency, so the hypothesis can be reconsidered only if the
cost shape changes again.

### The Instrument Was Measured Too

Counters became a runtime capability only after idle and recording costs were separated
and bounded. External syscall counts were used to audit application counters.
Tests assert counters against system totals across serial, parallel, and platform bulk
paths, because a plausible zero is more dangerous than a missing field.

## 9. Evidence Gaps and Open Questions

### Platform Coverage Is Asymmetric

Fifty-three experiment artifacts are macOS and three are Linux.
macOS tuning spans several real APFS regimes; all current Linux measurements are
virtualized. There is no Windows performance evidence.
Windows builds and tests, but no timing claim should be inferred from that.

Virtualization is especially important for cold-device hypotheses.
Dropping a guest page cache does not drop the hypervisor or host cache, so
inode-ordering and physical-I/O claims remain unresolved until bare-metal Linux
measurement.

### Absolute Iteration History Is Incomplete

The original protocol required enough jobs to decide each hypothesis, but it did not
require one full metric matrix or exact surviving source revision after every verdict.
That is a serious historical omission: the repository cannot honestly produce a
continuous, multi-dimensional absolute trend for the whole campaign from the retained
artifacts alone.

| Absolute checkpoint dimension | macOS | Linux |
| --- | ---: | ---: |
| Cold indexed wall | 36 / 41 | 3 / 3 |
| Warm revalidation wall | 27 / 41 | 2 / 3 |
| Warm snapshot-load wall | 12 / 41 | 0 / 3 |
| Cold indexed CPU | 36 / 41 | 3 / 3 |
| Cold indexed peak RSS | 36 / 41 | 3 / 3 |

Only 21 of all 44 checkpoints have an exact source revision recoverable from the
artifact or Git history.
Even populated cells cross several hosts, trees, and scale regimes, so a single
connected line would still be false.
The interactive chart leaves missing cells marked and breaks lines at every regime
boundary.

Future artifacts close the gap by requiring `index-core-v1`, the kept arm, the full
source revision, and all five jobs.
`make perf-replay-revisions` can backfill a chosen Git sequence under one controlled
corpus and current toolchain.
A backfill remains a new measurement, not a retroactive claim about the original host or
compiler.

### Supporting Studies Use a Different Contract

The Linux byte-identical-rewrite, known-parent loader, content-cache, allocator, and
`io_uring` studies are well documented but are supporting studies rather than rows in
the 56-artifact experiment ledger.
They remain labeled as supporting studies and are excluded from the 56-experiment count.
A sibling contract could normalize them while preserving their different regimes and
evidence grades.

### The Harness Can Enter the Profile

A probe’s own verification digest accounted for 38.8% of one profile.
That does not invalidate paired end-to-end timing, but it can hide the engine function a
profile is meant to explain.
Profiles must identify and subtract or separate oracle work before attributing shares.

### A Neutral Interval Is Not Proof of Zero

The instrumentation experiments bound overhead below what the current sample can see.
Exp-054’s macOS cold interval permits a small improvement or regression.
The terms *neutral* and *not detected* preserve that uncertainty; *free*, *unchanged*,
and *zero* would overstate the evidence.

### The Aggregate Is a Decision Aid, Not a Result

The 12.7% composite is derived from heterogeneous cells.
Its usefulness depends on visible weights and source measurements.
No graph should connect it to experiment points as though all were observations on one
machine.

### One Experiment Remains Open

Exp-046’s two-opener macOS pool cleared a short screen at −4.0%, but context switches
doubled and later runs encountered extreme host outliers.
It needs a quiet 12-pair run and an independent topology before a verdict.
Until then, its point remains gray.

## 10. A Reusable Performance-Research Template

### Campaign Setup

1. Define the user-visible operation and its state tiers.
2. Enumerate platform, host, filesystem, cache, scale, and virtualization regimes.
3. Establish exact correctness and subject-identity oracles.
4. Separate end-to-end jobs from mechanism-isolating component jobs.
5. Version the post-decision checkpoint profile and require it after every terminal
   experiment.
6. Measure the harness and instrument before optimizing product code.
7. Publish the acceptance rule before the first candidate.

### Hypothesis Card

Every hypothesis should answer:

| Field | Required content |
| --- | --- |
| Observation | Profile or counter evidence that creates the question |
| Mechanism | Work the candidate removes, overlaps, or makes contiguous |
| Scope | Platform, tier, start state, tree shape, and scale |
| Prediction | Primary metric, expected direction, and approximate magnitude |
| Falsifier | Result that would reject the mechanism, not merely the patch |
| Secondary risks | CPU, memory, faults, dependencies, unsafe code, and semantics |
| Component probe | Smaller job that should move if the mechanism is real |

### Experiment Artifact

Use one soft-schema Markdown artifact per attempt.
The authoritative envelope should contain, at minimum:

- identity and hypothesis references;
- subject and host regime;
- exact control and candidate;
- paired schedule and artifacts;
- per-job metrics and intervals;
- oracle validity;
- effect direction and product decision as separate fields;
- checkpoint profile, kept arm, and exact source revision; and
- complexity and landing/revert provenance.

Use the Markdown body for rationale, profiler interpretation, anomalies, and follow-up
questions. Never ask a generated report to recover structured values from prose.

### Decision Sequence

1. Run the correctness matrix before timing.
2. Run paired, interleaved claim-grade trials.
3. Inspect primary and component metrics together.
4. Classify direction: improved, regressed, unclear, or unknown.
5. Apply the predeclared threshold and complexity trade.
6. Land or revert the candidate.
7. Run the complete checkpoint profile on the surviving state.
8. Record the kept arm and exact source revision.
9. Commit the artifact even when rejected.
10. Run a cumulative anchor after a coherent group of changes.
11. Re-profile the new baseline before selecting the next hypothesis.

### White-Paper Structure

A reusable campaign report should contain these sections in this order:

1. abstract and current measured outcome;
2. system and workload model;
3. measurement and acceptance protocol;
4. soft-schema evidence contract;
5. complete chronological experiment map;
6. wins grouped by mechanism;
7. failures grouped by mechanism;
8. cross-platform transfer and non-transfer;
9. method retrospective;
10. evidence limitations and unresolved work;
11. next hypotheses; and
12. source documents and reproducibility commands.

Generate the graphical companion from the same artifacts, with:

- a clearly labeled current-outcome panel;
- an absolute kept-state small-multiple chart before local percentage effects;
- simultaneous platform lanes with independent raw units;
- green, red, and neutral local evidence with accessible details;
- connections only between compatible absolute regimes;
- an adjustable aggregate whose weights and formula are visible;
- mechanism-level win/failure roll-ups; and
- a searchable schema-directed artifact table.

### Definition of Done

A campaign is ready to hand off when:

- every attempted candidate has an artifact and verdict;
- every terminal checkpoint names a profile, kept arm, full source revision, and
  complete metric matrix;
- generated views match authoritative frontmatter;
- cumulative outcomes are rerun on each claimed platform;
- neutral and environmental results are labeled conservatively;
- the report names missing regimes rather than silently extrapolating;
- Markdown and interactive views remain usable without network access; and
- formatting, schema validation, tests, and generated-data drift checks pass.

## 11. Next Experiments

The next useful experiments target structural work instead of another round of buffer
constants.

1. **Finish exp-046 under quiet macOS conditions.** Resolve the shared opener pool
   before building new work on it.
2. **Add bare-metal Linux evidence.** Revisit physical-I/O hypotheses only where cache
   state can be controlled without host-cache ambiguity.
3. **Preserve parent identity across the cold producer boundary.** Exp-051 and the 51.9%
   loader study show the same re-derivation pattern in two paths.
4. **Investigate producer ownership for cross-thread frees.** Treat mimalloc as a
   diagnostic that motivates a dependency-free ownership change.
5. **Reduce duplicate retained names and per-directory map allocation.** Target
   million-entry RSS and index locality together.
6. **Measure classification from interned extension identity.** The content tier still
   reclassifies paths whose extension is already stored.
7. **Promote supporting studies into a sibling soft-schema contract.** Preserve their
   different evidence grades while making them discoverable and renderable.
8. **Prototype the generalized table as a Metabrowser plugin only after the static
   projection stabilizes.** Keep domain schemas in the plugin and consume the public
   host SDK and design tokens.

## 12. Sources and Reproduction

Source documents:

- [performance campaign status](report-2026-08-14-performance-campaign-status.md) for
  the current branch outcome and evidence gaps;
- [experiment ledger](report-2026-08-10-fdu-performance-experiments.md) and the
  [experiment artifacts](../experiments/) for all 56 structured verdicts;
- [performance loop](../guides/performance-loop.md) for the protocol and live
  hypotheses;
- [instrumentation playbook](../guides/performance-instrumentation-playbook.md) for the
  reusable three-tier method;
- [performance architecture](report-2026-08-12-fdu-performance-architecture.md) for the
  engine cost model;
- [platform tuning](../guides/platform-tuning.md) for regime provenance;
- [Linux first measurements](../research/research-2026-08-13-linux-first-measurements.md)
  for syscall and product scouting; and
- [Linux three-tier baseline](../research/research-2026-08-13-linux-three-tier-baseline.md)
  for aggregate, index, and content comparisons.

Repository Make targets regenerate the committed HTML data from experiment frontmatter
and check it for drift.
The HTML has no network dependency, so a checkout preserves the visual evidence beside
its source artifacts.

Replay a source-history window by resolving its immutable plan first:

```shell
make perf-replay-revisions ARGS='\
  --range 4af00b0..HEAD --path crates/fdu \
  --root /path/to/frozen/tree --label index-history \
  --baseline-fingerprint /path/to/tree-fingerprint.json \
  --name index-history --dry-run'
```

Run the same command without `--dry-run` after reviewing the full commits and five jobs.
The runner archives rather than checks out each revision, builds every probe under one
current locked toolchain, then measures all binaries in one interleaved schedule.
Repeat the resolved commit list on macOS and Linux with platform-local frozen corpora.
The `4af00b0` anchor contains the committed real-tree probe and is production-engine
equivalent to the earlier `b565882` campaign baseline; replaying an older revision
requires a documented compatible probe reconstruction.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
