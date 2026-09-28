---
type: is
id: is-01m3h4knhxm6kwm8ykhpb7br6n
title: "Linux index tier: remove glibc cross-thread frees from the detached builder (H159)"
kind: task
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies:
  - type: blocks
    target: is-01m3kkrj5f6n9b38g6d1w6mrew
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T09:54:44.925Z
updated_at: 2026-09-28T13:11:00.607Z
---
The 2026-09-27 Linux comparison (docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md) found fdu's indexed tree 19% behind pdu and 18% behind diskus on a 1M-entry tree, while summary mode leads both. The unchanged binary under LD_PRELOAD mimalloc/jemalloc/tcmalloc closes the whole gap (1.39 -> 1.11-1.13 s; summary 0.98 -> 0.82 s); glibc.malloc.arena_max=1 makes it 3.3 s. A context-switch profile names the blocking sites: the consumer freeing walker-allocated Vec<DetachedChild> buffers and directory_ids PathBuf keys into walker-owned arenas, plus walker-side PathBuf/file_name allocations waiting on arenas that glibc's tcache has mixed across threads. Candidates, dependency-free first: return drained child buffers to the producing worker (per-worker pools, batched); replace the PathBuf-keyed directory map with walker-assigned directory tokens carried in the queue claim; move the claimed rel_dir into DetachedDirectory instead of copying. Measure each under the accept rule on cold-scan-index and the product CLI job. An allocator dependency stays out unless a structural change cannot reach it (H74, H85 history).

## Notes

## Pre-registration (2026-09-28, before any measurement)

Hypothesis: H159 (registry row exists). Tier: index (detached cold bootstrap). Platform
of the claim: Linux/glibc. Branch: claude/perf-h159-recycle, stacked on the stack-141 top
56c506e1 (engine a5c0ab46).

Change: the consumer no longer frees walker-allocated listing buffers. Each detached
walker owns a recycle channel; every published chunk carries a sender to it.
DetachedIndexBuilder::push_directory drains a listing in place (&mut DetachedDirectory,
children.drain(..), control.take()) instead of consuming it, and the consumer sends the
emptied Vec<DetachedDirectory> back. At its next publish the worker takes back what was
returned: it clears any children or control left in a listing the consumer did not
apply, keeps up to 16 listings (4 x DIR_CLAIM) whose child buffer is at most 256
children, keeps one emptied publish list, and frees the rest on its own thread.
begin_directory reuses a spare listing by clearing its path buffer and pushing the same
bytes to_path_buf would copy. Removed from the consumer per directory: the path buffer,
the child buffer, and (per chunk) the publish list. Ordering, parent-first causality,
and every listing's facts are unchanged; the builder does exactly the same work.
Precedent: H147 ScannerBatch::recycle on the transient tier (exp-151).

Budget: Linux 450k index tier 1.78x parfloor against the 1.4x closure threshold
(exp-141). The LD_PRELOAD allocator screen (report-2026-09-27) attributes the whole
1M indexed-tree gap to pdu, about 19% (1.39 -> 1.11-1.13 s), to glibc allocation
behavior. H159 reaches only the consumer-side cross-thread frees of listing buffers,
about 2-3 per directory (125k directories on balanced-1M), and whatever walker-side
arena waiting those frees cause through tcache mixing; it cannot reach walker-internal
churn (DirEntry names, joined paths). Expected share: part of that 19%, not all of it.

Prediction (deciding, Linux): probe default-tree wall down at least 3% with the 95%
interval entirely below zero; cold-scan-index also predicted to move (same mechanism,
index freed inside the timed region, so possibly diluted); peak RSS non-inferior (upper
bound at most +5%, the qualification limit). Allocation counters: allocs/entry down.

## Linux decision protocol (not run tonight: no Linux host reachable)

1. Control: release perf_probe and release fdu CLI of 56c506e1. Candidate: the same two
   builds of the H159 commit. Same toolchain, build argv recorded, copied outside trees.
