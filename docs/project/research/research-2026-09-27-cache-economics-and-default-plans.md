# Research: When the Cache and the Index Pay, on macOS and Linux

**Date:** 2026-09-27

**Author:** fdu project, with Claude Code

**Status:** Complete; the proposal awaits a decision

## Overview

fdu’s published speed numbers measure `fdu --cache off`, but the command people type is
`fdu .`, and on the million-entry Linux benchmark tree the two differ by a fifth.
The default writes a metadata snapshot that no later `fdu .` reads, and the default
`--view summary` builds a full per-entry index to report six numbers.
Neither cost shows in the headline, and both are paid on every run.

Separately, the Linux comparison
([report](../reports/report-2026-09-27-fdu-linux-tool-comparison.md)) found fdu’s
indexed tree slower than pdu and diskus, while on macOS
([report](../reports/report-2026-09-26-fdu-live-tool-comparison.md)) fdu is ahead of
every tool, dumac included.

This brief explains both from first principles, maps each common use case to the state
its answer actually needs, states when the cache pays on each platform, and proposes
default behavior. The decision it supports is whether ordinary one-shot runs should keep
building a retained index and writing a snapshot.

## Questions to Answer

1. Why does writing the snapshot cost so much on a plain metadata run?
2. Should basic tallies and the default tree build a retained per-entry index at all?
3. When does the cache pay, per platform and per use case?
4. Why is fdu fastest on macOS, even against dumac, but not on Linux, even with the
   cache off?
5. What should the defaults be?

## Scope

Included: warm-steady filesystem caches, the committed `balanced` 1,000,001-entry tree
(875,000 files, 125,001 directories), the README’s usual commands, the metadata snapshot
and the content sidecar.

