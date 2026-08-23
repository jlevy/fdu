# Spike Harnesses

Small, self-contained instruments for one-off measurement spikes.
Nothing here runs in `make check` or the benchmark suite, and nothing here produces
ledger evidence by itself: results from these tools order hypotheses, and anything that
motivates a production change re-runs under
[the performance loop](../../docs/project/guides/performance-loop.md) on real hardware.

## walkspike.c

Isolates the Linux directory-enumeration and metadata layer.
One single-threaded BFS walk with interchangeable strategies, all producing identical
file/directory/byte tallies (a built-in oracle: any variant that disagrees is broken):

| Variant | Mechanism |
| --- | --- |
| `readdir` | glibc `opendir`/`readdir` + `fstatat` — the shape of `std::fs::read_dir` plus `DirEntry::metadata` |
| `getdents` | raw `getdents64`, 256 KiB buffer, `fstatat` per entry |
| `statx` | raw `getdents64` + `statx(dirfd, name, STATX_BASIC_STATS)` |
| `narrow` | `statx` with only TYPE/SIZE/BLOCKS/MTIME/CTIME/INO requested |
| `filesonly` | `statx` only for `DT_REG` (+`DT_UNKNOWN` fallback); directories descended by `d_type` — the summary-tier shape |
| `inosort` | `statx` in ascending `d_ino` order per directory — the cold inode-locality claim |
| `uring` | hand-rolled io_uring `IORING_OP_STATX`, queue depth 128, one ring, no liburing dependency |

```shell
gcc -O2 -o walkspike walkspike.c
./walkspike statx /path/to/tree
```

Wide directories that overflow the fixed name arena fall back to `readdir` for that
directory (reported tallies stay exact); the arena bound is generous for real trees.

The `elide` variant is the `fdu-jnuo` probe: it skips the terminating empty `getdents64`
whenever the previous call left at least one maximum-size dirent of slack, which on
in-tree filesystems can only mean the iterator hit EOF. Measured 2026-08-15 on the 450k
generated tree it removes exactly one call per directory (57,260 to 28,630) with
identical tallies and a warm wall change inside noise, so the elision is real but only
worth composing into a cold or dir-heavy campaign.
Production use would additionally need a `statfs` `f_type` allowlist, because FUSE and
network filesystems may legally return a short buffer mid-stream.

## parfloor.c

The parallel companion to `walkspike.c`, and the denominator for any question of the
form “how close is fdu to the machine”.
`walkspike` is single-threaded, which is right for ranking syscall strategies and wrong
as a lower bound for a parallel walker: fdu’s aggregate tier runs four workers, so a
one-thread floor sits above it rather than below.
`parfloor` runs N workers over a shared directory queue, doing raw `getdents64` plus one
`statx` per entry into four integer accumulators, and nothing else — no index, no
retained paths, no per-entry allocation, no delta contract.

| Variant | Mechanism |
| --- | --- |
| `enum` | `openat` + `getdents64` + `close`, classifying by `d_type`; no metadata call. The floor for a search tool’s job, not a disk-usage tool’s |
| `stat` | `enum` plus `statx(dirfd, name)` per entry. The floor for du’s job, and fdu’s |
| `abspath` | `stat`, but each entry statted by full absolute path instead of dirfd-relative |

The `stat`/`abspath` pair exists because it is the one difference between fdu’s
per-entry metadata call and every ecosystem walker’s. `walkdir` and `ignore` both call
`fs::symlink_metadata(&self.path)`, resolving every path component from the root per
entry; `std::fs::DirEntry::metadata`, which fdu uses, resolves one component against the
directory descriptor.
Isolating it costs nothing else, so the gap prices that choice alone.

```shell
gcc -O2 -pthread -o parfloor parfloor.c
./parfloor stat /path/to/tree 4
```

Tallies match `walkspike`, `arena_spike`, `peerwalk` and fdu’s summary, so any variant
or thread count that disagrees is broken rather than fast.

## peerwalk.rs

