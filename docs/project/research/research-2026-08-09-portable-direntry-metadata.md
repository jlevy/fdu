# Research: Unix `DirEntry` Metadata Evidence

**Date:** 2026-08-09

**Status:** Implemented and validated

## Decision

Use `std::fs::DirEntry::metadata()` in the Unix directory-enumeration loops instead of
constructing an absolute path and calling `std::fs::symlink_metadata()` for every entry.
Use a fresh
[`symlink_metadata()`](https://doc.rust-lang.org/std/fs/fn.symlink_metadata.html) query
on non-Unix platforms.

Both APIs preserve the required non-following behavior. The
[Rust documentation](https://doc.rust-lang.org/std/fs/struct.DirEntry.html#method.metadata)
states that `DirEntry::metadata()` is semantically equivalent to `symlink_metadata()`
on Unix but reuses enumeration data without a syscall on Windows. That Windows shortcut
is not a valid fingerprint source: Microsoft documents that
[directory-enumeration attributes may be non-current on NTFS](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstfilew).
The fresh query preserves cache correctness there.

On the measured macOS 100,000-entry corpus, the Unix change improved all three affected
component medians and preserved the exact index digest in every sample.

This is evidence for one localized implementation choice, not a product-performance
claim. The optimized Linux syscall walker, cross-tool comparison, dedicated-host
protocol, and full scale matrix remain open.

## Hypothesis

The portable walker previously performed these operations for every directory entry:

1. allocate an absolute `PathBuf` with `DirEntry::path()`;
2. resolve that path from the process root with `symlink_metadata()`;
3. allocate the same absolute path again only if an error had to be reported.

On Unix, `DirEntry::metadata()` can avoid the caller-owned path construction on the
success path while retaining the same non-following semantics. The error path still
constructs a full path so diagnostics retain the existing precision.

On Windows, Rust obtains `DirEntry` metadata from directory-enumeration data. The first
cross-platform CI run observed one entry's cached attributes change between the cold
scan and immediate revalidation in both the Rust and installed-wheel suites. Treating
those cached values as an authoritative fingerprint would create spurious updates and
could accept non-current attributes. Non-Unix builds therefore pay for a fresh path
query.

## Method

- Before source: commit `fda1039`
- Before probe SHA-256:
  `e589861bd7a81acd66ef22cb8031619c684499738f2997242311b27a53a70301`
- After probe SHA-256:
  `1b7955a10482caaa62bf1ea2aa1f7cf4c6eebc84412ad355457ed788803ffca7`
- Build: `cargo build --locked --release -p fdu --example perf_probe
  --no-default-features`
- Toolchain: Rust 1.97.1, target `aarch64-apple-darwin`
- Host class: Apple M1 Pro, 10 logical CPUs, Darwin 25.5.0, APFS
- Corpus: `balanced`, 100,000 required descendants
- Observed manifest SHA-256:
  `1fa57ff836818181ca456c56b9b36bc5be83b2548de9176ff71e61ef91202b16`
- Filesystem-cache state: uncontrolled; no cold or verified-warm label is implied
- Samples: seven alternating before/after pairs per job on one corpus
- Snapshots: one compatible snapshot per binary for revalidation
- Acceptance: every sample exited successfully and its exact engine digest matched the
  observed manifest

Builds used an isolated detached worktree and separate target directory. The corpus was
created once, neither binary mutated it, and the harness removed the worktree, corpus,
snapshots, and build directory after the paired set.

## Results

| Component | Before median | After median | Median change | Paired median change |
| --- | ---: | ---: | ---: | ---: |
| Scan producer | 606.019 ms | 551.787 ms | -8.95% | -8.24% |
| Scan plus index | 724.154 ms | 694.395 ms | -4.11% | -7.80% |
| Unchanged revalidation | 938.866 ms | 849.080 ms | -9.56% | -6.84% |

Raw component durations, in milliseconds:

| Component | Before | After |
| --- | --- | --- |
| Scan producer | 679.867, 659.359, 589.542, 606.019, 588.423, 701.234, 595.605 | 654.103, 551.787, 571.554, 541.778, 545.659, 607.727, 546.501 |
| Scan plus index | 735.071, 723.033, 717.457, 721.363, 1001.083, 732.449, 724.154 | 773.173, 656.801, 661.506, 694.395, 772.140, 816.145, 667.672 |
| Unchanged revalidation | 985.637, 865.920, 1173.358, 881.175, 1080.715, 861.479, 938.866 | 918.175, 849.080, 833.741, 857.292, 889.666, 815.649, 813.458 |

The paired medians are the more useful comparison because they retain local scheduling
and cache drift within each adjacent old/new pair. All three are favorable. A separate
10,000-entry sweep across batch sizes 64, 256, 1,024, 4,096, 16,384, and 65,536 showed
no stable batch-size effect, so the default remains 1,024.

## Correctness and Engineering Review

- Both metadata paths are non-following, so a retained symlink cannot turn the tree walk
  into traversal of its target.
- A focused Unix regression test points a retained symlink at an external directory and
  proves that the scan records the link itself without traversing the target.
- A cross-platform regression mutates a file after obtaining its directory entry and
  proves that the fingerprint helper observes the post-mutation size. This fails if the
  Windows implementation regresses to cached enumeration metadata.
- The existing scan, revalidation, snapshot, golden CLI, and cross-platform suites still
  own the complete semantic contract.
- All 42 paired performance samples matched the independent per-run engine digest.
- No dependency, public API, snapshot format, unsafe code, or alternate mutation path
  was introduced.

## Limitations

The run used one local macOS host and an uncontrolled operating-system cache. It
measured the probe's internal component boundary rather than CLI or Python latency and
did not invoke a comparator. The raw set was produced by the focused paired script
rather than the immutable release runner. These limits prevent a broader claim, but
they do not weaken the causal Unix comparison between two binaries that differ only in
the metadata call and operate on the same exact corpus. They provide no performance
evidence for Windows, where correctness requires the fresh query.

## Follow-Up

The portable improvement does not replace `fdu-atqk`. The Linux fast path still needs
the designed `getdents64`/dirfd-relative `statx` implementation and fallback, and
parallel traversal still requires its bounded cancellation and concurrency proof. The
next scale spike should use the committed runner and record build and host provenance
before evaluating those larger changes.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