Linux figures were measured for this brief on a 4-vCPU Firecracker KVM guest with ext4.
Rows marked **harness** come from the paired tool harness with its oracles; rows marked
**screen** are 5–10-run `hyperfine` means on the same quiet host, good for proportions
and not for published claims.
macOS figures are not re-measured here: they come from the
[2026-09-26 macOS result](../reports/fdu-live-tool-comparison-result-2026-09-26.json),
[the platform tuning guide](../guides/platform-tuning.md#snapshot-participation-is-a-cost-decision-and-apfs-reverses-its-conclusion),
and the recorded experiments cited inline.

The use-case table and the figures derived from it were measured on this branch after
merging the code-analysis work ([#133](https://github.com/jlevy/fdu/pull/133), whose
default tree is five levels deep with a 1% share floor) and shared metric resolution
([#137](https://github.com/jlevy/fdu/pull/137)). The snapshot anatomy, the cache
readers, and the CPU split were measured before that merge; neither change touches those
paths.

Excluded: cold-cache regimes (this host is virtualized, so cold reads say nothing about
the device), Windows, and content-analysis internals beyond what the cache decision
needs.

## Findings

### Every usual command, and what it pays for

| Use case | Command | What the answer needs | What fdu does today | Linux, fdu | Linux, fastest peer |
| --- | --- | --- | --- | ---: | ---: |
| Which directories are large? | `fdu .` | Roll-ups through depth 5, rows of at least 1% of the root | Full index, snapshot write, tree | 1.51 s | pdu 1.09 s |
| Same, cache disabled | `fdu . --cache off` | Same | Full index, tree | 1.25 s | pdu 1.09 s |
| Totals | `fdu . --view summary` | Six tallies, including the ignored share | Full index, snapshot write | 1.45 s | diskus 1.02 s |
| Totals, no ignore share | `--view summary --no-gitignore` | Five tallies | Streaming reducer, nothing retained | **0.93 s** | diskus 1.02 s |
| Stale build directories | `--kind dir --include node_modules …` | Matching directories with their roll-ups | Full index, snapshot write | not measured | no peer |
| Content metrics, repeated | `--analyze=lines` | Per-file records for unchanged files | Load snapshot, revalidate, restore sidecar | 8.8 s | no peer |
| Live tree | `--watch`, Python `open` | A retained index | Load snapshot, revalidate, keep index | 1.00 s to first answer (probe) | no peer |

All rows are screens except the peer columns, which agree with the harness matrices.
The peer for each row does the same user-visible job: pdu at its defaults renders a size
tree that likewise hides entries under 1% of the total (0.98 s when also limited to
depth 5), and diskus returns the total.

Two rows stand out.
The default tree pays 0.26 s over `--cache off` for the snapshot, and
0.16 s more than pdu for the index.
The default summary pays 0.21 s for a snapshot and 0.32 s for an index that the
`--no-gitignore` path shows it does not need.

### Re-screened on the integrated stack

On 2026-09-28 the same commands were screened again on the integrated stack’s release
build (`7acface5`: [#137](https://github.com/jlevy/fdu/pull/137) with the merged code
analysis and presentation work, plus H156), ten runs per arm on the same host.
The table’s proportions hold.
The raw samples are in
[the screen results](../reports/fdu-linux-screens-result-2026-09-28.json).

| Command | Snapshot regime | Linux, fdu | `--cache off` | Snapshot cost |
| --- | --- | ---: | ---: | ---: |
| `fdu .` | an earlier run’s snapshot exists | 1.51 s | 1.27 s | 0.24 s |
| `fdu . --view summary` | an earlier run’s snapshot exists | 1.51 s | 1.25 s | 0.26 s |
| `fdu .` | first run, empty cache directory | 1.69–1.81 s | 1.25–1.27 s | 0.47–0.54 s |
| `fdu . --view summary` | first run, empty cache directory | 1.69–1.75 s | 1.22–1.25 s | 0.47–0.50 s |
| `--view summary --no-gitignore` | none written | 0.95 s | — | — |

The first-run rows are the regime the next section describes: the kernel must allocate
every page of a fresh 79 MB file.
The refreshed tool matrices put pdu at 1.02 s and diskus at 1.04 s on the same host.

### Anatomy of the snapshot write

The snapshot is the whole per-entry inventory: per entry, a parent slot (4 bytes), a
kind (1), the name with its length (about 20 on this tree), and six 64-bit attributes
(48). That is 79 bytes per entry, 79,299,986 bytes for this tree, and a run pays for all
of it regardless of what the report showed.

| Stage | Where the time goes | Linux, 1M entries |
| --- | --- | ---: |
| Encode | Pre-order walk of the whole index into one growing buffer | about 100 ms (by subtraction) |
| Checksum | Table-driven software CRC-32C over every byte | 58 ms |
| Write | One 79 MB `write()` into the page cache | 20–40 ms with reusable pages; 340–380 ms when the kernel must allocate them |
| `fsync` | Forcing the file to the virtio device | 55–85 ms |
| Rename and cache-directory sweep |  | 5 ms |
| **Isolated save, probe** | encode + checksum + write + `fsync` | **230–250 ms warm; 670 ms first** |

The write figure is the kernel’s, not fdu’s: a Python loop writing 79 MB to a fresh file
on the same filesystem measured 378 ms on its first write and 20–33 ms thereafter, and
the first fdu run on a new cache directory measured 343 ms inside `write()`.

A repeated run over an unchanged tree skips the write but not the cost.
[exp-067](../experiments/exp-067-skip-the-identical-snapshot-rewrite-on-the-cold-scan-path.md)
(`fdu-2um8`) removed the identical rewrite, but proving the image identical still means
encoding all of it, reading the existing 79 MB back, comparing it byte by byte, and
checksumming it: `fdu .` measured 1.51 s repeated against 1.25 s with the cache off
(**screen**).

All of it runs on one thread after the walk, and the command line joins that thread
before it exits, so every millisecond is on the user’s path.
The recorded Linux leftover analysis
([exp-147](../experiments/exp-147-linux-first-run-leftover-is-still-the-walk.md)) timed
the isolated save at 23.6 ms on a 92,474-entry tree, about 5% of a first run; at a
million entries the same stages cost a fifth of the run because the walk itself is so
cheap on ext4.

On macOS the write measured 90 ms over 175,128 entries (0.51 µs per entry, platform
tuning guide) and 45.3 ms over 158,705
([exp-135](../experiments/exp-135-post-h128-first-run-default-tree-leftover.md)), 11–16%
of a first run. Extrapolated to a million entries that is 0.3–0.5 s of the 6.0-second
macOS figure; it was not measured, because that figure was taken with the cache off.

### Who reads the snapshot, and what each read saves

| Reader | Linux saving per read | macOS saving per read |
| --- | --- | --- |
| A later `fdu .` or any metadata one-shot under `auto` | none: it never loads the snapshot (H108) | none: same rule |
| `--cache only` (answer labelled stale) | 1.26 s → 1.04 s, **0.22 s** (screen) | 521 ms → 146 ms at 175k entries, **3.6×** |
| `open`, watch startup: load and revalidate | cold 1.22 s → warm 1.00 s, **0.22 s** (probe) | not re-measured since H75 |
| `--analyze` repeated | none from the metadata snapshot: see below | none from the metadata snapshot, by the same mechanism |

Content analysis is where caching pays most, and that value lives in the content
sidecar, not the metadata snapshot.
A repeated `--analyze=lines` took 8.8 s against 26.7 s for the first run.
With the metadata snapshot deleted before each run, it took 8.9 s, reported `cold scan`,
and still restored all 875,000 records from the sidecar with 0 bytes of content read.
The sidecar’s per-file fingerprint (size, mtime, ctime, inode, device) is checked
against a fresh walk as easily as against a loaded index.

On Linux, the loaded snapshot also barely beats a walk even with no verification at all:
`--cache only` deserializes 79 MB on one thread in about 0.9 s, while the cold walk
spreads 4.5 CPU-seconds across four cores in 1.26 s. On macOS the same load is 3.6 times
faster than a walk, because an APFS walk costs about 3.5 times an ext4 walk per entry
while deserialization costs the same.

### What each answer needs, from first principles

A tally is a commutative reduction: files, directories, apparent and allocated bytes,
and newest mtime each fold in constant space, in any order, across threads.
The ignored share needs one more input per entry, whether a `.gitignore` rule matches
it, and evaluating that needs the matcher stack along the entry’s path, which the walker
already builds when it reads each directory’s `.gitignore`. Nothing in a summary needs
any entry to survive past the moment it is counted.

A depth-*d* tree needs a roll-up per directory up to depth *d* and the *n* largest
children of each. Each roll-up is still a commutative fold, of everything below the
directory, so a walk can stream it upward and retain only the directories it will print
plus the open frames on its way down.
pdu does exactly this at 3.7 MiB.

A retained index, 319 MiB on this tree, is what a *second* question needs: a watch, a
Python session, an opened root, or a filter changed without rescanning.
It is also what a snapshot serializes.

Today every view except the `--no-gitignore` summary builds the full index, for one
reason each: the tree renderer reads a retained index, and the summary reducer keeps no
control table, so a request that observes `.gitignore` “falls closed” to the index
([the planner](../../../crates/fdu-core/src/execution.rs), `fdu-elnn`). Measured, that
fallback costs 0.32 s (1.25 vs 0.93 s, screen) and 309 MiB before any snapshot, on a
tree whose `.gitignore` count is zero.

### Why the ranking differs between macOS and Linux

The harness records user and kernel CPU for every sample, and the split explains the two
rankings directly.

| Platform, tool | Wall | Kernel CPU | User CPU | Kernel CPU per entry | Cores kept busy |
| --- | ---: | ---: | ---: | ---: | ---: |
| macOS, fdu | 5.99 s | 35.4 s | 1.29 s | 35 µs | 6.1 |
| macOS, dumac | 6.33 s | 23.9 s | 0.78 s | 24 µs | 3.9 |
| macOS, pdu | 9.25 s | 39.2 s | 1.74 s | 39 µs | 4.4 |
| macOS, diskus | 8.65 s | 60.5 s | 2.37 s | 60 µs | 7.3 |
| Linux, fdu | 1.38 s | 3.22 s | 1.28 s | 3.2 µs | 3.3 |
| Linux, pdu | 1.12 s | 3.68 s | 0.74 s | 3.7 µs | 3.9 |
| Linux, diskus | 1.12 s | 3.63 s | 0.76 s | 3.6 µs | 3.9 |

On macOS the kernel is 95–97% of every parallel tool’s CPU. fdu and dumac read metadata
with `getattrlistbulk`, one call per directory batch; pdu, diskus, and dust call the
standard library’s per-entry `lstat` (confirmed in their sources), and on APFS that
per-entry path costs more and scales worse across threads — diskus spends 60 µs of
kernel time per entry to fdu’s 35. fdu’s user-space work, index included, is 3.5% of its
CPU, so the index is nearly free there.
dumac spends less kernel time per entry than fdu but keeps only 3.9 cores busy to fdu’s
6.1; fdu’s adaptive worker pool is what puts it 8% ahead.

On Linux there is no bulk-metadata call, and every tool issues the same `getdents64` and
per-entry `statx`, with identical counts under `strace -c`. fdu’s kernel time per entry
is the lowest of the three; it loses on user CPU (1.28 s to 0.74 s) and on cores kept
busy (3.3 to 3.9). That is not the cache, which was off in these runs.
Both come from building the index:

- glibc’s allocator serializes the index builder and the walkers on each other’s arena
  locks; the unchanged binary under `LD_PRELOAD` mimalloc, jemalloc, or tcmalloc ran the
  indexed tree in 1.11 s, level with pdu, and forcing one glibc arena made it 3.3 s
  (H159, `fdu-578e`);
- freeing the million-entry index before exit cost 95 ms on the main thread, now moved
  off the answer’s path (H156, exp-160, in this pass).

Core count is not the cause: restricted to two cores the indexed tree trailed pdu by
15%, and on four by 21% (**screen**).

The same user-space costs exist on macOS but hide under a kernel ten times more
expensive per entry.
Linux is the platform that measures fdu’s own efficiency.

## Key Insights

- **The default invests on every run in a snapshot that only opt-in paths read.** A
  later `fdu .` never loads it (H108, deliberately), and the one ordinary repeated
  workload that benefits from caching, content analysis, draws its benefit from the
  sidecar, which works without it.
- **The snapshot’s value is platform-shaped.** On macOS a `--cache only` read is 3.6
  times a walk, so the file is worth keeping for anyone who uses that path; on Linux it
  saves 0.22 s at a million entries and returns a stale answer.
- **Retained state is right for sessions and wrong for one-shot answers.** An index is
  what watch, `open`, and Python reuse; a one-shot tree or summary needs streamed
  roll-ups of a few thousand directories at most.
- **Disabling the cache does not make fdu the fastest on Linux.** The index costs 0.3 s
  and 309 MiB there whether or not it is saved, and the summary builds it only because
  its reducer cannot yet classify ignored entries.
- **Headline numbers must name the invocation.** Both published tables used
  `--cache off`, which no one types.

## When the Cache Makes Sense

Persisting pays when the expected saving of later reads exceeds the write cost now:

*write cost per entry* < *probability of a later read* × *(walk cost − load cost) per
entry*

| Quantity | Linux ext4, 4 vCPU | macOS APFS, M1 Pro |
| --- | --- | --- |
| Write cost per entry | 0.23–0.25 µs warm, 0.67 µs first | 0.29–0.51 µs |
| Metadata walk, wall per entry | 1.26 µs | 2.97 µs (175k tree) |
| Snapshot load, wall per entry | 0.9–1.0 µs | 0.83 µs |
| Saving per `--cache only` read | about 0.25 µs | about 2.1 µs |
| Saving per one-shot `fdu .` | 0 | 0 |

So:

| Situation | Persist? |
| --- | --- |
| One-shot metadata report, nobody asked for a cache | No, on either platform: no default reader exists |
| Watch or `open` session | Yes, periodically and at exit: the session is itself the reader |
| Content analysis | Yes, the sidecar; the metadata snapshot is not required |
| The user asks for a fast later `--cache only` | Yes, and it should be explicit: macOS repays it on the first read, Linux barely |
| Cold filesystem caches (for example after reboot) | Yes for `only`: a snapshot read beat a cold ext4 scan 118 ms to 277 ms at 84k entries |

Retaining an index in memory follows the same rule with the walk replaced by the next
question: keep it when a second question is coming (watch, `open`, Python), and stream
the answer when there is only one.

## Options Considered

### Option A: Stop writing the metadata snapshot on one-shot metadata runs

**Description:** Under `auto`, a one-shot report without analysis writes nothing; watch,
`open`, analysis runs (sidecar), and an explicit `--cache refresh` still write.
`--cache only` without a snapshot keeps failing loudly, naming the command that creates
one.

**Pros:**
- Removes 0.26 s from `fdu .` and 0.21 s from the default summary on Linux at a million
  entries, and an estimated 0.3–0.5 s from the macOS default; stops writing 79 MB per
  run.
- No default reader loses anything; analysis keeps its reuse through the sidecar.
- Agrees with the planner’s own rule that a one-shot metadata report gains nothing from
  the snapshot (H108).

**Cons:**
- `--cache only` and Python `open` after a plain `fdu .` find no snapshot until
  something writes one.
  On macOS that forfeits a real 3.6× fast path for users who relied on it implicitly;
  the skill and usage guide document the path but not an implicit seed.
- A behavior change to a documented cache contract
  ([cache design](../guides/cache-design.md#the-policy-axis)).

### Option B: Keep writing, but make the write cheap

**Description:** Hardware CRC-32C (SSE4.2 and ARMv8 both have it), no `fsync` for a file
whose corrupt state already reads as absent (`fdu-n75m` part 3), and a cheaper
unchanged-image check than re-encoding and re-reading 79 MB.

**Pros:**
- No contract change; every writer benefits, including watch and analysis.
- Checksum 58 ms → under 10 ms and `fsync` 55–85 ms → 0 are straightforward.

**Cons:**
- Encoding and the page-cache write remain, 120–450 ms at a million entries.
- Still pays on every run for a file no default run reads.

### Option C: Answer one-shot views without a retained index

**Description:** Two planner tiers, each falling closed to the index for anything it
cannot prove:

1. **Ignore-aware summary:** the streaming reducer classifies each entry against the
   walker’s matcher stack and folds an ignored share; default `--view summary` then
   costs what `--no-gitignore` costs today.
2. **Transient tree** (H66, `fdu-sk7v`): a one-shot tree folds files into directory
   roll-ups and retains directory topology, or only the printed depth, without file
   records.

**Pros:**
- Targets the usual commands directly: default summary 1.45 s → about 0.93 s, and the
  default tree toward pdu’s 1.09 s, with memory from 319 MiB to tens of MiB.
- No snapshot is written because there is no index to write, so Option A follows for
  these views by construction.

**Cons:**
- Engineering, not a flag: exact parity with the indexed answer must be proven by the
  golden and parity corpora, including hard links, ignored shares, and file counts.
- Filters, multiple views, and `--limit` interactions widen what the transient tree must
  prove before it can be selected.

### Option D: Reduce the index’s user-space cost on Linux

**Description:** Remove the glibc cross-thread frees and arena contention in the
detached builder (H159, `fdu-578e`), and rerun the rejected direct file fold with the
product job pre-registered (`fdu-o6um`).

**Pros:**
- Helps every retained path (watch, `open`, Python, filtered views) that Option C cannot
  cover. The `LD_PRELOAD` screen bounds the prize at about 0.28 s on this tree.

**Cons:**
- Structural work in the detached builder; an allocator dependency is the fallback and
  was declined before (H74, H85).

### Eliminated Options

- **Load the snapshot on `fdu .`:** a metadata answer must stat every entry anyway, so
  loading is additive (H108, H9).
- **Write the snapshot on a detached thread and exit without joining:** the write would
  be killed at exit, leaving a staging file behind on every run.
- **A size threshold for writing (`SNAPSHOT_MIN_ENTRIES`):** already measured and
  rejected, because on APFS a snapshot repays itself at any size (`fdu-hvs5`); the
  question here is who reads it, not how large it is.

## Recommendations

1. **Adopt Option C.1 first.** It is the smallest engine change with the largest effect
   on a usual command: the ignored share is a per-entry predicate, and the walker
   already holds the matcher.
2. **Adopt Option A on both platforms**, with one addition so macOS users keep their
   fast path deliberately: document `--cache refresh` (or a new explicit write policy)
   as the way to seed `--cache only`, and have a failed `--cache only` say so.
   Uniform behavior avoids a `cfg` branch the rules would call a guess, and the
   information that decides it — whether anyone will read the file — is the same on
   both.
3. **Adopt Option B regardless** for the writers that remain (watch, sessions, explicit
   refresh), starting with hardware CRC-32C and the `fsync` decision.
4. **Pursue Option C.2 (H66) and Option D in parallel**; C.2 fixes the default tree for
   one-shot runs, D fixes every path that must keep an index.
5. **Measure and publish the default invocations**, not `--cache off`: the harness
   already has `fdu-default-tree` and `fdu-index-summary` contracts, and the macOS table
   needs the same rerun.

Expected Linux outcome at a million entries, before D: `fdu .` 1.51 s → about 1.25 s (A)
→ toward 1.09 s (C.2); default summary 1.45 s → about 0.93 s (C.1). Each needs its own
measurement under the accept rule.

## Next Steps

- [ ] Decide Option A, including whether it applies to Python `open` defaults
  (`fdu-0t1v`)
- [ ] Measure the ignore-aware summary reducer (C.1, `fdu-1ovb`) under the accept rule
- [ ] Re-prioritize H66 (`fdu-sk7v`) and H159 (`fdu-578e`)
- [ ] Measure default-invocation contracts on Linux and macOS through the harness
- [ ] Decide the snapshot durability policy (`fdu-n75m` part 3) and hardware CRC-32C

## Methodology

Linux measurements used the release build of this branch with H156 applied, on the
generated `balanced` tree at seed `fdu-balanced-v1`, warm-steady, quiet host.
Harness rows are from the
[Linux comparison](../reports/report-2026-09-27-fdu-linux-tool-comparison.md) and its
committed result files.
Screens are 5–10-run `hyperfine` means with one warm-up, with the fdu cache directory
isolated per series.
Probe components are `perf_probe` internal timings, three runs each.
The snapshot write breakdown combines an `strace -T` timeline of the writer thread, the
isolated probe save, and a separate write-and-`fsync` test of the same 79 MB size.
The CPU split is read from the per-sample `wait4` accounting in the two committed
tool-comparison results.

Not established here: macOS default-invocation timings at a million entries, the macOS
allocator’s behavior under the same cross-thread pattern, and any cold-cache figure on
Linux hardware.

## References

- [Linux tool comparison, 2026-09-27](../reports/report-2026-09-27-fdu-linux-tool-comparison.md)
- [macOS tool comparison, 2026-09-26](../reports/report-2026-09-26-fdu-live-tool-comparison.md)
- [The fdu cache: two layers, and what verification costs](../guides/cache-design.md)
- [Platform tuning: snapshot participation](../guides/platform-tuning.md#snapshot-participation-is-a-cost-decision-and-apfs-reverses-its-conclusion)
- [Cache layers and defaults plan](../specs/done/plan-2026-08-15-fdu-cache-layers-and-defaults.md)
- [First Linux measurements](research-2026-08-13-linux-first-measurements.md)
- [Hypothesis registry](../guides/performance-loop.md#hypotheses): H9, H66, H74, H85,
  H108, H156–H159
- [Experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md):
  exp-066, exp-067, exp-135, exp-147, exp-160

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