2. Deciding subject: reconstructible linux-v6.12. Screening subject: linux-balanced-1m
   (python -m benchmarks.generate create --recipe balanced --entries 1000000, seed
   fdu-balanced-v1).
3. Probe: make perf-compare JOBS="default-tree cold-scan-index" TRIALS=12
   PERF_HOST_REGIME=quiet on each subject; primary default-tree wall_ns. Accept rule:
   median at least 3% faster and 95% interval entirely below zero, zero invalid samples,
   no fingerprint drift; peak RSS non-inferior.
4. Product: make perf-compare-tools with PERF_TOOL_CONTRACT=fdu-default-tree, anchor =
   control CLI, TOOL_ARGS="--tool candidate:fdu-default-tree=<candidate CLI>", 12
   adjacent pairs, 3 warmups, quiet, on both subjects; add the `fdu` indexed-tree
   contract (--cache off --depth 1 --limit 10), on which the 2026-09-27 pdu/diskus gap
   was measured, as a second tool on balanced-1m. Report both beside the probe verdict;
   they do not replace the probe primary.
5. Record with make perf-record (decision on the deciding subject), then perf-ledger and
   perf-report; update the H159 registry row and this bead.

## 2026-09-28 results (macOS only; Linux decision still pending)

Branch claude/perf-h159-recycle: 666b51f0 (mechanism), b1f57ecd (retention bounds),
then the record commit. Gates on b1f57ecd: cargo test -p fdu-core (835 lib tests plus
integration), cargo test -p fdu (including detached_performance_invariants), clippy
--workspace --all-targets --all-features -D warnings, fmt --check, make test-golden
(210 passed, no golden changed), make lib-only: all pass.
detached_bootstrap_matches_the_streaming_reducer_for_each_worker_count passes at 1-4
workers; two new tests pin listing reuse and the drain contract.

macOS (M1 Pro, 10 cores, internal APFS, bare metal, warm-steady). Release probes
56c506e1 vs candidate, 12 interleaved pairs. Every quiet attempt either was refused or
lost samples to the 25% busy bar (other workloads, load 5-26), so the recorded cells are
uncontrolled: exploration, not a held-out claim.

- exp-166 (superseded; first build 666b51f0, up to 16 spare listings of up to 256
  children per worker). system-private-frameworks: default-tree wall +1.32%
  [-5.10%, +5.20%], cold-scan-index +2.40% [-4.43%, +8.79%]; peak RSS +1.43%
  [+1.12%, +1.79%] and +1.54% [+0.52%, +2.99%]. rustup-toolchains default-tree peak RSS
  +5.04% [+2.89%, +11.52%] (25.1 -> 27.4 MiB), past the 5% margin. Retained spare
  listings are memory the consumer can no longer reuse.
- exp-167 (in-progress, kept candidate; b1f57ecd, reuse bounded to DIR_CLAIM listings
  of at most 64 children). system-private-frameworks: default-tree -1.41%
  [-5.32%, +4.41%], cold-scan-index +1.06% [-3.29%, +5.08%]; peak RSS -0.06%
  [-1.33%, +0.18%] and -1.20% [-1.55%, -0.30%]. rustup-toolchains: default-tree -3.21%
  [-5.27%, -0.69%] (uncontrolled, few directories: not claimed), cold-scan-index +0.07%
  [-7.73%, +4.78%]; peak RSS +2.77% [-0.96%, +4.89%] (+0.4 MiB).
- Counters (untimed scan-index, frameworks): allocations 1,654,218 -> 1,582,362
  (-4.3%), bytes 309.1 -> 290.0 MB (-6.2%); detached route confirmed.

Reading: no macOS regression (no median worse than +1.1% after the bound), but
non-inferiority at the 3% margin is not established on this host (upper bounds +4.4% and
+5.1%). The Linux cell above decides H159; the registry row and runbook Current Pickup
item 5 point at it. Close only after that cell is recorded.
