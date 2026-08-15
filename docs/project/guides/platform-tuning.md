# Platform and Environment Tuning

Which measured constants are portable, which are not, and how to tell.

Every tuning constant in fdu was chosen by measurement.
Until 2026-08-13 every one of those measurements came from the same host — a 10-core
Apple M1 Pro on APFS — because that is where the optimization loop ran.
That is not a criticism of the numbers; it is a statement about their scope.
A constant measured in one regime is evidence about that regime, and carrying it into
another without a second measurement is exactly the “believing an improvement that is
not there” failure [the performance loop](performance-loop.md) exists to prevent.

This guide maps each shipped constant back to the run that chose it, and states the rule
for adding a platform-specific value.
It is the third of the three performance documents: [the loop](performance-loop.md) owns
the protocol, [the ledger](../reports/report-2026-08-10-fdu-performance-experiments.md)
owns the results and is regenerated from artifacts, and this page owns the mapping from
a value in the source to the evidence behind it.

## The three axes

A measurement’s regime is a point in three dimensions — platform, host, and cache state
— and all three belong in any recorded result.
[The loop defines them](performance-loop.md#what-we-measure) and the ledger’s **regime
coverage** table counts which combinations the evidence actually spans, generated from
the artifacts rather than asserted here.
What follows is only what each axis means *for choosing a constant*.

### Platform

The operating system and filesystem together, because the syscall surface and the
metadata layout move as a unit.

| Platform | Status | What is different about it |
| --- | --- | --- |
| macOS / APFS | Primary; all 51 ledger experiments | `getattrlistbulk` returns enumeration and complete stat-tier metadata per directory, so the per-entry metadata wait the portable path pays is largely hidden |
| Linux / ext4 | Measured, not yet in the ledger | No bulk-metadata analog is profitable; the standard library already issues `getdents64` + dirfd-relative `statx`, so per-entry kernel time is the floor |
| Windows / NTFS | CI-tested for correctness; unmeasured for speed | — |

### Host

Whether the machine is real or virtual, which decides what a *cold* measurement means
and nothing else.

| Host | Warm measurements | Cold measurements |
| --- | --- | --- |
| Bare metal | Valid | Valid; the only place storage-latency claims can be settled |
| Virtualized (KVM, containers, CI, cloud, WSL) | **Valid, and the common deployment case** | Order strategies only; guest-cold reads may still be served from host cache, so device latency is understated |

This asymmetry is the point.
Virtualization does not distort user-space cost, syscall cost, allocation behaviour, or
thread scheduling in any way that has been measured to matter, so a warm result from a
VM describes the environment most fdu runs actually happen in — a container, a CI job, a
cloud instance, a WSL session — and deserves to be treated as evidence about it rather
than discounted as second-class.
What virtualization does distort is the storage layer beneath the guest.
A hypervisor’s page cache sits under the guest’s, so dropping the guest’s caches does
not reach the disk. That makes exactly one class of claim untestable on a VM: anything
whose mechanism is device latency or I/O ordering.
Inode-ordered statting (H73) and queue-depth hypotheses (H76, io_uring) are that class,
and they need bare metal.
Everything else does not.

### Cache state

`warm-steady` and `controlled-cold`, with their preparation and their labelling rules,
are defined in [the loop](performance-loop.md#what-we-measure); that is the one place
they are specified. What matters here is only that a constant tuned in one state is not
evidence about the other — a worker count chosen where the walk is CPU-bound says
nothing about the depth a latency-bound walk wants — so a default claiming both states
needs a measurement in both.

## Constants and where their evidence comes from

Every value below is in `crates/fdu/src/scan.rs` unless noted, and every one carries a
doc comment citing the measurement that chose it.
The column that matters is the last one.

This table is written by hand because nothing else records the link it carries.
An experiment artifact knows the regime it ran in, which is why the ledger can count
regime coverage without anyone maintaining it; what no artifact records is *which
shipped constant a run settled*. Until an experiment can name the constants it fixed,
that mapping is asserted here and in the doc comments beside the values, and both have
to be updated by whoever changes one.
Prefer the doc comment: it is what the next person editing the value will read.

| Constant | Value | Measured on | Linux evidence |
| --- | ---: | --- | --- |
| `DEFAULT_SCAN_THREADS_CAP` | 6 | M1 Pro, 10 cores, 60k `node_modules` tree | **None.** The knee was found where four and six matched within noise and eight was 4% worse |
| `ADAPTIVE_SCAN_THREADS_CAP` | 16 | M1 Pro, 720k cache-pressure corpus (exp-015) | **None** |
| `ADAPTIVE_SCAN_PARALLELISM_MULTIPLIER` | 2 | M1 Pro | **None** |
| `ADAPTIVE_SCAN_CALIBRATION_ENTRIES` | 16,384 | M1 Pro | **None** |
| `ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY` | 30,000 | M1 Pro; APFS regimes of ~18/22/42 µs per entry | **None, and suspected inert** — see below |
| `DEFAULT_RECONCILE_THREADS_CAP` | 4 | M1 Pro (exp-030) | **None** |
| `RECONCILE_WAVE_DIRECTORIES` | 1,024 | M1 Pro; 4,096 refuted at 60k (exp-031) | **None** |
| `DEFAULT_BATCH_SIZE` | 1,024 | M1 Pro | **None** |
| `macos_bulk::BUFFER_BYTES` | 64 KiB | M1 Pro; 256 KiB refuted (exp-029/039) | Not applicable — macOS only |
| `content_analysis::READ_CHUNK_BYTES` | 64 KiB | M1 Pro, 307–2,001-entry trees | **None** |
| `DEFAULT_MAX_FILE_BYTES` | 16 MiB | Policy choice, not a measured knee | Not a tuning constant |
| Global allocator | system | Never chosen by measurement | Measured, not adopted. mimalloc wins **only the aggregate tier** (−23.0% [−28.4%, −16.7%]); the index tier and snapshot load both span zero. Costs +139% peak RSS on that tier and is unmeasured on macOS, where the system allocator differs. See H74/H85 |

### The adaptive threshold is the clearest suspected mismatch

`ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY` exists to recognize a latency-bound scan and
unlock deeper parallelism.
Its comment records how 30 µs was chosen: APFS measurements of “roughly 18 microseconds
on the 60k tree, 22 on the 120k boundary, and 42 or more on the 720k cache-pressure
tree”, with thirty placed in the gap.

Those are APFS numbers.
The Linux scouting measured a warm single-threaded floor of about **1.5 µs per entry**,
some twenty times below the threshold.
If warm Linux service time never approaches 30 µs, the trigger never fires, and an
automatic scan stays at its six-worker cap in every regime the threshold was meant to
distinguish. That is a concrete mechanism for the one place Linux measurement found fdu
behind: `diskus`, which runs three times the core count, led the cold scalar class by
22.8%.

This is a hypothesis (H76, `fdu-tk1b`), not a conclusion.
It is stated here because the constant’s own documentation makes the platform dependence
legible, and because a sweep is the cheap way to settle it: `perf_probe --threads N`
takes the worker count directly.

### The one-shot timing is sensitive, but no replacement qualified

The calibration accumulates the first 16,384 entries to complete, decides once, and is
then discarded. `scan::tests::completion_order` demonstrates the consequence
deterministically: one tree can produce opposite decisions under different valid
completion orders.

The Apple Silicon/APFS campaign corrected the interpretation of that fact.
Completion-order sensitivity is not automatically performance harm.
On a trace-verified fast-then-slow corpus, the shipped controller held at six workers
while repeated and staged alternatives expanded.
The alternatives were 58.49% and 60.73% slower; fixed eight, ten, and sixteen workers
also regressed. The profile showed more kernel work, lock wait, and scheduler pressure
after expansion, with no macOS bulk fallbacks.

No candidate survived discovery, so the one-shot procedure, the 30 µs threshold, and all
worker caps remain unchanged.
Future sweeps must still record the bounded policy history: otherwise a threshold result
describes an unknown mixture of decisions.
The characterization, experiments, and no-change decision are in
[the adaptive-worker gap-closure report](../reports/report-2026-08-15-adaptive-worker-gap-closure.md).

## How a divergence is expressed in code

Two kinds of platform difference live in this engine, and only one of them is a tuning
question.

A platform **API** — `getattrlistbulk` — does not exist elsewhere and cannot compile
elsewhere. Those stay `cfg`-gated at their call site as optional accelerators over a
portable path, falling back rather than failing, exactly as
[the design principles](../architecture/fdu-design-principles.md) require.

A platform **tuning** is the same portable code wanting a different value, or a
different one of two portable strategies.
Those live in `crates/fdu/src/platform_tuning.rs` as data, one table per platform, and
the module holds three properties that matter more than the values in it.

**Both tables compile in every build.** `cfg` selects the *default*, never the
*existence*. This is the load-bearing rule: an arm that only compiles where it is the
default cannot be type-checked or parity-tested anywhere else, so it rots, and then a
change made for one platform breaks the other invisibly.
`ScanOrder` is the existing precedent — breadth-first and depth-first both compile
everywhere and only the default is chosen.
A strategy therefore needs no new machinery; it is a `Tuned<T>` whose `T` is an enum.

**The guarantee is compile-time.** A `const` assertion block evaluates every table in
every build, so an edit that broke the macOS table would fail a Linux build.
Nobody has to own a Mac to keep the Mac arm honest, which is what makes it safe for a
Linux measurement to land without a macOS replication in the same change.

**An inherited default cannot pass for a measured one.** Each value carries `Measured`
or `Inherited`, and the const block asserts that the portable table still says
`Inherited`, with the message *promote this to `Tuned::measured` in the same change that
lands the sweep*. The build therefore forces whoever lands a Linux number to say which
experiment settled it, rather than letting a value quietly change standing.

### What CI guarantees, and what it cannot

Three layers, each catching something the others cannot:

| Layer | Runs where | Catches |
| --- | --- | --- |
| `const` assertion block | Every build, every target | A table that stops being well-formed, on a platform nobody present can build |
| `every_platform_table_produces_the_same_index` | Every platform’s test job | A tuning value that changes the *answer* rather than the speed. The swept settings are read out of the tables, so the test widens by itself when a platform diverges |
| Golden tryscripts | ubuntu, macos, windows runners | Output drift: any field that renders differently per platform fails that runner, which is why unstable fields carry named patterns rather than elisions |

The watch tests add a fourth, exercising inotify, FSEvents and ReadDirectoryChangesW on
their own runners, which is the only way per-platform event semantics get caught before
a user finds them.

What none of this catches is a **speed** regression on the platform you are not on, and
that is deliberate: a timing gate on a shared CI runner measures the runner.
The protection there is procedural rather than automated — a divergence is landed with
its regime recorded, and the platform that did not measure keeps `Evidence::Inherited`
until someone runs the loop there.

What this does **not** license is a `cfg` per disagreement.
A divergence costs two values to keep true and two regimes to re-measure whenever either
moves, so the bar is a measured reversal on a decision that matters, not a difference
within noise. Prefer one adaptive mechanism that measures the machine over two constants
that name it.

### Snapshot participation is a cost decision, and APFS reverses its conclusion

The cache is not a tuning constant today, and the measurement that was going to give it
one instead argued against it.
The reason belongs beside the others.

Measured on Linux/ext4 over 84,539 entries, warm operating-system cache, nine
interleaved paired trials: an unfiltered metadata summary answered transiently in 71 ms,
while the same request under a warm revalidating `auto` policy took 161 ms, and a
no-scan `only` read took 81 ms.
The mechanism is that revalidation stats every entry regardless of what the snapshot
holds, so for a metadata query the snapshot avoids no filesystem work; deserialisation
then costs about what a warm walk costs, roughly 0.96 against 0.84 microseconds per
entry.

The snapshot does pay where it avoids expensive work: content analysis went from 639 ms
to 325 ms warm, and with a cold operating-system cache a snapshot-only read beat a cold
scan 118 ms against 277 ms.

That gives the rule — persist when the retained state costs more to recompute than to
load and revalidate — but not the number.
The entry-count threshold above which an ordinary metadata run should persist a snapshot
is exactly the kind of constant this guide exists to flag: the only value the ext4 data
supports is roughly 250,000 entries, measured on a virtualised host.

Measuring it on Apple Silicon and APFS did not confirm that number.
It removed the premise underneath it.
Over 175,128 entries, warm, nine interleaved paired trials on an uncontrolled host, a
transient summary took 521 ms (2.97 microseconds per entry) while a no-scan `only` read
took 146 ms (0.83). Deserialisation costs about the same on both filesystems, but an
APFS metadata walk costs roughly three and a half times what an ext4 one does, so the
comparison that came out at +18% against the snapshot on ext4 comes out at more than
three times *for* it here.
The snapshot write measured 90 ms (0.51 per entry) against a 375 ms saving on each later
`only` read: it repays itself about four times over on the first reuse, at any tree
size.

So the proposed gate — persist only above a size, because metadata only pays back under
a cold cache — is an ext4 artifact.
On APFS a *warm* read already pays, and suppressing the write would give up that value
on every tree small enough to be gated.
`SNAPSHOT_MIN_ENTRIES` therefore stays `None`, which is a measured decision rather than
a deferred one; `fdu-hvs5` carries the evidence.
This is the clearest case in this guide of a constant that was reasonable in the regime
that produced it and wrong in the one it would have shipped to.
[The cache-layers plan](../specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md)
carries the full cost model.

## The rule for a platform-specific constant

1. **A shared default must have evidence in every regime it claims.** One measurement
   supports one regime.
   A constant with macOS evidence and no Linux evidence is a macOS constant that Linux
   currently inherits, and should say so in its doc comment.
2. **Prefer one adaptive mechanism over two hardcoded values.** The service-time
   calibration is the right shape: it measures the machine instead of asking which
   machine it is. When it needs different constants per platform, that is a signal the
   measured quantity is not the invariant one.
3. **`cfg(target_os)` on a tuning value needs both numbers measured.** A platform branch
   with a guess on one side is worse than a shared value, because it looks like
   evidence.
4. **Record the regime in the experiment artifact, all three axes.** `host_cpu`,
   `filesystem`, and `os_cache` already exist; virtualization belongs alongside them so
   a later reader can tell whether a cold number was ever able to mean what it says.

## What to measure next per platform

Each hypothesis id below is stated in full, with its predicted metric and current
status, in [the loop’s registry](performance-loop.md#hypotheses).

| Platform | Open question |
| --- | --- |
| Linux | Worker-count sweep in both cache states (H76, H84); allocator replacement (H74); the warm-open inversion, which is a defect rather than a tuning question (H75) |
| Linux, bare metal | Inode-ordered statting (H73) and any queue-depth claim; these cannot be settled on a VM |
| macOS | Whether the reconcile wave and batch sizes still hold after the content tier landed |
| Windows | Everything; no speed measurement exists |

The Linux evidence behind these is in two research notes, neither of them ledger
artifacts:
[the first Linux measurements](../research/research-2026-08-13-linux-first-measurements.md),
which established that the syscall layer has no headroom and the index consumer is the
gap, and
[the three-tier baseline](../research/research-2026-08-13-linux-three-tier-baseline.md),
which measured the aggregate, index, and content tiers together and found the warm-open
inversion scale-independent.

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
