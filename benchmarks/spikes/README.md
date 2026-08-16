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