The ecosystem anchor.
fdu writes its own walker, and whether that is worth doing can only be answered against
the walkers a Rust program would otherwise reach for — chiefly
[`ignore`](https://docs.rs/ignore), which is ripgrep’s walker: it is published from the
ripgrep repository and ripgrep depends on it, so it is the closest thing to a known-good
reference in this ecosystem.

The comparison only means something if the *job* is held fixed, and the natural jobs
differ: a search tool needs to know which entries are files, which `d_type` answers for
free, while a disk-usage tool needs their sizes, which costs a metadata call per entry.
Both are measured, separately, and only the second is comparable with fdu.

| Variant | Job |
| --- | --- |
| `ignore-nostat` | enumerate and classify by `d_type` — ripgrep’s job |
| `ignore-stat` | enumerate and read metadata per entry — du’s job, and fdu’s |
| `ignore-default` | ripgrep’s default filters on: gitignore, hidden, parent ignores |
| `walkdir` | the single-threaded walker `ignore` is built on |
| `jwalk` | the parallel walkdir-alike |

It is not a workspace member and `make` never builds it, because it takes third-party
dependencies the shipped crate deliberately does not have.
Build it in a throwaway directory:

```shell
mkdir -p /tmp/peerwalk/src && cp peerwalk.rs /tmp/peerwalk/src/main.rs
cat > /tmp/peerwalk/Cargo.toml <<'EOF'
[workspace]
[package]
name = "peerwalk"
version = "0.1.0"
edition = "2021"
[dependencies]
ignore = "0.4"
walkdir = "2"
jwalk = "0.8"
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
EOF
cargo build --release --manifest-path /tmp/peerwalk/Cargo.toml
/tmp/peerwalk/target/release/peerwalk ignore-stat /path/to/tree 4
```

The numbers it produced are in
[the metadata-walk physics research](../../docs/project/research/research-2026-08-23-metadata-walk-physics.md).

## arena_spike.rs

The consumer-side companion to `walkspike.c`: the same syscall load and worker count as
fdu’s portable scan, with the index consumer replaced by worker-local entry and name
arenas, per-directory tally slots, and one bottom-up roll-up pass by depth after the
walk (the S4/H60 shape).
It answers “what does the retained-index representation itself cost?”
the way walkspike answers it for enumeration, and it must reproduce fdu’s summary
tallies to count.

```shell
rustc -O -o arena_spike arena_spike.rs
./arena_spike /path/to/tree 4
```

It is a physics measurement, not a correctness prototype: no delta contract, no
progressive publication, no error provenance.
See
[the consumer structural-headroom research](../../docs/project/research/research-2026-08-15-consumer-structural-headroom.md)
for the numbers it produced.

## paired_runner.py

Adjacent-paired tool comparison in the project’s decision style: alternating A/B order
per pair, wall time around spawn+wait, rusage via `wait4`, paired median with a
bootstrap 95% interval.
`warm` mode performs explicit full-tree warmups first; `cold` mode runs `sync` and
writes `3` to `/proc/sys/vm/drop_caches` before every sample (root only).

```shell
SPIKE_TREE=/path/to/tree SPIKE_FDU=target/release/fdu \
  python3 paired_runner.py warm "fdu-summary:diskus,fdu-tree:dut" 10
```

Edit the `TOOLS` table for the binaries under test; entries reference competitor
binaries by absolute path so the exact artifact measured is unambiguous.

## gen_tree.py

Deterministic heterogeneous tree generator (node_modules-heavy projects, nested
packages, sparse pack files, assets, symlinks) used for the 2026-08-13 Linux scouting
measurements in
[the Linux first-measurements research](../../docs/project/research/research-2026-08-13-linux-first-measurements.md).

```shell
python3 gen_tree.py /tmp/fdu-spike/tree 450000
```

Sizes use sparse files, so apparent sizes are realistic while disk use stays small;
allocated-size distributions are therefore *not* realistic, which is fine for
metadata-path timing and wrong for anything comparing allocated-byte semantics.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
