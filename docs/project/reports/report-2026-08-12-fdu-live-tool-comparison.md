# fdu Live Uncached Tool Comparison

**Date:** 2026-08-12

**Status:** Local performance evidence

## Outcome

On a 976,295-entry workspace, a fresh fdu process with its persisted cache disabled
produced a depth-one, ten-row tree in a 4.237-second median.
It was faster than every other tool that produced a tree or retained index.
Dust took 7.546 seconds, pdu 6.684 seconds, gdu 8.315 seconds, and ncdu 28.576 seconds.

Dumac was the only faster program at 3.566 seconds.
It performs a narrower job: one hard-link-deduplicated allocated-byte total.
fdu retains an exact entry index and per-directory roll-ups for subsequent size, count,
recency, type, query, snapshot, and incremental-update operations.
The dumac result is therefore an informative lower bound, not an equivalent-output loss.

| Tool | Work class | Median wall | Versus paired fdu | 95% interval | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| **fdu** | indexed tree and 10-row report | **4.237 s** | baseline | — | 585.2 MiB |
| pdu | rendered tree | 6.684 s | +60.9% | +49.8% to +82.4% | 16.7 MiB |
| dust | rendered tree | 7.546 s | +80.3% | +65.5% to +102.2% | 455.9 MiB |
| gdu | rendered tree | 8.315 s | +93.3% | +74.7% to +105.4% | 431.6 MiB |
| ncdu | indexed tree, UI disabled | 28.576 s | +560.2% | +501.2% to +712.5% | 2.0 MiB |
| dumac | scalar total only | **3.566 s** | -17.6% | -27.1% to -9.1% | 45.3 MiB |
| diskus | scalar total only | 7.064 s | +66.9% | +58.3% to +77.8% | 34.0 MiB |
| dua | scalar total only | 8.352 s | +92.5% | +77.0% to +128.2% | 63.3 MiB |

Positive percentages mean the competitor took longer than the immediately adjacent fdu
run. Medians in the table summarize each tool’s samples; paired percentages are the
stronger comparison because they control for filesystem-cache drift.

## Method and Validity

The subject was the project workspace, including its ignored benchmark corpus, build
output, dependency trees, Git data, and pinned comparator source checkouts.
Its redacted fingerprint contained 113,025 directories, 862,911 files, 359 symlinks, and
no other entries to depth 24. It held 24,311,032,049 apparent bytes and 26,647,465,984
allocated bytes.

The benchmark used an Apple M1 Pro MacBook with 32 GiB RAM and a local APFS SSD. It ran
with the operating system’s filesystem cache in its ordinary warm-steady state.
“fdu cache off” means no fdu snapshot was read or written; it does not mean the APFS
metadata cache was purged.

Each competitor ran next to the same immutable fdu binary, with order alternating at
each ordinal.
The run used three warmups and twelve timed pairs per competitor: 210 timed
process invocations over 71 minutes.
Every process started fresh.
fdu, dust, gdu, and pdu emitted depth-one trees; fdu, dust, and gdu were limited to ten
rows. Ncdu built its index with the interactive UI disabled.
The scalar tools scanned the complete tree and emitted one total.

The pre-run and post-run independent tree digests were both
`c80d469fce7f831edd27da7a3da2a9a95613db3d7de745c5181c2240344e9312`. There were no
invalid timed samples, no baseline drift, no tree mutation, and exactly one stable
output digest per tool.
The tree contained 43,028 duplicate hard-link paths across 12,805 inode groups,
representing 3,138,498,560 path-counted allocated bytes.
Tools differ in hard-link attribution, so this is a traversal comparison rather than an
assertion that their byte totals are equal.

