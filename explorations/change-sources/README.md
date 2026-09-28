# Change-Source Review Evidence

This directory holds the instruments and sanitized results of the 2026-09-27 review of
the sources fdu could use to find what changed on disk between two moments: FSEvents
history replay, open-writer notification, filesystem catalogs, and full walks.
The findings and their interpretation are in
[the disk-growth change-sources research](../../docs/project/research/research-2026-09-27-disk-growth-change-sources.md);
this directory keeps what a reader needs to re-run or check them.

The FSEvents spike under review is [fsevents-replay](../fsevents-replay/README.md).
Each `MANIFEST.txt` (one per subdirectory, plus one here for `timing-lock`) lists every
file with its source in the review’s scratch directory, any change made for publication,
and its SHA-256.

## Contents

Commands run from the workstream’s copy under `$REVIEW` (see [Running](#running)).

| Directory | What it measures | Main instrument |
| --- | --- | --- |
| [pilot](pilot/) | Time to `HistoryDone` against cursor age for a quiet filter on the external volume | `timing-lock ./agesweep "$REVIEW/quiet" 3600 16 120` (root, cursor age in seconds, create flags, timeout in seconds) |
| [replay-cost](replay-cost/) | What replay cost scales with: log bytes behind the cursor, matching records, flush modes, cursor lookup | `timing-lock bin/replaycost --root "$REVIEW/quiet" --age 3600 --flags 16`; batches through `src/drive.py`, reduced by `src/analyze.py` |
| [harness-audit](harness-audit/) | Whether the spike’s live-root misses are open writers, and when a written descriptor notifies | `python3 close_order.py prepare "$REVIEW/close-order/leaves"`, then after 10 minutes `python3 close_order.py run "$REVIEW/close-order/leaves" "$REVIEW/bin/probe"` |
| [writer-coverage](writer-coverage/) | Which write paths emit an event while a file stays open; writer discovery through libproc | `python3 src/ops.py --root "$REVIEW/writer-coverage/fixtures/live-matrix" --livewatch bin/livewatch --probe "$REVIEW/bin/probe" --out results/live-ops.jsonl` (the root must be under `/Volumes`); `bin/writers --json` |
| [catalog](catalog/) | Daemonless catalogs: `searchfs` by ctime, APFS dir-stats generation counts, clone attributes | `src/bench_searchfs.sh external`; for dir-stats, `apfs.util -M` on an empty `$REVIEW/catalog/fixtures/ds`, `python3 src/dirstats.py populate` on it, then `python3 src/dirstats.py matrix "$REVIEW/catalog/fixtures/ds" data/dirstats-matrix.json` |
| [architecture](architecture/) | Walk, snapshot-load and snapshot-write cost per scope with the installed fdu, which predates `--cache auto\|on\|off` (its `--cache only` rows are today’s `--stale-ok`; its `refresh` rows, a cold scan that rewrote the snapshot, have no exact replacement); path identity; clone and link accounting | `./run-all.sh`, then `python3 summarize.py raw` |
| [os-facilities](os-facilities/) | Operating-system facilities that track changes as they happen (survey), unprivileged per-process I/O counters, Spotlight latency | [survey](os-facilities/survey.md); `src/rusage_sampler` samples `proc_pid_rusage` for same-user processes twice, 60 s apart |
| [dirstats-verify](dirstats-verify/) | Adversarial verification of APFS directory statistics: semantics on both volumes, privilege, persistence and `fsck` in a disk image, unset, accounting, write cost, and a gencount-pruned refresh against a walk oracle at 225k entries | `cc -O2 -o bin/ds src/ds.c`; `python3 src/semantics.py`; `src/run_scale.sh` (needs `timing-lock` on `PATH`) |
| [watch-soak](watch-soak/) | A one-hour read-only soak of `fdu --watch` on agent state B with a libproc writer list every 30 s: resident cost, event rates, persistence, and the watcher’s final view against two walks | `src/soak_start.sh <root> 10s`, then after the soak `src/soak_end.sh`; `python3 src/compare.py` and `src/counterfactual.py` |

`timing-lock` in this directory runs a command under an exclusive lock on
`$REVIEW/timing.lock`, so timed runs from concurrent agents do not overlap.

## Regime

All results come from one host on 2026-09-27: macOS 26.5.2, arm64, APFS internal +
external USB SSD, loaded host (Apple M1 Pro, 10 cores, 32 GiB; load average 6–28 from
concurrent review agents).
The external volume is mounted `noowners`, which makes its `.fseventsd` log readable
without root; the internal Data volume’s log is root-only.
The architecture timings used the installed `fdu 0.1.0-dev+g7a499493e`. Treat absolute
times as exploratory and compare within a run.

## Running

Requires macOS with the Xcode Command Line Tools and Python 3.12 or newer.
Scripts read these environment variables instead of host paths:

| Variable | Meaning |
| --- | --- |
| `REVIEW` | A new working directory on the volume under test, outside every observed root |
| `EXTERNAL_VOLUME` | Mount point of the external volume (catalog `searchfs` series) |
| `EXTERNAL_DEV` | That volume’s `st_dev`, from `stat -f %d "$EXTERNAL_VOLUME"` (replay-cost reduction) |
| `PROJECT_ROOT` | A large project checkout used as a scope (architecture) |
| `FDU` | The fdu binary to measure (architecture; default `fdu` on `PATH`) |
| `CHURN_SCRATCH_TOP` | Top-level directory name counted as scratch in `replay-cost/src/churn.py` |

Copy the workstreams into `$REVIEW` and build there; the scripts expect that layout
(`catalog` looks for its tools in `$REVIEW/catalog/attic/`).

```shell
export REVIEW=/path/on/the/test/volume/change-sources-run
export TMPDIR="$REVIEW/tmp" PATH="$REVIEW:$PATH"
mkdir -p "$TMPDIR" "$REVIEW"/{bin,quiet,catalog/attic,replay-cost/bin,writer-coverage/bin}
cp -R explorations/change-sources/* "$REVIEW/"
cc() { xcrun clang -O2 -Wall -framework CoreServices "$@"; }
cc explorations/fsevents-replay/probe.c -o "$REVIEW/bin/probe"
cc "$REVIEW/pilot/agesweep.c" -o "$REVIEW/pilot/agesweep"
for t in replaycost cursorcost pidcpu; do cc "$REVIEW/replay-cost/src/$t.c" -o "$REVIEW/replay-cost/bin/$t"; done
for t in livewatch writers; do cc "$REVIEW/writer-coverage/src/$t.c" -o "$REVIEW/writer-coverage/bin/$t"; done
for t in volcaps searchfs_ct extattr_walk dirstat_fsctl; do cc "$REVIEW/catalog/src/$t.c" -o "$REVIEW/catalog/attic/$t"; done
for t in pathprobe privsize; do cc "$REVIEW/architecture/$t.c" -o "$REVIEW/bin/$t"; done
xcrun swiftc "$REVIEW/architecture/volcap.swift" -o "$REVIEW/bin/volcap"
```

`harness-audit/analyze_real_root.py` re-reads a `real_tree.py` state directory from the
spike, which holds private inventories and is not published; its sanitized output is in
`harness-audit/results/`. `replay-cost/src/analyze.py` and `breakeven.py` need
`data/logindex.csv`, regenerated by
`src/logindex.py "$EXTERNAL_VOLUME/.fseventsd" data/logindex.csv data/logindex-summary.json`.

## Constraints for Re-Running

- **Real trees are read-only.** Use metadata calls only: never read contents, create
  cookie files, or mutate anything in a home, agent-state, or project tree.
- **Mutate only fixtures you created under `$REVIEW`.** `apfs.util -M` sets a
  persistent, inherited maintain-dir-stats flag; use it only on owned fixture
  directories and delete them to remove it.
- **Keep all state outside the observed root.** Binaries, fixtures, caches, outputs,
  `TMPDIR`, and the lock file belong under `$REVIEW`, never inside a root being measured
  or in this repository.
- **Change nothing system-wide.** No `sudo`, no journal purges, no SIP or Spotlight
  changes.
- **Hold `timing-lock` for timed runs** and keep each hold short.

## Placeholders in the Results

Published results replace host-specific values:

| Placeholder | Stands for |
| --- | --- |
| `$REVIEW`, `$SCRATCH` | The review’s working directory and the scratch directory above it on the external volume |
| `<external-volume>`, `<external-volume-name>` | The external volume’s mount point and name |
| `/Users/<u>`, `<project>`, `<fdu-checkout>` | The home directory, the project scope, and this repository’s checkout |
| `<dev-internal>`, `<dev-external>` | Device numbers of the internal Data and external volumes |
| `<data-volume-uuid>`, `<external-volume-uuid>` | Volume UUIDs |
| `<uid>` | The measuring user’s uid |

`harness-audit/results/codex-shallow-analysis-v2.sanitized.txt` also re-keys path
labels, inodes, and process ids to tokens, as its header describes.

## Upstream Sources Not Copied

- **XNU for the call-site map** (writer-coverage): `apple-oss-distributions/xnu` tag
  `xnu-12377.1.9`, commit `f6217f891ac0bb64f3d375211650a4c1ff8ca1ea`; files
  `bsd/kern/sys_generic.c`, `bsd/kern/kern_descrip.c`, `bsd/kern/ubc_subr.c`,
  `bsd/vfs/vfs_vnops.c`, `bsd/vfs/vfs_syscalls.c`, `bsd/vfs/vfs_subr.c`,
  `bsd/vfs/kpi_vfs.c`, `bsd/vfs/vfs_attrlist.c`, `bsd/vfs/vfs_fsevents.c`,
  `bsd/nfs/nfs_serv.c`, and `osfmk/vm/bsd_vm.c`.
- **XNU for the catalog review**: the same repository’s `main` on 2026-09-27 (newest tag
  `xnu-12377.121.6`); `bsd/vfs/vfs_attrlist.c`, `bsd/vfs/vfs_syscalls.c`,
  `bsd/vfs/vfs_vnops.c`, `bsd/sys/vnode.h`, `bsd/sys/fsctl.h`, and `bsd/sys/attr.h`.
- **Libc dir-stats**: `apple-oss-distributions/Libc` `main`, `libdarwin/h/dirstat.h`,
  `libdarwin/dirstat.c`, and `tests/dirstat.c`.
- **Apple File System Reference**, 2020-06-22 edition, pages 80–93 (directory statistics
  records and inode flags).
- **Manual pages** `searchfs(2)`, `getattrlist(2)`, and `getattrlistbulk(2)` from the
  installed SDK.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
