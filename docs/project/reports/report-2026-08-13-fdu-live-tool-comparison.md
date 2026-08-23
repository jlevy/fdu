# fdu Live Uncached Tool Comparison

**Date:** 2026-08-13

**Status:** Local performance evidence

## Outcome

On a self-contained 901,963-entry tree, a fresh fdu process with its persisted cache
disabled built a reusable exact index and rendered a depth-one, ten-row tree in a
3.324-second median.
The maintained tree and index peers took 5.657–20.550 seconds.

| Tool | Work class | Median wall | Versus paired fdu | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| **fdu** | reusable exact index and 10-row tree | **3.324 s** | baseline | — | 398.0 MiB |
| pdu | rendered depth-one tree | 5.657 s | +71.3% | +66.2% to +74.9% | 13.3 MiB |
| dust | rendered 10-row tree | 6.016 s | +78.3% | +69.3% to +80.2% | 402.3 MiB |
| gdu | rendered 10-row tree | 6.782 s | +107.1% | +99.2% to +110.4% | 397.1 MiB |
| ncdu | browseable index, UI disabled | 20.550 s | +513.6% | +497.2% to +523.0% | 2.0 MiB |

fdu also has a derived execution plan for the existing cache-off, unfiltered summary
composition. It retains only the exact five summary tallies and does not build a path
index. That richer summary took 3.125 seconds and was faster than every scalar tool
except dumac, which was statistically tied in the paired comparison.

| Tool | Work class | Median wall | Versus paired fdu | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| **fdu** | files, dirs, both byte totals, newest time | **3.125 s** | baseline | — | 13.6 MiB |
| dumac | allocated-byte total only | 2.980 s | -2.2% | -5.7% to +1.7% | 44.4 MiB |
| diskus | scalar total only | 5.708 s | +81.7% | +74.9% to +87.1% | 28.9 MiB |
| dua | scalar total only | 5.459 s | +81.1% | +67.5% to +91.8% | 22.0 MiB |
| BSD du | scalar total only | 13.075 s | +321.5% | +285.0% to +351.8% | 2.0 MiB |
| GNU du | scalar total only | 20.426 s | +561.7% | +552.3% to +601.4% | 6.6 MiB |

Positive percentages mean the competitor took longer than the immediately adjacent fdu
run. The paired percentages are stronger than differences between the displayed medians
because they control for filesystem-cache and machine drift.

## Method and Validity

The subject was the repository’s `explorations/benchmarks/` subtree.
It contains the ignored generated corpus, benchmark environment, harness, schemas, and
prior result artifacts, but excludes the more volatile repository build and Git state.
Its immediate redacted fingerprint contained 110,369 directories including the root,
791,261 regular files, 333 symlinks, and no other entries to depth 23. It held
16,537,467,483 apparent bytes and 18,714,214,400 allocated bytes.

The benchmark used an Apple M1 Pro MacBook with 32 GiB RAM and a local APFS SSD. The
operating system’s filesystem cache was in its ordinary warm-steady state.
“fdu cache off” means no fdu snapshot was read or written; it does not imply that APFS
metadata was purged.

### What Warm-Steady Means

