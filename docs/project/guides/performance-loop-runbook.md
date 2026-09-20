# The Performance Loop Runbook: One Unattended Round

How to run one turn of [the performance loop](performance-loop.md) on this host without
a person watching, from picking the hypothesis to the commit that records the verdict.

The loop guide is the protocol: why each step exists and what a result means.
This document is the checklist an agent follows at 3 a.m., and it is written to the
resume rule of
[the experiment-loop method](../specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md):
everything needed to pick the loop up mid-stream lives in the registry (what to try
next), the record (what has been tried), and here (how to run one round).
Every command below was run once while writing it.

Start at [Current Standing](#current-standing-2026-09-18). That section is the pickup:
standing best, host regime, Darwin subjects, and the next-up list with enough context to
start each item. Do not reconstruct the queue from chat, from `macos-agenda` priority
order, or from the 2026-08-23 Tier 1 list alone.

## Current Standing (2026-09-18)

Post-0.1.0 Darwin revisit on this desktop (Apple M1 Pro, Darwin 25.5.0, bare metal,
APFS). The engine that shipped in 0.1.0 has a unified request model, opened-root
serving, watch, `.gitignore` default-on, and a content sidecar.
Campaign 1 and campaign 2 remain the history; this standing is a registry and
measurement layer on top of them, not a rewrite of H86.

Branch `perf/campaign-next-2026-09-19`, stacked on
[#91](https://github.com/jlevy/fdu/pull/91) (`perf/campaign-quiet-2026-09-18` at
`e667b739`, which holds H115, H120, and the R1–R2 / S1–S3 review fixes).
Continue on the stacked branch.
Do not push to #91. Never merge.
Never force-push.

### Standing Best and Regime

**H133 / exp-132** is the latest wall-speed increment on deciding-scale
`content-cache-hit` (−6.37% on top of H131). **H131 / exp-130** remains the restore DFS
parent-path join (−4.07%). **H129 / exp-128** remains the restore-without-classify
accept (−13.11%). **H125 / exp-124** remains the restore-count completeness accept
(−8.03%). **H115 / exp-112** remains the restore-rebuild accept (−9.69%). **H120 /
exp-117** is the standing content-hit RSS best before H129 (peak RSS −10.13%; streaming
restore kept); H129 also moved peak RSS −11.83% on its own pair.

**exp-105** is the current rustup *probe* self-comparison baseline, 12-pair,
`os_cache: warm-steady`, **uncontrolled**.

| Job | Wall median | Peak RSS | Subject |
| --- | ---: | ---: | --- |
| `default-tree` | 149.8 ms | 33.0 MiB | rustup-toolchains, 77,132 entries / 73,714 files |
| `cold-scan-index` | 297.1 ms | 24.7 MiB | same |

Probe `default-tree` is about 492k files/s on that tree.
That sits above the README ballpark of 200K files/s, so the ballpark is not an overclaim
of engine capability, and it is **not a reason to raise it**.

**exp-107 / H108** is the installed-CLI determination on an immutable deciding tree:
`system-private-frameworks`, 158,705 entries / 96,542 files (35% directories), this
branch’s release CLI (`fdu 0.1.0-dev+gbd03cd6cc`). One OS warmup, then 12 isolated-cache
first/second `fdu PATH` pairs.
Quiet start gate passed (CPU busy 20.44%); final 26.08% broke the cell.
Labeled **uncontrolled**. The 25% bar was not lowered.

| Arm | Wall median | Peak RSS | files/s |
| --- | ---: | ---: | ---: |
| first `fdu PATH` | 2.100 s | 89.7 MiB | 46.0k |
| second `fdu PATH` | 2.060 s | 89.3 MiB | 46.9k |

Every second run stayed `cold scan`. Wall −0.63% [−4.97%, +4.05%]. **Confirmed.** No
engine patch.
This CLI cell is a directory-heavy system prefix, not a reason to lower the
README 200K capability ballpark (the rustup probe still sits above it).
Do not quote probe files/s as a product claim.

The 2026-09-18 [installed-CLI QA](../reports/report-2026-09-18-cli-installed-qa.md) is
still a different table: 34,145 files in 0.43 s (~79k files/s) on a mutating fdu
checkout. Cached lines/s was not re-measured.

**exp-106 / H107** (rejected): shipped `read_controls` vs `--no-controls` on the live
metabrowser checkout (145,931 entries).
Wall +1.64% [−4.00%, +4.37%]. User CPU +22% and RSS −6.9% cancelled on the critical
path. No engine patch was kept.

**exp-108 / H109** is the deciding-scale `content-cache-hit` **profile** on
`metabrowser-clone` (145,931 entries / 133,597 files).
Same-binary 12-pair, uncontrolled (quiet start gate failed; 25% bar not lowered).
Every sample was a sidecar hit (133,597 cache hits, 0 applied).

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,218.0 ms | 916.7 ms | 378.2 MiB |
| candidate | 1,219.9 ms | 906.4 ms | 379.6 MiB |

Self-comparison −0.29% [−1.09%, +1.05%]. **Baseline.** No engine patch.
`install_controls` is 7.2% of the profile / 7.5% of the engine (was 19.43%/25.5% on the
3k screening subject).
The hit path is `load_content` (60.5% of engine) and snapshot parse (25.6%). Path
rewrite is not justified.

**exp-109 / H112** splits that restore on the same `metabrowser-clone` tree (engine
digest re-observed to `3fbfed48…`; same shape).
12-pair current-best vs off-by-default phase timers.
Quiet start gate failed (31.7% busy); pair ran **uncontrolled**. The 25% bar was not
lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | --- |
| control | 1,195.5 ms | 891.2 ms | 387.6 MiB |
| candidate | 1,210.1 ms | 903.4 ms | 386.5 MiB |

Wall +0.31% [−0.81%, +1.63%], non-inferior.
**Baseline.** Timers kept (89 lines, off by default, no unsafe).
Apply dominates restore (timers 63.3%; counters-off sample 53.6% of `load_content`).
Candidates 25%; parse 8.5%. Every sample 133,597 cache hits / 0 applied; content digest
unchanged from exp-108.

**exp-110 / H113** tests the leftover completeness walk on the same `metabrowser-clone`
tree (engine digest unchanged).
12-pair current-best (timers in the binary, off) versus a file-count completeness check
instead of walking `analysis_candidates` for `len()`. Quiet start gate failed (27.8%
busy); pair ran **uncontrolled**. Initial busy 40.0%; final 63.65%. The 25% bar was not
lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,246.3 ms | 937.6 ms | 379.6 MiB |
| candidate | 1,156.8 ms | 836.9 ms | 375.9 MiB |

Wall −7.59% [−10.76%, +2.24%]. **Rejected.** Median past 3%; interval includes zero.
Component −13.07% [−14.30%, −7.89%]. Shortcut reverted.
Incomplete-sidecar fail-closed test kept.
Every sample 133,597 cache hits / 0 applied; content digest unchanged from exp-108 /
exp-109.

**exp-111 / H114** tests the leftover apply-path type-id `String` alloc on the same
`metabrowser-clone` tree (engine digest unchanged).
12-pair current-best at `c06d09e7` (timers in the binary, off) versus `get_mut` before
`entry` in `ContentRollUp::add`. Quiet start gate failed (49.5% busy); pair ran
**uncontrolled**. Initial busy 49.25%; final 25.78%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,328.9 ms | 1,003.4 ms | 386.5 MiB |
| candidate | 1,290.9 ms | 952.0 ms | 388.8 MiB |

Wall −0.56% [−17.92%, +4.79%]. **Rejected.** Median below 3%; interval includes zero.
Component −1.34% [−16.90%, +4.31%]. Type-id alloc trim reverted.
Every sample 133,597 cache hits / 0 applied; content digest unchanged from exp-108 /
exp-109 / exp-110.

**exp-112 / H115** tests one bottom-up content roll-up after sidecar restore inserts on
the same `metabrowser-clone` tree (engine digest unchanged).
12-pair current-best at `2736ec16` (timers in the binary, off) versus
`commit_without_rollup` plus `ContentIndex::rebuild_rollups` after the apply loop.
Quiet start gate failed (28.9% busy); pair ran **uncontrolled**. Initial busy 28.71%;
final 59.14%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,287.6 ms | 959.1 ms | 387.6 MiB |
| candidate | 1,174.1 ms | 855.6 ms | 389.8 MiB |

Wall −9.69% [−26.02%, −7.13%]. **Accepted.** Median past 3%; interval entirely below
zero.
Component −10.54% [−30.23%, −8.80%]. User CPU −8.23% [−11.11%, −7.73%]. Engine kept
(`7798fdc1`). Every sample 133,597 cache hits / 0 applied; content digest unchanged from
exp-108 / exp-109 / exp-110 / exp-111.

**H113 quiet confirmatory** after that accept (`fdu-rfr6`) was pre-registered as exp-113
on the same `metabrowser-clone` tree, control = this HEAD with H115 in.
`PERF_HOST_REGIME=quiet` refused at the start gate: CPU busy **46.7% > 25.0%**. No pair
ran. The file-count shortcut was not re-measured and is not in the engine.
exp-113 was not consumed.
**H113 still needs a quiet host.** Do not run another uncontrolled H113. Later the same
night a quiet CPU start passed, then the cell failed to hold: first thermal `fair` (0
paired samples), then 3 pairs / 16 invalid, then 9 pairs / 4 invalid.
Those incomplete cells are not a verdict.
The file-count shortcut is not in the engine.
exp-113 remains reserved.
The 02:44 PT overnight tick skipped further H113 for the rest of that night.

A 2026-09-19 stacked-PR retry (`fdu-rfr6`, ~10:31 PT) refused again at the start gate:
CPU busy **69.4% > 25.0%**. No pair ran.
The file-count shortcut was compiled only for that gate attempt and is not in the
engine. exp-113 remains reserved.
A later tick the same afternoon refused at **43.79%**, then **85.17%**. No pair.
Shortcut not compiled.
exp-113 remains reserved.
The leftover is still 16.0% of `content_open` after H115+H120 (exp-123). Next is H122.

**exp-114 / H116** tests restore without a full `analysis_candidates` Vec+HashMap on the
same `metabrowser-clone` tree (engine digest unchanged).
12-pair current-best at `7f289d5f` versus `Index::lookup` plus restore-only classify
skip. Quiet start gate refused (85.6% busy); pair ran **uncontrolled**. Initial busy
76.31%; final 72.15%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,475.4 ms | 1,122.9 ms | 377.3 MiB |
| candidate | 1,573.7 ms | 978.6 ms | 335.3 MiB |

Wall +8.70% [−19.33%, +63.90%]. **Rejected.** Median the wrong way; interval includes
zero. User CPU −15.65% [−16.12%, −14.32%]; peak RSS −11.27% [−11.65%, −10.98%]. Engine
reverted. Every sample 133,597 cache hits / 0 applied; content digest unchanged.
Do not retry H116 uncontrolled.

**exp-115 / H118** tests first-pass `analyze_index` insert-then-rebuild (H115’s restore
shape) on the same `metabrowser-clone` tree.
12-pair current-best at `55261e6c` versus `apply_restored_analysis` plus one rebuild
after the receive loop.
Quiet start gate refused (39.7% busy); pair ran **uncontrolled**. Initial busy 82.08%;
final 98.8%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 10,020.6 ms | 9,232.5 ms | 253.1 MiB |
| candidate | 10,022.1 ms | 9,151.2 ms | 254.2 MiB |

Component −2.60% [−12.00%, +23.86%]. **Rejected.** Median misses the 3% bar; interval
includes zero. Wall −5.01% [−13.28%, +23.39%] also includes zero.
User CPU −4.53% [−5.21%, −3.31%] moved.
File I/O hid the ancestor walk.
Engine reverted. Every sample 133,597 applied; content digest unchanged.
Do not retry H118 uncontrolled.

**H119** was profiled before any engine change on the same `metabrowser-clone`
`content-basic` path (12 s `sample`, 6 repeats, 76,605 stacks, counters off).
`read` 59.06%; `__open` 17.44%; `semaphore_wait_trap` 7.93%;
`BasicAccumulator::push_text` 5.47%; `fdu::scan` 0.13%; `getattrlistbulk` 0.48%. Walk
overlap cannot clear 3% wall.
`openat` from a retained parent dirfd is the leftover named cut and needs a new `unsafe`
block (person-gated).
No pair. No engine change.
Do not retry walk-overlap.

**exp-116 / H117** is the opened-retention determination on `system-private-frameworks`
(158,705 entries / 96,542 files).
Same probe both variants.
New mode `opened-second-report` opens, waits until Ready, runs the default tree report
twice, and times only the second read.
Quiet start gate refused (25.5% busy); pair ran **uncontrolled**. Initial busy 36.44%;
final 15.0%. The 25% bar was not lowered.

| Job | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| `default-tree` | 2,612.2 ms | 2,604.6 ms | 85.1 MiB |
| `opened-second-report` | 4,003.4 ms | 1.6 ms | 196.2 MiB |

Second retained report 1.6 ms versus one-shot 2,612.2 ms (~1,630×). **Confirmed.**
`default-tree` stays a cold scan.
Probe mode kept. Not a snapshot load on `fdu PATH`.

**exp-117 / H120** streams sidecar records into apply on `metabrowser-clone` (engine
digest unchanged). 12-pair current-best at `984e4618` versus parse-into-apply with no
decoded-records `Vec`. Quiet start gate refused (28.1% busy); pair ran **uncontrolled**.
Initial busy 19.82%; final 90.64%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,111.0 ms | 822.5 ms | 377.5 MiB |
| candidate | 1,103.2 ms | 812.5 ms | 339.4 MiB |

Peak RSS −10.13% [−10.49%, −10.03%]. **Accepted.** Wall −0.59% [−1.63%, +0.51%]
non-inferior. Digest identical.
Streaming restore kept.
H115 remains the standing wall-speed best.

**exp-118 / H122** is the deciding-scale metadata CLI/walk **profile** after the current
engine, on `system-private-frameworks` (digest unchanged).
Same-binary 12-pair `default-tree`. Quiet start gate refused (40.3% busy); pair ran
**uncontrolled**. Initial busy 49.45%; final 56.17%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,807.7 ms | 1,798.7 ms | 84.9 MiB |
| candidate | 1,873.4 ms | 1,866.5 ms | 85.6 MiB |

Self-comparison +0.18% [−2.91%, +13.43%]. **Confirmed.** Walk is 96.6–97.5% of
instrumented component (second run still a full walk; `snapshot_written` false).
Sample leftover: `__open` 56.74%, `getattrlistbulk` 17.79%, finish 0.3–0.4%. No Darwin
cut named. No engine patch.

**exp-122 / H122** is the tighter leftover after that profile, same subject (digest
unchanged). `dir_enumeration_calls` counts successful `getattrlistbulk` syscalls
including the empty terminator (off by default).
Same-binary 12-pair `default-tree`. Quiet start gate refused (30.3% busy); pair ran
**uncontrolled**. Initial busy 49.15%; final 100.0%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 2,408.2 ms | 2,401.0 ms | 84.7 MiB |
| candidate | 2,467.7 ms | 2,461.1 ms | 84.7 MiB |

Self-comparison −1.95% [−16.09%, +7.63%]. **Accepted** as a determination, not a speed
win. 77,509 enumeration calls / 55,256 opens = **1.403** per directory.
20 s sample (165,807 stacks): `__open` 50.36%, `getattrlistbulk` 18.95%, `fdu::scan`
3.02% (largest symbol 0.94%), allocator 3.00%. No userspace walk cut ≥3%. No walk-cut id
minted. Counter kept.

**exp-123 / H113** is the completeness leftover after H115+H120, on the frozen
`metabrowser-clone` (digest unchanged).
No shortcut compiled.
Same-binary 12-pair `content-cache-hit`. Labeled uncontrolled after a 43.64% pre-pair
busy check. Official initial busy 18.37% / thermal `fair`; final 45.72%. The 25% bar was
not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,116.5 ms | 832.2 ms | 331.9 MiB |
| candidate | 1,221.3 ms | 930.1 ms | 331.3 MiB |

Self-comparison +3.26% [−1.37%, +13.31%]. **Accepted** as a determination.
20 s sample: completeness walk (`open_for_report` lib.rs:601–602) **16.0%** of
`content_open` (2,136 / 13,357). exp-109 was 12.6%. About 12% of H121 claim-grade wall.
H113 later superseded by H125. exp-113 unused.

**exp-124 / H125** skips that second walk by comparing `hits` to the candidate count
restore already stored, on the same frozen `metabrowser-clone`. Not the file-count
heuristic. 12-pair `content-cache-hit`. Quiet start this tick refused H113 at 45.48%. An
H125 quiet pair started (24.23%) but did not hold (13 invalid); not a verdict.
Claim-grade pair **uncontrolled**. Initial busy 43.98%; final 26.55%. The 25% bar was
not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,063.1 ms | 783.4 ms | 332.8 MiB |
| candidate | 975.1 ms | 688.4 ms | 333.4 MiB |

Wall −8.03% [−10.79%, −7.79%]. **Accepted.** Digest `3be19a3e…`. Engine kept
(`be8d4d69`). H113 superseded.

**exp-125 / H126** is the post-H125 leftover **profile** on the same frozen
`metabrowser-clone`. Same-source 12-pair `content-cache-hit`. Quiet not retried this
cell. Pair **uncontrolled**. Completeness walk is gone (1 sample / 15,296). First
`analysis_candidates` walk remains (15.7% of `content_open`). Restore mix unchanged
(candidates ~48%, apply ~43%). No new userspace cut.
Do not retry H116.

**exp-126 / H127** is first-pass walk versus opened-discovery I/O on the same frozen
`metabrowser-clone`. Same-binary 12-pair of `cold-scan-index` and `opened-discovery`.
Official quiet check 53.86% busy; pair **uncontrolled**. Initial busy 35.11%; final
55.87%. Opened component 2,761 ms versus first-pass 315 ms (~8.8×). Same 11,517 dir
opens. First-pass 1.952 `getattrlistbulk`/dir.
Opened uses `read_dir`+`fstatat`; 11,524 journal clones; 1.12M live roll-up merges.
No smallest cut.

**exp-127 / H128** is `default-tree` leftover on the same frozen file-heavy clone after
H122 (frameworks). Same-binary 12-pair.
Official quiet check 39.06% busy; pair **uncontrolled**. Initial busy 38.02%; final
43.96%. Wall 355.8 ms / component 343.4 ms.
Walk 92.9% of the matching counters-on hit.
Snapshot present and not loaded.
1.952 `getattrlistbulk`/dir.
No new cut.

**exp-128 / H129** skips classify on cache-only restore (walk and apply self-check),
keeping the HashMap and `path_of`, on the same frozen `metabrowser-clone`. Not H116.
12-pair `content-cache-hit`. Quiet this tick refused at 31.53%. Pair **uncontrolled**.
Initial busy 33.76%; final 28.67%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 986.2 ms | 701.9 ms | 334.9 MiB |
| candidate | 855.6 ms | 567.1 ms | 294.2 MiB |

Wall −13.11% [−20.22%, −12.67%]. **Accepted.** Digest `3be19a3e…`. Engine kept
(`6887a864`). `path_of` remains.

**exp-129 / H130** is leftover after H129 on the same frozen clone.
Same-source 12-pair `content-cache-hit`. Official quiet check 34.97% busy; pair
**uncontrolled**. Initial busy 66.8%; final 66.6%. Wall −0.21% [−1.77%, +21.32%].
Attachment only. Classify 0 of `content_open`. Completeness 0. Snapshot 43.3%. `path_of`
11.85%. `commit_record` 11.01%. No engine patch.

**exp-130 / H131** joins the parent path the restore DFS already holds, instead of
`path_of` per file, on the same frozen `metabrowser-clone`. Not H116. Public `path_of`
stays. 12-pair `content-cache-hit`. Quiet this tick refused at 27.23%. Pair
**uncontrolled**. Initial busy 25.84%; final 25.73%. The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 857.8 ms | 568.8 ms | 295.8 MiB |
| candidate | 824.3 ms | 535.1 ms | 296.8 MiB |

Wall −4.07% [−4.54%, −3.28%]. **Accepted.** Digest `3be19a3e…`. Engine kept
(`7840ce9b`).

**exp-131 / H132** is leftover after H131 on the same frozen clone.
Same-source 12-pair `content-cache-hit`. A pre-pair busy check read 24.38%;
`PERF_HOST_REGIME=quiet` then refused.
Pair **uncontrolled**. Initial busy 26.27%; final 26.67%. Wall −0.02% [−0.67%, +0.70%].
Attachment only. Restore-walk `path_of` 0 of `content_open`. Snapshot `path_of`
(`insert_loaded_child`) 9.89%, discarded because one-shot load has `serving = None`.
Completeness still 0. Snapshot 42.78%. No engine patch.

**exp-132 / H133** skips `path_of` in `insert_loaded_child` when serving is off, on the
same frozen `metabrowser-clone`. Not H131. Not H109. 12-pair `content-cache-hit`. Quiet
this tick refused at 27.87%. Pair **uncontrolled**. Initial busy 33.96%; final 29.48%.
The 25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 829.5 ms | 541.2 ms | 297.3 MiB |
| candidate | 778.0 ms | 487.4 ms | 297.4 MiB |

Wall −6.37% [−18.23%, −5.66%]. **Accepted.** Digest `3be19a3e…`. Engine kept
(`143a1c73`).

**exp-133 / H134** is leftover after H133 on the same frozen clone.
Same-source 12-pair `content-cache-hit`. Quiet this tick refused at 28.07%. Pair
**uncontrolled**. Initial busy 29.36%; final 40.04%. Wall −0.37% [−0.82%, +0.38%].
Attachment only. Snapshot `path_of` 0 of `content_open`. Completeness still 0.
Restore-walk `path_of` still 0. Classify still 0. Snapshot 37.94%.
`rebuild_content_rollups` 18.72%. `insert_loaded_child` 16.33%.
`reclassify_controlled_subtrees` 12.07%. `commit_record` 12.01%. No engine patch.

**exp-134 / H135** is leftover after H124 on the same frozen clone.
Same-source 12-pair `content-basic`. Quiet this tick refused at 56.4%. Pair
**uncontrolled**. Initial busy 35.75%; final 82.48%. Wall −3.70% [−8.25%, +6.96%].
Attachment only. Interval includes zero.
First-pass leftover is still file I/O (`read` 58.87%, `__open` 16.09%). `classify_with`
2.02% of process. `commit_record` 0.59%. `merge_ancestors` 0.43%. No engine patch.

**exp-135 / H136** is leftover after H128 on the same frozen clone.
Same-source 12-pair `default-tree-first`. Quiet this tick refused at 75.4%. Pair
**uncontrolled**. Initial busy 46.59%; final 88.11%. Wall +0.18% [−15.02%, +38.62%].
Attachment only. Every timed sample wrote the snapshot (10.5 MiB). Walk 83–88% of
first-run component.
Isolated `snapshot-save` 45.3 ms (~11–16%). No engine patch.

**exp-136 / H137** is leftover on deciding-scale `content-query` after H123. Same-source
12-pair. Quiet this tick refused at 76.8%. Pair **uncontrolled**. Initial busy 64.19%;
final 54.35%. Wall −3.27% [−14.42%, +4.41%]. Attachment only.
Interval includes zero.
Component ~26.8–27.3 s for 100 four-view reports (~278 ms / report).
`every_entry` runs once per unfiltered metric view.
No engine patch.

**exp-137 / H138** shares one `every_entry` walk across unfiltered entry-row views.
Control leftover HEAD probe; candidate H138. Quiet this tick refused at 93.2%. Pair
**uncontrolled**. Initial busy 69.09%; final 70.06%. Wall −18.76% [−22.86%, −13.69%].
Component −24.61% [−29.82%, −20.56%]. Engine kept (`a5c98d59`).

**exp-121 / H124** is the first-pass analyze I/O **profile** on the frozen
`metabrowser-clone` (146,047 entries / 133,708 files; digest `dc0df263…`). Path-binary
already skipped (8,022 files).
Opens 125,686; empty 731; discovered-binary after open 6,137. Read calls 249,533 (~2 per
open). Bytes read 951,822,681. Same-binary 12-pair `content-basic`. Quiet start gate
refused (29.6% busy); pair ran **uncontrolled**. Initial busy 68.55%; final 63.33%. The
25% bar was not lowered.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 9,014.4 ms | 8,312.1 ms | 254.2 MiB |
| candidate | 9,036.5 ms | 8,382.9 ms | 254.9 MiB |

Self-comparison −4.22% [−20.79%, +5.10%]. **Rejected.** Every admitted open is required
for lines; skippable share under 1% of wall.
A larger read chunk cannot clear 3%. No engine patch.

### Darwin Subjects

The 2026-08 nominated metabrowser corpus path is gone from disk.
The rustup store is 77k entries, not the 175k recorded in exp-066. Re-observed shapes
live in
[`nominated-subjects-darwin-arm64.json`](../reports/nominated-subjects-darwin-arm64.json).
Absolute paths live only in the gitignored `explorations/benchmarks/subjects.local.json`
(labels: `rustup-toolchains`, `metabrowser-clone`, `system-private-frameworks`,
`cargo-registry-src`). Read them from there.
Do not type a path into a commit.

`cargo-registry-src` (~22k) screens; it cannot decide a 3% verdict.
`system-private-frameworks` was the H108 / H117 subject (exp-107, exp-116); digest
unchanged from the nomination.
`metabrowser-clone` was the H109 / H112 / H113 / H114 / H115 / H116 / H118 / H120 / H121
/ H124 / H125 / H126 / H127 / H128 / H129 / H130 / H131 / H132 / H133 / H134 / H135 /
H136 / H137 / H138 subject (exp-108 through exp-112, exp-114, exp-115, exp-117, exp-120,
exp-121, exp-123, exp-124, exp-125, exp-126, exp-128, exp-129, exp-130, exp-131,
exp-132, exp-133, exp-134, exp-135, exp-136, exp-137); same shape as exp-106, engine
digest unchanged (`3fbfed48…`). A 2026-09-19 re-observe drifted to 145,988 entries /
133,654 files (digest `cc517e78…`); commit a fresh subjects document with the next
metabrowser cell (`make perf-subjects`). The CLI QA medium tree was skipped:
deciding-scale but mutating.
`system-private-frameworks` was also the H122 subject (exp-118); digest unchanged.

### Next Up

Take these in order.
Source of truth:
[the remaining-headroom block](../specs/active/plan-2026-09-19-post-h115-remaining-headroom.md).
The registry row in [the loop guide](performance-loop.md#current-engine-010) is the full
statement. Overnight H116–H120 is done; do not retry those.
H139 is recorded on stacked [#94](https://github.com/jlevy/fdu/pull/94) as exp-138. H140
is recorded there as exp-139. H141 is recorded there as exp-140. H111 is recorded there
as exp-141 (floor/RSS fail).
H142 is recorded there as exp-143 (file I/O leftover).
H143 is recorded there as exp-142 (walk floor + retained-index RSS). Do not mint those
ids on this Darwin branch.
Next free unused hypothesis id is **H149** (H144–H146 reserved on the Linux iteration
stack; H147 used for the Linux transient recycle keep; H148 used for the Linux PGO
screen; H72 used existing id).
Next free Linux experiment id is **exp-155**. Do not mint another meaning for H91–H106.
Next free Darwin experiment id after the Linux reservation is assigned on #94; do not
take exp-138 here. exp-113 remains reserved unused (H113 superseded).

This stacked session skipped H113 (quiet gates including 45.48%, 53.86%, 31.53%, 34.97%,
27.23%, a 24.38% pre-check that did not hold, and 28.07%), accepted H125 (exp-124,
restore-count completeness), confirmed H126 (exp-125 leftover; completeness gone),
confirmed H127 (exp-126; opened-discovery ~8.8× first-pass), confirmed H128 (exp-127;
file-heavy `default-tree` walk still the job), accepted H129 (exp-128,
restore-without-classify), confirmed H130 (exp-129 leftover; classify gone; `path_of`
11.85%), accepted H131 (exp-130, restore DFS parent-path join), confirmed H132 (exp-131
leftover; restore-walk `path_of` gone; snapshot `path_of` 9.89% discarded on one-shot
`serving=None`), accepted H133 (exp-132, skip unused snapshot path reconstruction),
confirmed H134 (exp-133 leftover; snapshot `path_of` gone; no new cut), confirmed H135
(exp-134 leftover; first-pass `content-basic` still file I/O; no new cut), confirmed
H136 (exp-135 leftover; first-run walk still the job; snapshot write ~45 ms and not
skippable), confirmed H137 (exp-136 leftover; ~278 ms per four-view content report;
`every_entry` once per unfiltered metric view; no engine patch), accepted H138 (exp-137,
share one `every_entry`; wall −18.76%), hunted H107 (no ignore-is-the-walk subject),
recorded exp-122 (H122 leftover), and recorded exp-123 (H113 leftover then 16% of
`content_open`). Earlier the same day: H113 69.4%, H122 (exp-118), H123, H121, H124. Do
not start H107 without an ignore-is-the-walk subject.
Do not retry metabrowser for H107 (exp-106). Do not start H111 on Darwin (no Linux).
H111 is recorded fail on [#94](https://github.com/jlevy/fdu/pull/94) (exp-141). Do not
pad cache-hit, first-pass I/O, or walk leftovers already recorded.
Do not raise the README 200K files/s or 4M cached lines/s.

1. **H138** (`fdu-sce1`). **Accepted** (exp-137). Share one `every_entry` walk across
   unfiltered entry-row views.
   Wall −18.76% [−22.86%, −13.69%] on frozen `metabrowser-clone`. Engine kept
   (`a5c98d59`). Quiet this tick 93.2%; pair initial 69.09% final 70.06%. **H137**
   (`fdu-nc3b`). **Confirmed** (exp-136). Content-query leftover after H123 is
   `every_entry` on each unfiltered Types / Families / Languages / Documents view, about
   278 ms per four-view report.
   Filtered views already share one walk.
   No engine patch. Quiet that tick 76.8%; pair initial 64.19% final 54.35%. **H136**
   (`fdu-w9jb`). **Confirmed** (exp-135). First-run leftover after H128 is still the
   walk (83–88% of `default-tree-first` component).
   Isolated `snapshot-save` 45.3 ms (~11–16% of first-run).
   Write is ≥3% and not skippable.
   No engine patch. Quiet that tick 75.4%; pair initial 46.59% final 88.11%. **H135**
   (`fdu-hh0t`). **Confirmed** (exp-134). First-pass leftover after H124 is still file
   I/O (`read` 58.87%, `__open` 16.09% of process).
   `classify_with` 2.02%. `commit_record` 0.59%. `merge_ancestors` 0.43%. No skippable
   ≥3% userspace cut. No engine patch.
   Quiet that tick 56.4%; pair initial 35.75% final 82.48%. **H134** (`fdu-03pr`).
   **Confirmed** (exp-133). Snapshot `path_of` gone (0 of `content_open`). Completeness
   still 0. Restore-walk `path_of` still 0. Classify still 0. Snapshot 37.94%. Remaining
   leftover is already-landed restore work and already-rejected stages
   (`rebuild_content_rollups` 18.72%, `insert_loaded_child` 16.33%,
   `reclassify_controlled_subtrees` 12.07%, `commit_record` 12.01%). No engine patch.
   Quiet that tick 28.07%; pair 29.36%. **H133** (`fdu-7m91`). **Accepted** (exp-132).
   Skip unused snapshot `path_of` when serving is off.
   Wall −6.37% [−18.23%, −5.66%] on frozen `metabrowser-clone`. Engine kept
   (`143a1c73`). Quiet that tick 27.87%. **H132** (`fdu-8z5i`). **Confirmed** (exp-131).
   Restore-walk `path_of` gone (0 of `content_open`). Snapshot `path_of`
   (`insert_loaded_child`) 9.89%, discarded because one-shot load has `serving = None`.
   Completeness still 0. Snapshot 42.78%. No engine patch.
   Quiet that tick refused after a 24.38% pre-check; pair 26.27%. **H131** (`fdu-1dxc`).
   **Accepted** (exp-130). Restore DFS joins the parent path.
   Wall −4.07% [−4.54%, −3.28%] on frozen `metabrowser-clone`. Engine kept (`7840ce9b`).
   Public `path_of` stays.
   Quiet that tick 27.23%. **H130** (`fdu-ajbw`). **Confirmed** (exp-129). Restore
   classify gone (0 of `content_open`). Completeness still 0. Snapshot 43.3%. `path_of`
   11.85%. No engine patch.
   Quiet that tick 34.97%. **H129** (`fdu-qjjh`). **Accepted** (exp-128). Restore omits
   classify. Wall −13.11% [−20.22%, −12.67%] on frozen `metabrowser-clone`. Engine kept
   (`6887a864`). HashMap stays.
   Quiet that tick 31.53%. **H125** (`fdu-wd4q`). **Accepted** (exp-124). Restore-count
   completeness. Wall −8.03% [−10.79%, −7.79%]. Engine kept (`be8d4d69`). H113
   superseded. Do not retry file-count.
   **H126** (`fdu-16jh`). **Confirmed** (exp-125). Completeness walk gone (0.007% of
   `content_open`). First candidates walk was 15.7%; H129 took the classify half.
   **H127** (`fdu-v12n`). **Confirmed** (exp-126). Opened-discovery 2,761 ms versus
   first-pass 315 ms (~8.8×). `read_dir`+`fstatat` versus `getattrlistbulk`; journal
   clones remain. Opened roots run no analyzers.
   No smallest cut. **H128** (`fdu-0wym`). **Confirmed** (exp-127). `default-tree` walk
   still the job on file-heavy metabrowser (92.9% of component).
   1.952 `getattrlistbulk`/dir.
   Snapshot not loaded.
   No new cut.

2. **H122** (`fdu-ytg5`). **Confirmed** (exp-118, tighter leftover exp-122). Walk is
   96.3–97.5% of deciding-scale `default-tree` component on `system-private-frameworks`.
   Leftover is directory `__open` (55,256) plus `getattrlistbulk` at **1.403 calls/dir**
   (77,509). No userspace symbol ≥3%. No Darwin walk cut.
   Do not retry as a snapshot-load, finish trim, or consume cut.

3. **H107** (`fdu-jcfn`). **Skipped.** Hunt recorded in exp-122: rustup and frameworks
   have no `.gitignore`; metabrowser is exp-106; cargo-registry screens only; tbd and
   urollup have the same `dir_opens` with controls on and off.
   Ignore does not skip descent.
   Do not retry metabrowser.
   Do not invent a subject.

4. **H123** (`fdu-rum0`). **Confirmed** (exp-119). Product `Index.report()` /
   `query::report` second pass 1.7 ms versus one-shot 2,078.3 ms (~1,222×). Probe kept.
   H117 remains the opened-root probe.
   Do not load a snapshot on `fdu PATH`.

5. **H121** (`fdu-vf4b`). **Confirmed** (exp-120). Apply 43% of restore after H115+H120;
   candidates 48%. No stage ≥50%. No apply cut.
   Do not retry H116.

6. **H124** (`fdu-i39y`). **Rejected** (exp-121). Every admitted open is required for
   lines; skippable share under 1% of wall; read calls already one data chunk per file.
   Do not retry a type/size gate or a larger read chunk.
   `F_RDADVISE` is person-gated `unsafe`.

7. **H111** (`fdu-jekg`). Linux floor stage of H86. **Failed** on
   [#94](https://github.com/jlevy/fdu/pull/94) (exp-141, virtualized).
   Recorded:
   [Linux parallel validation](../specs/active/plan-2026-09-19-linux-parallel-validation.md).
   Leftover is H143. Do not restart the rewrite.
   Do not treat a Darwin cell as this claim.

**H108** (`fdu-1a4z`, confirmed in exp-107). Do not open a cache/one-shot patch.
**H109** (`fdu-8nwq` / `fdu-hzyb`, screened in exp-108). Do not land a Path rewrite.
**H115** (`fdu-wx15`) landed in exp-112. **H120** landed in exp-117. Do not retry them.
`fdu-jxhk` remains the EntryId composite; do not restart that rewrite.

H110 needs a new named mechanism.
exp-126 names the leftovers (journal clone per directory, `read_dir`+`fstatat`, 1.12M
live merges) and does not compile a cut.
Do not retry H104–H106. H131 took the restore-walk ancestor half of `path_of` (exp-130).
Do not retry that join.
H132 confirmed restore-walk `path_of` is gone (exp-131). H133 took the unused snapshot
`path_of` skip (exp-132). H134 confirmed snapshot `path_of` is gone (exp-133). Remaining
cache-hit leftover is already-landed restore work and already-rejected stages.
H135 confirmed first-pass leftover is still file I/O (exp-134). H136 confirmed first-run
leftover is still the walk (exp-135); snapshot write ~45 ms is not skippable.
H137 confirmed content-query leftover is `every_entry` per unfiltered metric view
(exp-136). H138 took the share-one-walk cut (exp-137, −18.76% wall).
Do not invent a skip.
Do not join a parent path on snapshot insert.
Do not defer snapshot `merge_upward`. Do not retry H109.

### Dead Ends

- Do not restart the H86 structural rewrite (`fdu-xde5` remains for H111 only).
- Do not pad the night with unmeasured engine refactors.
- Do not invent a capability that exists only on the command line.
- Do not raise or lower the README 200K files/s or 4M cached lines/s from a probe cell
  or from one directory-heavy CLI tree.
- Do not treat campaign-1 no-controls walls as the current default speed; treat their
  tallies as a different answer (exp-106).
- Do not force a metadata one-shot to load its snapshot (H108 / H9).
- Do not land an H109 control-matcher Path rewrite.
  The deciding-scale share collapsed (exp-108).
- Do not retry a sidecar parse-speed or instruction trim.
  H112 (exp-109) put parse at 8.5% of restore.
  H121 (exp-120) re-checked the mix: apply 43%, candidates 48%. No apply cut.
- Do not retry the H113 file-count completeness shortcut (exp-110; quiet never held).
  Median −7.59% but the interval included zero; the shortcut is reverted.
  H125 took the restore-count form of the same skip (exp-124). H113 is superseded.
  exp-113 unused.
- Do not retry the H114 type-id `String` alloc trim on `ContentRollUp::add` (exp-111).
  Wall −0.56% [−17.92%, +4.79%]; the trim is reverted.
- Do not retry H115. exp-112 accepted the restore-only bottom-up rebuild (−9.69%
  [−26.02%, −7.13%]). Do not restart the `fdu-jxhk` EntryId rewrite from that result.
- Do not retry H116 on another uncontrolled cell (exp-114). Wall +8.70%
  [−19.33%, +63.90%]; the path-lookup patch is reverted.
  User CPU and RSS moved and are not an accept.
- Do not retry H118 on another uncontrolled cell (exp-115). Component −2.60%
  [−12.00%, +23.86%]; file I/O hid the ancestor walk.
  The insert-then-rebuild first-pass patch is reverted.
  User CPU is not an accept.
- Do not retry H119 walk-overlap of analyze I/O with the metadata walk.
  `fdu::scan` is 0.13% of deciding-scale `content-basic`. `openat` is person-gated
  (`unsafe`).
- Do not retry H124 type/size gate or a larger `READ_CHUNK_BYTES` after exp-121.
  Path-binary is already skipped; remaining opens are required for lines.
  Read calls are already one data chunk plus EOF. `F_RDADVISE` / `F_RDAHEAD` is
  person-gated `unsafe`.
- Do not retry H129 after exp-128. Restore-without-classify is in the engine
  (`6887a864`).
- Do not compile a leftover H130 cut after exp-129. Classify is gone.
  H131 later took the named `path_of` ancestor walk (exp-130).
- Do not retry H131 after exp-130. The DFS parent-path join is in the engine
  (`7840ce9b`). Public `path_of` stays.
- Do not compile a leftover H132 cut after exp-131. Restore-walk `path_of` is gone.
  H133 later took the unused snapshot `path_of` skip (exp-132).
- Do not retry H133 after exp-132. The unused snapshot path skip is in the engine
  (`143a1c73`). Public `path_of` stays.
  Opened-root serving insert stays.
- Do not compile a leftover H134 cut after exp-133. Snapshot `path_of` is gone.
  Remaining leftover is already-rejected or already-landed.
- Do not compile a leftover H135 cut after exp-134. First-pass leftover is still file
  I/O. Do not invent a first-pass classify skip.
- Do not compile a leftover H136 cut after exp-135. First-run leftover is still the
  walk. Snapshot write is not skippable.
  Do not retry H100. Do not mint H78/H92.
- Do not compile a leftover H137 cut after exp-136 unless it is the named share-one-walk
  of unfiltered metric views.
  H138 took that cut (exp-137). Do not invent a cache-hit skip.
- Do not retry H138 after exp-137. The shared `every_entry` walk is in the engine
  (`a5c98d59`).
- Do not register another restore alloc-trim, parse-speed cut, or H103-shaped
  instruction rewrite; those are on the remaining-headroom block’s rejected list.
- A quiet cell may not hold on this desktop.
  Attempt `PERF_HOST_REGIME=quiet` first; if it fails or the final snapshot exceeds 25%
  busy, label **uncontrolled** and do not claim quiet.
  Do not lower the 25% busy bar so the cell passes.
  H113’s quiet-only exception is closed: that hypothesis is superseded.

### Process Pack

| Document | Role |
| --- | --- |
| This standing section | Standing best, next-up order |
| [Post-H115 remaining-headroom block](../specs/active/plan-2026-09-19-post-h115-remaining-headroom.md) | Remaining Darwin queue: H107 (no subject) |
| [Linux parallel validation](../specs/active/plan-2026-09-19-linux-parallel-validation.md) | Linux recorded on #94: H139–H142 same; H111 fail; H143 leftover confirmed. Ready to merge onto #92 |
| [The loop guide](performance-loop.md) | Protocol, accept rule, hypothesis registry |
| [The campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) | Floor-anchored strategy; the 2026-08-23 Tier 1–3 list is history |
| [The instrumentation playbook](performance-instrumentation-playbook.md) | Instrument before optimizing; `FDU_COUNTERS=1` |
| [First Principles](../architecture/fdu-design-principles.md#first-principles) | Caching never changes semantics; one model per concept; keep/reject rules |
| [Engine architecture](../architecture/fdu-engine-architecture.md) | Index ownership, one-shot vs opened serving, journals, observers |
| [The ledger](../reports/report-2026-08-10-fdu-performance-experiments.md) | Every verdict, including negatives |
| [The evidence report](../reports/report-2026-08-20-fdu-performance-evidence.md) | Charted view; regenerate after every record |
| [Platform tuning](platform-tuning.md) | Which shipped constant was measured in which regime |
| [Installed-CLI QA](../reports/report-2026-09-18-cli-installed-qa.md) | Product-path table; not interchangeable with probe jobs |

## Linux Standing (2026-09-20)

Stacked [#94](https://github.com/jlevy/fdu/pull/94) on `perf/campaign-linux-2026-09-19`,
rebased onto [#92](https://github.com/jlevy/fdu/pull/92) `937f9445` (R1–R3 plus
`c441edf6` single-view Cow borrow and the #91 restore-evidence repair).
Ready to merge onto #92. This is not a Linux floor pass.
H141’s four-view `content-query` still shares one walk (`row_consumers > 1`). A
single-view report now borrows instead of cloning; that does not change the four-view
cell. H139 was a well-formed cache-hit tree; snapshot alias reject and restore-timing
evidence do not change that cell.
4-core KVM Intel Xeon, 16 GiB, Linux 6.12.94+, ext4, virtualized.
Same host class as exp-103. Linux quiet uses load/core ≤ 0.25 (instantaneous busy% is
Darwin-only).

**H139 / exp-138** is the standing Linux cache-hit replication: **same**. Quiet 12-pair
`content-cache-hit` of #91 `e667b739` (H115+H120) versus this engine on reconstructible
`linux-v6.12` (92,474 entries / 86,643 files, tag `v6.12` / `adc21867`). Wall −22.48%
[−23.46%, −21.39%]. Peak RSS −10.24%. Digest identical.
Load/core 0.115–0.153 held.
A clean metabrowser clone here is 916 entries and was not the subject.

**H140 / exp-139** is the standing Linux walk leftover: **same** identity.
Walk 95.7–96.1% of `default-tree` component.
Leftover is `getdents64`+`statx` (one open/dir, one stat/entry).
Quiet attempt invalidated 23/24 timed samples.
Do not compile a walk trim.
Do not retry H71.

**H141 / exp-140** is the standing Linux content-query replication: **same**.
Uncontrolled 12-pair `content-query` of the same #91 control versus this engine (H138
share-one-walk) on `linux-v6.12`. Wall −17.60% [−18.07%, −17.17%]. Component −21.32%.
Peak RSS +0.24% non-inferior.
Digest identical. Quiet start 0.185/core did not hold.
Do not retry H138.

**H111 / exp-141** is the standing Linux floor/RSS stage: **failed** on this virtualized
host. 450k index 1.78× `parfloor` vs 1.4×; RSS 5.20× `arena_spike` vs 3×. Aggregate on
nominated reals 1.59× / 1.86× vs 1.25×. p95/median passes.
Quiet requested; table uncontrolled.
Do not restart H86.

**H143 / exp-142** is the standing leftover after that fail: **same** identity.
Quiet 12-pair `cold-scan-index` on reconstructible `linux-450k`. Walk 94.7–94.8% of
component. Detached finish 4.3% is the H86 consume.
Peak RSS ~150 MiB. No new cut.

**H142 / exp-143** is the standing Linux first-pass leftover: **same** identity.
Uncontrolled 12-pair `content-basic` on `linux-v6.12`. 86,634 opens, 184,057 reads
(~2.12/file). Digest identical.
Quiet start 0.036/core did not hold.
Do not retry H124.

**H145 / exp-145** is the standing Linux opened-discovery leftover: **same** identity.
Uncontrolled 12-pair `opened-discovery` on `linux-v6.12`. 5,772 journal clones; 438,021
live roll-up merges; opened component ~2.75× first-pass (Darwin H127 was 8.8×). Quiet
start 0.151/core did not hold.
Do not port `macos_bulk`.

**H84 / exp-146** is the standing Linux thread-policy screen: **unlock silent**. ~2
µs/entry vs 30 µs; expansions 0; start 4 / reserve 8. Named-job `--threads 8` is not a
3% win (default aggregate +1.75% on `linux-v6.12`; **+7.12% quiet regression on
`/usr`**, exp-149). `--no-controls` aggregate is a warm sign (−5.42% quiet on
`linux-v6.12`; −10.06% quiet on nominated `/usr`, exp-148). Do not ship `PORTABLE`.
`fdu-tk1b` stays open.

**H146 / exp-147** is the standing Linux first-run leftover: **same** identity.
Quiet 12-pair `default-tree-first` on `linux-v6.12`. Walk 93% of component.
Isolated `snapshot-save` 23.6 ms (~5%) is ≥3% and not skippable.
Load/core 0.082–0.119 held.
Do not retry H100.

**H85 / exp-150** is the standing Linux transient-recycle screen: **rejected** against
its 20% bar. Quiet `linux-v6.12` `--no-controls` aggregate −4.98%. Incomplete quiet
`linux-450k` −11.31% (n=7). Peak RSS flat.
Do not lower the bar.

**H147 / exp-151** is the 3% keep of that same patch.
Quiet `linux-v6.12` −4.98% [−5.92%, −4.33%]. Default gitignore-on placebo includes zero.
Engine kept (`5c6e6394`). Unmeasured on macOS.

**H72 / exp-152** is the standing Linux source-tree `d_type` screen: **rejected**
against 3%. Quiet `linux-v6.12` `--no-controls` aggregate −1.63% [−3.33%, −0.72%]. Stats
92,474 → 86,644. RSS flat.
Same class as the earlier −1.4% on 6.4% directories.

**H72 / exp-153** is the directory-heavy keep of that same patch.
Quiet nominated `/usr` −9.01% [−12.52%, −6.30%]. 22% dirs+symlinks.
RSS flat. Fresh baseline (image digest had drifted).
Engine kept (`f841662c`). Unmeasured on macOS.

This host’s nominated set is
[nominated-subjects-linux-x86_64.json](../reports/nominated-subjects-linux-x86_64.json):
`linux-v6.12` and `usr-prefix` decide; rustup screens (46,632 entries).
Two deciding characters, so the set cannot yet carry a ranking or transfer claim.

### After #94 (Linux)

[#94](https://github.com/jlevy/fdu/pull/94) is recorded and ready to merge onto #92.
Further Linux cells are on
[Linux performance iteration](../specs/active/plan-2026-09-20-linux-performance-iteration.md)
(`cursor/linux-perf-iterate-de1b`).

1. **H144** — Linux cache-hit leftover after the landed stack.
   **Same leftover identity** (exp-144, quiet).
   Already-landed restore work.
   Do not retry H125–H133.
2. **H145** — Linux opened-discovery leftover (H127 analog).
   **Same leftover identity** (exp-145, uncontrolled).
   Journal clones plus live roll-ups; 2.75× first-pass.
   Do not port `macos_bulk`.
3. **H84** — adaptive unlock / thread-policy screen (`fdu-tk1b`). **Confirmed silent**
   (exp-146). Named-job `--threads 8` is not a 3% win.
   `--no-controls` is a warm sign, not a shipped `PORTABLE` constant.
4. **H146** — Linux first-run leftover after H140 (H136 analog).
   **Same leftover identity** (exp-147, quiet).
   Walk 93%; isolated save ~24 ms not skippable.
   Do not retry H100.
5. **H85 / H147** — transient `--no-controls` aggregate recycle.
   H86 detached arenas did not consume `RetainedState::Summary`. **H85 rejected**
   against its 20% bar (exp-150, quiet `linux-v6.12` −4.98%). **H147 accepted** as the
   3% keep (exp-151, same cell).
   Engine kept (`5c6e6394`). Peak RSS flat.
   Default gitignore-on placebo includes zero.
   Do not retry H85’s 20% bar.
   Do not restart H86. Do not retry H71.
6. **H72** — `d_type` skip on transient summary.
   **Rejected** on reconstructible `linux-v6.12` (exp-152, −1.63%). **Accepted** on
   nominated `/usr` (exp-153, −9.01% [−12.52%, −6.30%]). Engine kept (`f841662c`). Do
   not retry H71. Do not skip directory `statx` when `one_filesystem` is on.

**H148 / exp-154** is the standing Linux PGO screen: **accepted** as a number, not as a
shipped release profile.
Quiet `linux-v6.12` `cold-scan-index` −8.35% [−10.35%, −6.92%] and `warm-revalidate`
−8.15% [−8.64%, −7.07%]. Revalidate component is flat.
`[profile.release]` unchanged.
See [Linux PGO after #97](../specs/active/plan-2026-09-20-linux-pgo-screen.md).

Next free Linux experiment id is **exp-155**. Next free unused hypothesis id is
**H149**.

## Before the First Round

Do these once per session, in order.
Each one has caught a real mistake.

1. **Use a linked worktree, one branch, one PR.** Leave the primary checkout intact.

   ```shell
   git fetch origin
   git worktree add -b perf/campaign-$(date +%Y-%m-%d) \
     ../fdu-perf-$(date +%Y%m%d) origin/main
   ```

   If the next increment is stacked on an open performance PR, create that stacked
   branch from the current #91 head and keep one PR whose base is
   `perf/campaign-quiet-2026-09-18`, not `main`. Do not push further commits onto #91.
   One stacked pull request, updated after every experiment, never merged unattended.

2. **Find the queue.** Start from [Current Standing](#current-standing-2026-09-18), not
   from the `macos-agenda` label in isolation.
   Remaining order after the overnight is
   [the remaining-headroom block](../specs/active/plan-2026-09-19-post-h115-remaining-headroom.md).
   That label still holds older campaign-2 items; several have landed, and H86’s
   remaining gap is still the Linux floor after H111 failed on #94 (not this host).

   ```shell
   tbd show fdu-8ya1 fdu-rfr6 fdu-ytg5 fdu-jcfn fdu-rum0 fdu-vf4b fdu-i39y fdu-jekg
   tbd list --spec plan-2026-09-19-post-h115-remaining-headroom.md
   tbd list --label macos-agenda
   ```

   Read the bead before starting: its notes hold the recorded attempts and the blocker
   that may have moved.

3. **Audit for leftovers.** An earlier agent may have left a RAM disk or a worktree; an
   unexplained one is cleanup, not a shared cache, and the loop guide’s
   [temporary-volume section](performance-loop.md#temporary-volumes-on-macos) says how
   to resolve it.

   ```shell
   hdiutil info | grep -c "image-path" ; git worktree list
   ```

4. **Confirm the subjects are the ones on record.**

   ```shell
   make perf-subjects-check
   ```

   Nominated trees drift; the check says what moved.
   Drift in a deciding subject means re-observing it (`make perf-subjects`) and
   committing the new document with the first experiment, so the ledger’s fingerprints
   and the subject document agree.

5. **Build the control probe from the starting commit and copy it out of the tree.**

   ```shell
   make perf-probe-release
   mkdir -p /tmp/fdu-realtree && cp target/release/examples/perf_probe /tmp/fdu-realtree/perf_probe.control
   git rev-parse --short HEAD > /tmp/fdu-realtree/perf_probe.control.commit
   ```

   After an accepted experiment, the candidate becomes the next control: repeat this
   step. After a rejected one, the control is unchanged.

6. **Fingerprint each subject you will measure on.** The label must be the nominated
   label, because the artifact records it and the ledger groups by it.
   The paths live only in the gitignored nominations file, so read them from there
   rather than typing them anywhere a commit could pick them up:

   ```shell
   python3 -c 'import json;[print(s["label"],s["path"]) for s in json.load(open("explorations/benchmarks/subjects.local.json"))]' \
     | while read -r label path; do make perf-baseline PERF_TREE="${path/#\~/$HOME}" PERF_LABEL="$label"; done
   ```

## One Round

```
PICK → PREDICT → CHANGE → MEASURE → DECIDE → RECORD → COMMIT → RE-SCREEN
```

### PICK

Take the next item on the agenda.
Mark it and say so in the bead:

```shell
tbd update fdu-XXXX --status in_progress
```

### PREDICT

Before touching code, write down in the bead notes: the hypothesis id (an existing `HNN`
from the registry, or the next free number — the registry header says which), the tier
and the job that measures it, the subject, the metric and direction, and the regime.
If the change is expected to move a component rather than wall, say so now; a metric
chosen after the run is never an accept.

For a new hypothesis, add its row to the registry table in the loop guide in the same
commit as the artifact.

### CHANGE

The smallest diff that tests the one idea.
One idea per experiment; a second idea is a second round.
Run the unit tests that cover the code you touched before measuring, so the round does
not measure a bug:

```shell
cargo test --locked -p fdu-core
make perf-probe-release
```

### MEASURE

Measurement is the only step that needs the host to itself.
Nothing else may run: no builds, no other agent, no `make check`. Check first, and wait
rather than proceed:

```shell
uptime   # load average per core should be well under 1 before declaring quiet
```

Then run the paired comparison.
`JOBS` names the job the hypothesis predicts plus any job its mechanism could plausibly
move; `NAME` is the experiment id and a slug.

```shell
make perf-compare PERF_TREE=$HOME/.rustup PERF_LABEL=rustup-toolchains \
  CONTROL=/tmp/fdu-realtree/perf_probe.control \
  JOBS="default-tree cold-scan-index" TRIALS=12 \
  PERF_HOST_REGIME=quiet NAME=exp-067-skip-identical-snapshot-rewrite
```

What the flags mean, and what happens when they bite:

- `PERF_HOST_REGIME=quiet` makes the harness refuse to start if the host is over 25%
  busy, and invalidates any sample whose boundary observations exceed it.
  An invalidated sample is kept and counted, never replaced: if too many are invalid the
  round is inconclusive and is run again later, not topped up.
  Attempt quiet first.
  On this desktop a quiet cell often cannot hold (see
  [Current Standing](#current-standing-2026-09-18)): label `uncontrolled` and say so in
  the record. Do not lower the 25% bar so the cell passes.
  `uncontrolled` is exploration, not a held-out claim.
- `TRIALS=12` is the minimum for a verdict.
  A change predicted under 5% wants 16 or 20.
- The harness fingerprints the tree before and after.
  If it changed, the run exits nonzero and the numbers are not comparable: find what
  wrote to the subject, and run again.
- Content-tier hypotheses use `make perf-content-compare` with the same variables; its
  jobs are the content set.
- An aggregate-tier hypothesis (H72, H85) cannot name `aggregate-summary` alone.
  That job measures `fdu --view summary`, which reads `.gitignore` and so retains the
  index; the transient tier needs the probe’s `--no-controls` on both variants.
  `make perf-compare` cannot add it to the candidate, so run `measure` directly, as
  [the aggregate tier](performance-loop.md#the-aggregate-tier) shows.

Run on at least one deciding subject.
A screening subject (`cargo-registry-src`) is for checking that a job works, and its
numbers do not decide anything.

### DECIDE

The harness prints `ACCEPT` or `REJECT` per job from the arithmetic in the accept rule:
median at least 3% faster, the 95% interval entirely below zero, no sample invalidated.
Read it with the three checks the arithmetic cannot make:

- Was it the **predicted** job and metric?
  A win on a job the hypothesis did not name is a new hypothesis, not this one’s
  verdict.
- Is the **tail** acceptable?
  The run JSON records `p95_over_median` per arm and the ledger prints it beside the
  verdict once it reaches 1.5×; a median win with a worse tail is recorded as such.
- Is the **complexity worth it**? Write the one judgment sentence.

A `REJECT` is a result.
It is recorded exactly like an accept, and the code is reverted.

### RECORD

The artifact is lifted from the run JSON; the operator supplies only what a measurement
cannot know. Write the body first — what the profile or the bead suggested, what was
built (by commit and entry point), what the prediction got right and wrong — then
record:

```shell
PROV=$(python3 -c 'import json;print(next(s["provenance"] for s in json.load(open("docs/project/reports/nominated-subjects-darwin-arm64.json"))["subjects"] if s["label"]=="rustup-toolchains"))')
make perf-record ARGS="--run /tmp/fdu-realtree/results/run-exp-067-skip-identical-snapshot-rewrite.json \
  --id exp-067 --title 'Skip the identical snapshot rewrite on the cold-scan path' \
  --hypothesis H100 --control 'main at 778aa74' \
  --candidate 'byte-compare the encoded snapshot against the file before writing' \
  --decision accepted --primary-job default-tree \
  --reason 'one sentence: the number, the gate, the judgment' \
  --commit $(git rev-parse --short HEAD) --lines-changed 40 \
  --tree-provenance \"$PROV\" --body /tmp/fdu-realtree/exp-067-body.md"
make perf-ledger
make perf-report PREPARED=$(date +%Y-%m-%d)
make perf-test
```

`--tree-provenance` is required and has no default; the nominated-subjects document
holds each subject’s sentence, read into `PROV` above, and `--tree-reconstructible` is
added only when that document says `true` for the subject.
`ARGS` is re-parsed by the recipe’s shell, so keep titles and reasons free of
apostrophes, or double-quote them with the quotes escaped as `PROV` is.

`--commit` names the commit that **contains the change**, which is not the commit that
is checked out while recording: the schema means it as the place a reader goes to find
the code. So an accepted experiment lands in two commits — the change alone first, then
the artifact and the regenerated views naming its hash.
Recording before committing puts the *control’s* hash in the field, which points a
reader at the code without the change; that had happened to four artifacts before it was
caught. `--primary-metric` is added only when the hypothesis pre-registered a component.
The id is the next free `exp-NNN`; two agents in one night reserve ranges first, because
a collision is silent until `perf-ledger` fails.

### COMMIT

One commit per experiment, with the numbers in the message:

- **Accepted:** the code, the artifact, the regenerated ledger and evidence page, the
  registry row, and the bead update.
- **Rejected:** the artifact and the views only; the code is reverted first.

Then the gate and the push:

```shell
make check
tbd sync
git push -u origin HEAD
gh pr create --fill   # first experiment only; afterwards, gh pr edit to update the body
```

`make check` fails if the ledger or the evidence page does not match the artifacts,
which is the point: an experiment that is not published cannot merge.
It also takes about seven minutes and loads every core, so it runs *after* measurement,
never during.

### RE-SCREEN

Update the registry row’s status, close or update the bead with the verdict and the
experiment id, and check whether the change moved the next item’s headroom: two
hypotheses aimed at the same cost divide one budget, and this record has seen it three
times. If it did, say so in that bead before starting it.
Rewrite [Current Standing](#current-standing-2026-09-18) so the next-up table and
standing-best numbers match the ledger; a stale standing is how the next agent repeats a
finished experiment.

```shell
tbd close fdu-XXXX --reason "exp-067: accepted, default-tree -18.2% [-21.0%, -15.1%]"
tbd sync
```

## What an Unattended Agent Does Not Do

Each of these is either irreversible, a decision that belongs to a person, or a way of
producing a number that means nothing.

- Merge to `main`, or rebase or force-push the night’s branch.
- Change the accept rule, the bootstrap, the schema’s statistics, or a subject’s
  nomination. The set is re-observed (`make perf-subjects`) when it drifts, not edited.
- Start a person-gated item: the H86 rewrite (`fdu-xde5` is H111’s parent, not a license
  to restart the composite), the `searchfs` spike, the FSEvents journal, hardware
  CRC-32C (`unsafe` or a dependency), or `fdu-n75m` parts 2 and 3 (durability policy).
- Raise or lower the README 200K files/s or 4M cached lines/s from a probe cell.
- Retry H107 on a tree whose ignored share is not the walk.
- Land an H109 Path rewrite after exp-108.
- Retry the H114 type-id `String` alloc trim after exp-111.
- Retry H115 after exp-112, or restart the `fdu-jxhk` EntryId rewrite from that accept.
- Retry H113 file-count after H125 accepted the restore-count skip.
- Retry H129 after exp-128.
- Compile a leftover H130 cut after exp-129.
- Retry H131 after exp-130.
- Compile a leftover H132 cut after exp-131.
- Retry H133 after exp-132.
- Compile a leftover H134 cut after exp-133.
- Compile a leftover H135 cut after exp-134.
- Compile a leftover H136 cut after exp-135.
- Compile a leftover H137 cut after exp-136 other than the named share-one-walk.
- Retry H138 after exp-137.
- Retry H116 on an uncontrolled cell after exp-114.
- Retry H118 on an uncontrolled cell after exp-115.
- Retry H119 walk-overlap after the deciding-scale `content-basic` profile.
- Retry H124 type/size gate or a larger read chunk after exp-121.
- Load a snapshot on `fdu PATH` because H117 confirmed opened retention.
- Add a dependency, an `unsafe` block, or a platform gate without `make cross-lint`.
- Create a RAM disk, or write anything into a subject tree.
  Snapshots, results and scratch go under `/tmp/fdu-realtree/`.
- Record a verdict from a generated corpus, a screening subject, fewer than 12 trials, a
  run with invalidated samples, or a run whose fingerprint drifted.
- Regenerate a golden with `--update` without reading the diff; change what fdu prints
  without a reason the commit states.
- Measure while anything else is running, including `make check` or a second agent’s
  build.
- Continue past a failing `make check` by narrowing it; fix the failure or stop and say
  so in the PR.

When a step cannot proceed — the host never goes quiet, a subject keeps drifting, a
build fails for a reason outside the change — the right move is to record what was
observed in the bead and the PR body and move to the next agenda item, not to lower a
bar so the step passes.

## The Handoff

The pull request body is the night’s report and the morning’s reading.
After every experiment it carries a table — experiment id, hypothesis, subject, primary
job, change with interval, verdict — and a line for anything skipped and why.
A reader should learn the night’s result from the ledger diff and the PR body without
opening the transcript.
[Current Standing](#current-standing-2026-09-18) is the in-repo pickup for the next
agent; the PR body is not a substitute for updating it.

Before stopping:

```shell
make perf-subjects-check     # the subjects are still what the artifacts say
git status --short           # nothing uncommitted
hdiutil info | grep -c image-path   # no RAM disk left behind: expect 0
tbd sync
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