The measured fdu executable was commit `aeac4875d`, version `fdu 0.0.1-dev+gaeac4875d`,
with SHA-256 `bc6c69c0ac777e9ea7653ece1931e79a433fca63556dca516e3f76288b5ff910`. Exact
commands, versions, hashes, host facts, tree facts, medians, and paired intervals are in
the [reproduction manifest](fdu-live-tool-comparison-manifest-v1.json).
The operational procedure is in the
[performance harness README](../../../explorations/benchmarks/README.md).

## Why fdu Is Faster Than the Tree Renderers

The source review and process metrics agree on the explanation.
Dust, gdu, and pdu use portable recursive parallelism and spend 35–43 aggregate
CPU-seconds on this subject.
fdu uses macOS `getattrlistbulk` to enumerate a directory and retrieve its complete
stat-tier metadata in batches, then feeds one bounded index consumer.
It used 19.7 aggregate CPU-seconds while still retaining the complete entry inventory
and roll-ups. The advantage is fewer kernel transitions and less repeated path
resolution, not omitted entries: the independent fdu oracle validated the exact
976,295-entry result, and the pre/post fingerprint guarded the shared subject.

The breadth-first region scheduler also remained the right operating point after the CLI
merge. Experiment 036 found that eight workers gained only 1.30% while consuming 33.5%
more CPU; twelve and sixteen workers regressed.
Experiment 037 found depth-first traversal 3.57% slower on the same heterogeneous tree.
The external comparison therefore uses the measured six-worker automatic policy rather
than an unvalidated high-thread configuration.

## What the Faster Scalar Result Teaches Us

Dumac uses the same macOS bulk-enumeration family, but requests only name, object type,
file ID, and allocated size.
It reduces each subtree immediately to an integer, retains only a 128-shard inode set
for hard-link deduplication, and uses recursive Rayon work stealing.
fdu additionally requests device identity, flags, apparent size, mtime, and ctime; it
constructs stable paths and identities; and it retains queryable entries, directory
roll-ups, extension tallies, errors, provenance, and change-feed state.

That smaller contract explains both parts of the observed gap.
In the paired dumac arm, dumac used 15.7% less aggregate CPU and 67.4% less user CPU
than fdu. Its 45.3 MiB peak RSS was also far below fdu’s full index.
The 17.6% wall advantage is significant, but copying dumac’s scalar-only retention would
violate fdu’s one-scan-many-views and incremental-update design.

The useful response is to make the reusable index denser and cheaper to construct while
preserving its semantics.
The experiment queue now contains:

- **H19–H22 (`fdu-prph`):** measure and compact the full-index entry layout one field at
  a time
- **H58 (`fdu-r9he`):** test dua-style small, stealable metadata chunks on the portable
  backend, where per-entry stat calls remain
- **H59 (`fdu-hke6`):** design-gated bounded retention for cache-off reports, rejected
  if it makes output depth alter scan semantics or creates a CLI-only engine
- **H60 (`fdu-weey`):** construct worker-local subtrees and splice them at region
  completion, preserving stable identity and progressive publication
- **H61 (`fdu-f67r`):** prototype a dense immutable bootstrap with a sparse mutation
  overlay after the layout floor is known

Pdu’s low RSS motivates H59 and H60, dua’s four-entry completion chunks motivate H58,
and dumac’s small retained state raises H19–H22 and H61. Dust, gdu, and diskus add no
new APFS threading hypothesis: the measured worker-depth curve already rejected that
path.

## Limits

This is a local performance claim for one heterogeneous million-scale tree, one M1 Pro,
one local APFS SSD, and warm-steady operating-system cache state.
It is not a controlled-cold-disk result, a Linux result, or the complete Phase 1 release
matrix. The full generated-corpus matrix, dedicated-host provenance, and
platform-specific collectors remain separate gates.

The tools also expose different products.
Work classes keep those differences visible, but only fdu’s result was checked against
fdu’s full semantic oracle.
External output stability and successful traversal are calibration evidence, not proof
that another tool shares fdu’s hard-link, error, or metric contract.
Dut was source-reviewed but not timed because its current implementation is Linux-only.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