This run did intentionally benchmark a warm operating-system filesystem cache.
The immediate independent fingerprint was itself a complete metadata walk over all
901,963 entries. Every comparator then received three complete warmup traversals before
its twelve timed pairs; fdu ran beside every one of those warmups.
That is at least as explicit as the three-warmup Hyperfine protocol used in the
[dumac benchmark](https://healeycodes.com/maybe-the-fastest-disk-usage-program-on-macos)
and differs from merely rerunning a command and assuming the cache is warm.

“Warm-steady” does not mean the whole tree fits in a metadata cache.
A follow-up observation on the same host found `kern.maxvnodes=263168`, well below the
subject’s entry count.
That value is a global target rather than a per-tree hard limit, and `getattrlistbulk`
can avoid materializing a vnode for many entries, so it cannot measure a cache-hit ratio
by itself. It does show why “all metadata resident” would be an unjustified claim.
The measured state is a repeated cyclic traversal under realistic cache pressure.

Andrew Healey’s choice to publish warm-cache macOS numbers is valid for that stated
regime, and it matches the common case of rerunning a local interactive tool.
His stronger statement that warm performance correlates with cold performance is
plausible as a ranking observation but is not demonstrated by cold raw samples in the
article or repository.
It is not safe to infer the size of a cold-cache advantage from it.
The
[diskus benchmark](https://github.com/sharkdp/diskus/blob/90196e950017d25b2940e8e0fda51a321ca66e1a/README.md#benchmark)
illustrates the distinction: diskus keeps the lead over `du` in both Linux regimes, but
the published relative gap changes from 10.18× cold to 2.20× warm.

Both regimes are useful.
Warm-steady is the primary local-product and optimization comparison because it exposes
syscall, scheduling, and user-space constants without letting SSD state dominate.
Controlled-cold measures first access after eviction, memory-constrained or much-larger
trees, and remote/provisioned storage.
The queued Linux matrix will publish both, using `sync` plus `drop_caches` before every
cold sample and explicit warmups for the separate warm matrix.
On macOS, `/usr/sbin/purge` only promises an approximation of initial-boot buffer-cache
conditions, so a purge run must be labeled separately; a dedicated APFS volume remounted
between samples is the stronger future cold protocol.

The two matrices each used three warmups and twelve timed pairs per competitor.
Every competitor ran immediately beside the same immutable fdu binary, and pair order
alternated at each ordinal.
In total, 270 fresh processes traversed the subject over 32.5 minutes.
The scalar and tree matrices had separate immediate pre/post fingerprints because they
are separate runs.

Both matrices observed the same independent digest before and after:
`14dc5055091e31c0ebdd813ef0f036d25fe5673daed09c449721ff1cebbb98b7`. There were no
invalid timed samples, semantic mismatches, baseline drift, or mutations.
All sixty timed fdu summary anchors produced one stable semantic digest after excluding
only generator, root, and timestamps.
Each sample independently matched the Python fingerprint’s regular-file count,
descendant-directory count, apparent bytes, allocated bytes, and newest-file time.
Partial, stale, cached, or error-bearing reports are invalid by construction.

The natural human tree command was timed because it is the product being compared.
An untimed JSON invocation of the same immutable binary separately reported a complete,
error-free root with exactly 791,261 files, 110,368 descendant directories,
16,537,467,483 apparent bytes, 18,714,214,400 allocated bytes, and newest-file mtime
1,786,626,096,603,125,419 ns.
External tools were required to exit successfully with stable output, but their totals
are calibration references rather than fdu correctness oracles.

The measured fdu executable was commit `33af4a868`, version `fdu 0.0.1-dev+g33af4a868`,
with SHA-256 `dd4d8a0030ae5967f275c6a38e219ec9e1364020f030b12f32568ddd2ed5a0f5`. Exact
commands, versions, hashes, host facts, tree facts, resources, and intervals are in the
[reproduction manifest](fdu-live-tool-comparison-manifest-v2.json).
The operational procedure is in the
[performance harness README](../../../explorations/benchmarks/README.md).

## Why the Tree Product Leads

fdu, dust, gdu, and pdu all traversed the full tree and produced directory roll-ups.
fdu additionally retained stable entry identity and exact metadata for later views,
queries, snapshots, and incremental changes.

The process metrics and source review agree on the main difference.
Dust, gdu, and pdu use portable recursive parallelism and consumed 38.0–47.3 aggregate
CPU-seconds on this subject.
fdu used 13.1 seconds.
Its macOS backend obtains directory enumeration and the complete stat-tier attributes in
`getattrlistbulk` batches, avoiding one metadata system call and repeated path
resolution per file.
The breadth-first region scheduler and six-worker policy then overlap directory latency
without the CPU and contention costs that the worker-depth experiments measured at
larger pools.

Pdu demonstrates a different tradeoff.
Its 13.4 MiB peak RSS is excellent because it does not retain fdu’s complete reusable
index, but it took 71.3% longer in paired wall time.
Compacting fdu’s retained tree remains a worthwhile memory project even though the
current construction path is faster.

## What the Dumac Result Means

Dumac’s median was lower because it requests and retains much less: one selected
allocated-size metric plus inode state for hard-link deduplication.
Its paired wall change was -2.2%, but the -5.7% to +1.7% interval makes this a
statistical tie rather than evidence that either tool leads.
fdu’s summary also returns exact file and directory counts, apparent bytes, and newest
file time, with strict malformed-record, mount, firmlink, one-filesystem, non-UTF-8
name, partial-result, and portable-fallback behavior.

The wall advantage does not come from using fewer machine resources.
Dumac consumed 85.4% more aggregate CPU, 87.8% more system CPU, and 224.5% more peak RSS
than fdu in the paired samples.
It finished slightly earlier by sustaining more concurrent kernel work.
fdu’s richer summary used 13.6 MiB and statistically remained close to the same
directory-open and bulk-syscall floor.

Experiment 044 tested the obvious matched-workload response rather than assuming it
would help. A typed selected-total prototype requested only the chosen size metric,
folded files inside the bulk buffer, and retained names only for subdirectories.
It cut user CPU 51.5% and RSS 39.2%, but improved wall only 1.15% with an interval
crossing zero, did not beat dumac, and required a second unsafe parser plus a new public
view. The prototype was reverted.
The existing rich summary remains the smallest useful execution tier.

This is not evidence of a fundamental limit or of a remaining proved fdu deficit.
`getattrlistbulk` batches the children of one open directory; it does not traverse
several directories in one call.
This subject has 110,369 directories including root, so both implementations still pay a
directory-open and at least one bulk-call boundary for every directory they visit.
Experiment 044 removed most of the plausible fdu-only user-space work without changing
system CPU or elapsed time materially.
Dumac’s lower point estimate came with 87.8% more system CPU, consistent with pushing
more concurrent work through the same kernel boundary rather than executing fewer
required tree steps.
fdu separately tested dumac-like 128 KiB buffers, depth-first order, smaller directory
claims, and deeper worker pools; none produced a confirmed wall improvement.

The claim for this published matrix is therefore that the two tools match within its
resolution and the current fdu path is kernel/topology-bound on this tree.
H67 later replayed both the current and published binary pairs under a much busier
interactive host. Dumac led current fdu by 16.19% in twelve pairs and the published fdu
binary by 11.1% in a five-pair diagnostic, ruling out the intervening
reconciliation-only change as the cause.
fdu sustained 3.46 aggregate core-equivalents versus dumac’s 5.64. Exact process samples
put 96.10% of fdu worker tops and 94.21% of dumac worker tops in `open` or
`getattrlistbulk`, with both main threads waiting for workers.

That follow-up does not replace this quiet-host matrix; it establishes that the relative
wall result depends on host pressure and available concurrency.
H69 tested a genuinely different overlap strategy: six scan and parser workers plus two
bounded open-only helpers.
Its corrected five-pair point estimate was −4.47%, but the busy-host interval
[−31.04%, +33.91%] could not support retention.
H70 replaced the pairwise handoff with a shared pool.
Two openers improved wall 3.98% [0.70%, 9.87%] and aggregate CPU 15.98% in one exact
five-pair screen, at the cost of 111.80% more involuntary context switches.
A two/three/four-opener sweep and direct twelve-pair four-opener comparison with dumac
were too noisy to select a count; that four-opener fdu/dumac comparison was a wall-time
tie while dumac used 40.68% more aggregate CPU and 223.23% more peak RSS. A different
platform primitive or journal-scoped work may still win; neither a noisy gap nor a short
screen justifies duplicating an unsafe parser or weakening semantics.

This subject contained no duplicate in-tree hard-link paths, so hard-link deduplication
did not affect its regular-file total.
It did contain 333 symlinks.
fdu excludes symlinks from regular-file roll-ups, while the reviewed dumac revision
counts each as one 512-byte block.
That 170,496-byte semantic difference is negligible for timing but must remain visible
in any total comparison.

## The Linux Walker-Level Companion

This matrix compares *products* on macOS: whole tools, each returning its own work
class. A separate Linux measurement compares the *walkers* underneath them — fdu’s
against [`ignore`](https://docs.rs/ignore), which is ripgrep’s, plus `walkdir` and
`jwalk` — and reads all of them against a hand-written parallel syscall floor.
Its results are in
[the metadata-walk floor report](report-2026-08-23-metadata-walk-floor.md).

Two of them bear on this document.
fdu’s aggregate tier sits at 1.20–1.59× that floor depending on the tree, so the
headroom this comparison’s lead rests on is smaller than the lead.
And the peer ordering is not stable across subjects: fdu leads `ignore` by 12–26% on
generated trees, ties once real filenames appear, and trails by about 12% on `/usr`.
That report is scouting evidence on one virtualized host and does not meet the bar this
one does — no pinned binaries, no installation attestation, one real tree — so it orders
work rather than supporting a claim.

## Limits and Next Work

This is a local claim for one near-million-entry tree, one M1 Pro, one local APFS SSD,
and warm-steady operating-system cache state.
It is not a controlled-cold-disk result, a Linux result, or a universal rank.
The Linux matrix remains open and will report verified-warm and per-sample
controlled-cold regimes separately.

The results close several low-value macOS paths.
Larger bulk buffers, deeper worker pools, depth-first traversal, parent-relative
descriptor frontiers, worker-local summary reduction, narrower rich-summary records, and
a selected-total scanner all failed their wall-time gates.
The remaining high-impact queue is therefore architectural:

- confirm a shared bounded directory-opener pool on a quiet host and independent
  topology
- compact the reusable full-index entry layout and separate directory-only state
- construct disjoint worker-local subtrees and splice them in bounded region units
- test a dense immutable bootstrap with a sparse mutation overlay
- profile a distributed scheduler only if queue waiting remains material after those
  layout changes
- test portable wide-directory stat chunks on Linux, where per-entry metadata calls
  still exist
- use FSEvents to scope warm work to changed regions while retaining this fast full scan
  as the first-run and fallback path

The concise architectural synthesis is in the
[performance white paper](report-2026-08-12-fdu-performance-architecture.md), and every
accepted and rejected experiment is in the
[experiment ledger](report-2026-08-10-fdu-performance-experiments.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
