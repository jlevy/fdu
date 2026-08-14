# Platform and Environment Tuning

Which measured constants are portable, which are not, and how to tell.

Every tuning constant in fdu was chosen by measurement.
Until 2026-08-13 every one of those measurements came from the same host — a 10-core
Apple M1 Pro on APFS — because that is where the optimization loop ran.
That is not a criticism of the numbers; it is a statement about their scope.
A constant measured in one regime is evidence about that regime, and carrying it into
another without a second measurement is exactly the “believing an improvement that is
not there” failure [the performance loop](performance-loop.md) exists to prevent.

This guide names the regimes, records which constants have evidence in which, and states
the rule for adding a platform-specific value.

## The three axes

A measurement’s regime is a point in three dimensions, and all three belong in any
recorded result.

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

| State | Preparation | What it answers |
| --- | --- | --- |
| `warm-steady` | Explicit full-tree warmups per tool | Rerunning a CLI over a working tree, the dominant interactive case |
| `controlled-cold` | `sync`, then `3` to `/proc/sys/vm/drop_caches`, per sample | First access after boot or eviction, and trees larger than RAM |

macOS has no equivalent privileged control: `/usr/sbin/purge` only promises to
*approximate* boot conditions, so a purge-cold macOS run is a labelled diagnostic rather
than controlled-cold evidence.
Linux is where the cold regime can be measured honestly, and on bare metal is where its
device-latency conclusions become portable.

## Constants and where their evidence comes from

Every value below is in `crates/fdu/src/scan.rs` unless noted, and every one carries a
doc comment citing the measurement that chose it.
The column that matters is the last one.

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
| Global allocator | system | Never chosen by measurement | glibc `malloc` is the worst case for fdu’s cross-thread free pattern; a local mimalloc build measured −30.3% on the summary plan (H74, unconfirmed) |

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

| Platform | Open question |
| --- | --- |
| Linux | Worker-count sweep in both cache states (H76); allocator replacement (H74); the warm-open inversion, which is a defect rather than a tuning question (H75) |
| Linux, bare metal | Inode-ordered statting (H73) and any queue-depth claim; these cannot be settled on a VM |
| macOS | Whether the reconcile wave and batch sizes still hold after the content tier landed |
| Windows | Everything; no speed measurement exists |

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
