# fdu Performance Evidence

**Revised:** 2026-09-30, after the 0.3.0 release standing, covering exp-000 through
exp-202: 200 artifacts, since exp-113 and exp-168–169 are unused ids.

**Status:** Current overview of the performance record.
The per-experiment numbers live in the artifacts; this report says what they add up to.

fdu’s speed work is done as a loop: one hypothesis, one change, a paired and interleaved
measurement against the code it came from, and a written accept rule.
Every turn leaves a validated artifact in [docs/project/experiments/](../experiments/),
kept whether it was accepted or rejected.
Three views are generated from those artifacts and never edited by hand:

- [The ledger](report-2026-08-10-fdu-performance-experiments.md), `make perf-ledger`:
  every experiment in full, with the regime-coverage table.
- [The charted page](performance-evidence/index.html), `make perf-report`: absolute
  milliseconds, every paired effect with its interval, cost per entry across subjects,
  and a per-platform section.
  It is one self-contained file that opens from disk.
- [timeline.json](performance-evidence/timeline.json): the projection the page is drawn
  from, committed so a reviewer can diff what the page claims.

This document is the hand-written layer over them: where each platform stands, what each
round asked and found, which results carry qualifications, and what is open.
The protocol is [the performance loop](../guides/performance-loop.md); the next action
is in [Current Pickup](../guides/performance-loop-runbook.md#current-pickup-2026-09-30);
the peer-tool rankings are in the tool comparisons
[on macOS](report-2026-09-26-fdu-live-tool-comparison.md) and
[on Linux](report-2026-09-27-fdu-linux-tool-comparison.md).

## What the Whole Record Shows

The record is 200 artifacts, exp-000 through exp-202, measured from 2026-08-10 to
2026-09-30: 111 accepted verdicts, 61 rejected, 17 baselines, 5 superseded, 4 in
progress and 2 blocked; 137 on macOS and 63 on Linux.
exp-113, exp-168 and exp-169 are unused ids.
[Every Round in Full](#every-round-in-full) places each artifact in the round that ran
it.

- **Linux, 4-vCPU virtualized guest, ext4, warm cache, quiet.** The 0.3.0 release
  engine’s default `fdu PATH` takes 48.00% [−50.45%, −44.79%] less time than 0.2.1’s on
  the Linux v6.12 source tree, 14.09% less on a directory-dense `node_modules` tree, and
  14.57% less on a generated million-entry tree.
  On all three it leads pdu’s default, `pdu --max-depth 2` and diskus, the narrowest
  lead being +10% [+2%, +12%] over `pdu --max-depth 2` on the kernel tree
  ([exp-202](../experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md)).
  Most of that is two rounds: the
  [overnight round](#the-linux-overnight-round-2026-09-29), −39.00% end to end on the
  kernel tree (exp-194), and the [pdu track](#the-pdu-track-2026-09-30), another −3.05%
  (exp-201).
- **Linux, the other tiers.** `content-cache-hit` on the kernel tree is about 588 ms
  (exp-138, exp-155). The index tier on the 450k generated tree was 1.78× the syscall
  floor against its 1.4× gate when last derived (exp-141), before the 0.2.1 and 0.3.0
  work.
- **macOS, one M1 Pro, APFS, warm cache, uncontrolled.** A pre-0.2.0 build (`a5c0ab46`)
  built its reusable index and a ten-row tree in 6.4 s on the generated million-entry
  tree, ahead of every peer, with dumac taking 9% longer
  ([macOS comparison](report-2026-09-26-fdu-live-tool-comparison.md), 2026-09-28). The
  ignore-aware default summary cut peak RSS 69.12% (exp-170); five restore changes took
  `content-cache-hit` from 1,218.0 to 778.0 ms (exp-108 to exp-132); H153’s −47.01% on
  `content-query` is provisional (exp-159). No macOS cell since 0.1.0 held the quiet
  gate.
- **Unmeasured this cycle.** No macOS cell has measured the 2026-09-29 round, the pdu
  track or the 0.3.0 engine: the last macOS record is exp-172, of 2026-09-28, so the
  release’s effect on macOS and its standing there against the peers are unknown.
  Windows has never been benchmarked.
  Nothing in the record is a cold-cache or a bare-metal Linux measurement.

## Where Each Platform Stands

A number is evidence about the platform, host, tree, and cache state it was measured on.
Every figure below names them.
**Quiet** means the host held the loop’s one-second CPU-busy gate (at most 25%) for the
whole cell (Linux cells before 2026-09-27 used load per core at most 0.25);
**uncontrolled** means it did not, so the cell is exploration rather than a claim.
All runs are warm-steady: dropping the page cache needs root, and nothing here describes
a cold disk.

### macOS

137 of the 200 experiments ran on one Apple M1 Pro (10 cores, 32 GiB), bare metal, APFS,
Darwin 25.5.0. No macOS cell since 0.1.0 is recorded as quiet: the desktop never held
the gate for a whole cell, so every recent macOS result is uncontrolled.

- **Installed command.** On the generated 1,000,001-entry tree, `fdu --cache off` built
  its reusable index and a ten-row tree in 6.4 s, ahead of every peer; dumac took 9%
  longer [+4%, +14%] and returns only a total
  ([macOS comparison](report-2026-09-26-fdu-live-tool-comparison.md), 2026-09-28,
  uncontrolled). The default `fdu PATH` cost the same within 0.1%.
- **Default summary.** Classifying `.gitignore` while counting, instead of building the
  full index (H161), cut peak RSS 69% on a 137k-entry source checkout and 58% on a
  77k-entry tree with no `.gitignore`, wall no worse
  ([exp-170](../experiments/exp-170-macos-ignore-aware-transient-summary-cuts-default-summary-pe.md),
  [exp-171](../experiments/exp-171-macos-ignore-aware-transient-summary-cuts-peak-rss-58-on-a-t.md)).
- **Content cache hit.** Five accepted restore changes (H115, H125, H129, H131, H133)
  took the `content-cache-hit` job on the ~146k-entry metabrowser checkout from 1,218 ms
  ([exp-108](../experiments/exp-108-deciding-scale-content-cache-hit-profile-on-metabrowser.md))
  to 778 ms
  ([exp-132](../experiments/exp-132-skip-unused-snapshot-path-reconstruction-on-metabrowser.md)).
  Those are five separate pairs, not one comparison; a sixth, H120, cut peak RSS 10%.
- **Multi-view reports.** Resolving each file once for every unfiltered metric view
  (H153) took a 100-report probe from 38.6 to 20.6 s wall, 29.9 to 12.0 s of component
  time: −47.0% paired
  ([exp-159](../experiments/exp-159-share-content-metric-resolution-across-views.md)).
  This is report construction after one scan, not scan speed, and it stays provisional:
  the major-fault gate was inconclusive and the 2026-09-28 quiet confirmation failed to
  qualify (20 of 24 samples invalidated).
- **Changes decided on Linux.** H156 and H160 moved macOS wall by less than the host
  could resolve; H160 cut peak RSS 26%
  ([exp-164](../experiments/exp-164-macos-one-shot-index-release-shows-no-wall-change-and-no-reg.md),
  [exp-165](../experiments/exp-165-macos-auto-cache-policy-cuts-default-tree-peak-rss-26-but-mi.md)).
  H159’s bounded form shows no wall or RSS change here
  ([exp-167](../experiments/exp-167-macos-h159-bounded-listing-recycle-is-rss-and-wall-neutral-l.md)).
  H162 and H163 have not been measured on macOS, and neither has any change from the
  2026-09-29 Linux round or the pdu track.

### Linux

Virtualized 4-core guests, Intel Xeon and ext4 where recorded: a KVM host on Linux
6.12.94+ for exp-138–155, and Firecracker guests on 6.18.x for the cells before and
after. Eight kernel builds, 63 experiments, counted in
[the ledger’s regime table](report-2026-08-10-fdu-performance-experiments.md#regime-coverage).
Most Linux cells since 2026-09-20 are quiet.

- **Installed command, generated tree.** On the same 1,000,001-entry tree,
  `fdu --no-gitignore --view summary` was the fastest tool measured at 0.94 s (pdu +8%,
  diskus +12%). Building the index took 1.25 s, about 23% longer than pdu and 21% longer
  than diskus ([Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md),
  2026-09-28, quiet, measured before H159, H161, H162, H163, and the 2026-09-29 round).
  On that round’s final head (`ebc06c78`) the default `fdu PATH` took 1.09 s: pdu with
  `--max-depth 2` took 2.5% less [−4.0%, −1.4%], and pdu’s default 4% and diskus 7% more
  (same comparison, 2026-09-29, quiet).
  On the 0.3.0 release engine it took 0.951 s and led `pdu --max-depth 2` by 18.8%
  [+15.0%, +19.7%], pdu’s default by 25.3% and diskus by 23.9%, at 58.5 MiB peak RSS
  ([exp-202](../experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md),
  2026-09-30, quiet). In the same run 0.2.1 was level with `pdu --max-depth 2`, where the
  2026-09-29 figures put pdu about 11% ahead of it, so about ten points of that lead are
  the session’s, by an estimate across sessions.
- **Default command, real source tree.** On Linux v6.12 (92,474 entries, 358
  `.gitignore` files) the 0.2.1 work took the default tree from 590 to 211 ms and the
  default summary from 505 to 167 ms
  ([exp-173](../experiments/exp-173-linux-h162-allocation-free-gitignore-matching-halves-the-def.md),
  [exp-174](../experiments/exp-174-linux-h163-per-listing-control-chains-cut-another-third-from.md),
  quiet probe, an earlier kernel build).
  The [2026-09-29 overnight round](#the-linux-overnight-round-2026-09-29) cut the
  default tree in paired steps of −29.6% (H171), −3.3% (H175), −13.5% (H172) and −7.6%
  (H183). In one paired cell against the 0.2.1 engine, run in a slower host regime than
  those steps, the default tree came out 39.00% faster (200.3 to 119.8 ms) and the
  default summary 26.25% faster
  ([exp-194](../experiments/exp-194-linux-the-overnight-round-end-to-end-the-default-tree-39-fas.md)).
  The [pdu track](#the-pdu-track-2026-09-30) cut the default tree another 3.05%
  [−5.81%, −1.03%] and the default summary 6.14% against that round’s final head
  ([exp-201](../experiments/exp-201-linux-the-pdu-track-end-to-end-the-default-tree-3-and-9-fast.md)).
  In the same run fdu’s default command led pdu’s default by 13%, pdu `--max-depth 2` by
  3% [+1%, +8%], and diskus by 12%, from 2.4 times pdu’s default at the start of the
  overnight round; about six points of the lead over pdu’s default are that night’s host
  regime. The 0.3.0 release engine, measured directly against 0.2.1’s, takes 48.00% less
  time for the default tree (208.6 to 109.9 ms) and 34.63% less for the default summary
  ([exp-202](../experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md));
  in that run, a session about 30% slower than exp-201’s for every tool, it led pdu’s
  default by 15%, pdu `--max-depth 2` by 10% [+2%, +12%], and diskus by 17%.
- **Directory-dense trees.** Returning drained listings to the walker that allocated
  them (H159) cut the default tree 8.6% on a real 80k-entry `node_modules` tree
  ([exp-190](../experiments/exp-190-linux-h159-listing-recycle-clears-3-percent-on-a-real-direct.md),
  quiet) and 10.6% on the generated tree.
  The exact transient tree tier (H172) cut it another 10.3%
  ([exp-181](../experiments/exp-181-linux-h172-transient-tree-tier-cuts-the-default-tree-10-on-n.md)),
  and H185 with H186 another 8.94% [−12.29%, −5.04%] (exp-201). In that run fdu’s
  default command led pdu’s default by 15%, pdu `--max-depth 2` by 10% [+4%, +13%], and
  diskus by 11%. The release engine takes 14.09% less time than 0.2.1’s for the default
  tree there, and in exp-202’s run led pdu’s default by 18%, pdu `--max-depth 2` by 12%
  [+2%, +19%], and diskus by 12%.
- **Content.** `content-cache-hit` on Linux v6.12 is about 588 ms, 22.5% below the #91
  control
  ([exp-138](../experiments/exp-138-linux-cache-hit-stack-same-versus-91-control.md),
  [exp-155](../experiments/exp-155-linux-cache-hit-restore-mix-after-leftover-apply-timer-expan.md),
  quiet). H153 has not been measured on Linux (H154).
- **Floor.** On the 450k-entry generated tree the index tier is 1.78× the parallel
  syscall floor against the 1.4× gate, and peak RSS 5.20× `arena_spike` against 3×
  ([exp-141](../experiments/exp-141-h111-linux-floor-and-rss-gates-fail-on-current-engine.md),
  quiet requested, recorded uncontrolled).

### What Ships Where

0.2.0 (2026-09-28) carries H153, H156 (a large one-shot index is freed after the
answer), and H160 (`--cache auto` writes no snapshot for a one-shot metadata report).
0.2.1 (tagged 2026-09-29 on `c1644575`, published to crates.io and PyPI) adds H161 (the
default summary classifies ignore rules without an index), H159 (listings recycled to
the walker that allocated them, [#150](https://github.com/jlevy/fdu/pull/150)), and H162
and H163 (allocation-free matching and per-directory control chains,
[#155](https://github.com/jlevy/fdu/pull/155)). The generated-tree peer comparisons have
not been re-measured on 0.2.1; the real-tree tool cells of 2026-09-29 start from its
engine (exp-175, exp-176).

0.3.0 carries the 2026-09-29 accepts ([#161](https://github.com/jlevy/fdu/pull/161)) and
the pdu track’s ([#163](https://github.com/jlevy/fdu/pull/163)). Each landed as a change
commit followed by its record commit:

- H171, bucketed `.gitignore` matching;
- H175, control chains derived from the parent’s;
- H172 with H176 and F6e, the exact transient tree tier;
- H180, the summary route’s walker trims;
- H169 phase 1, a Linux-native directory reader with four audited `unsafe` expressions,
  built on glibc only;
- H183, cheap pre-checks for the residual `.gitignore` rules;
- H185, the folded tree route’s directory and symlink kinds taken from the listing once
  one stat in it has proved the directory searchable;
- H188 with H189, byte-wise paths in the summary fold and control reads sized from their
  length;
- H186, tree rows admitted before they are built and a folded index released on a
  detached thread.

None changes an answer or `IGNORE_RULES_VERSION`, and none adds a dependency.
Beside them, H184 puts every route on the native reader so that no stat of a listed
child triggers an automount, a correctness change screened for non-regression (exp-196).
Of these, only H171 changes the public API: `counters::Counts` gains three public
fields, `ignore_patterns_tested`, `ignore_bucket_probes`, and `ignore_bucket_hits`, for
its `FDU_COUNTERS=1` rows, which is semver-breaking, so the round ships in 0.3.0. The
reader’s public diagnostics fields (`fdu-q7hf`) are in
[#164](https://github.com/jlevy/fdu/pull/164), also for 0.3.0.

## Standing Results by Tier

The latest figure each tier holds, from the arm that stayed in the product.
Absolute values compare only within one row: each is its own subject on its own host.

| Tier and job | Platform | Subject | Latest | Regime | Source |
| --- | --- | --- | --- | --- | --- |
| CLI indexed tree, `--cache off` | macOS | generated, 1.0M entries | 6.4 s; dumac +9% | uncontrolled | [macOS comparison](report-2026-09-26-fdu-live-tool-comparison.md) |
| CLI indexed tree, `--cache off` | Linux | generated, 1.0M | 1.25 s; pdu 1.02 s | quiet | [Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md) |
| CLI summary, `--no-gitignore` | Linux | generated, 1.0M | 0.94 s; fastest measured | quiet | [Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md) |
| CLI default, against pdu and diskus | Linux | `linux-v6.12`, 92k | 0.110 s; pdu’s default +15% [+12%, +19%], pdu `--max-depth 2` +10% [+2%, +12%], diskus +17% [+3%, +20%]; from 2.4× pdu’s default | quiet | exp-175, exp-202 |
| CLI default, against pdu and diskus | Linux | `node-modules-dense`, 80k | 0.106 s; pdu’s default +18% [+15%, +23%], pdu `--max-depth 2` +12% [+2%, +19%], diskus +12% [+9%, +17%]; from 1.12× pdu’s default | quiet | exp-176, exp-202 |
| CLI default, against pdu and diskus | Linux | generated, 1.0M | 0.951 s; pdu `--max-depth 2` +19% [+15%, +20%], pdu’s default +25% [+21%, +28%], diskus +24% [+20%, +28%]; from pdu `--max-depth 2` 2.5% ahead on `ebc06c78` | quiet | [Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md), exp-202 |
| Default tree, `.gitignore` on | Linux | `linux-v6.12`, 92k | 109.9 ms in exp-202’s slower session, 85.2 ms in exp-201’s; paired −29.6% (H171), −3.3% (H175), −13.5% (H172), −7.6% (H183); end to end −39.00% from 200.3 ms at Q0, then −3.05% (H185 and H186); the release against 0.2.1 −48.00% | quiet | exp-178–180, exp-193, exp-194, exp-201, exp-202 |
| Default summary, `.gitignore` on | Linux | `linux-v6.12`, 92k | 128.1 ms in exp-202’s session, 84.6 ms in exp-201’s; paired −25.5% (H171), −5.6% (H180), −9.4% (H169), −6.15% (H188 with H189); end to end −26.25% from 156.5 ms at Q0, then −6.14%; the release against 0.2.1 −34.63% | quiet | exp-178, exp-184, exp-186, exp-194, exp-198, exp-201, exp-202 |
| Default summary, peak RSS | macOS | metabrowser, 137k | 11.0 MiB, from 35.9 | uncontrolled | exp-170 |
| Default tree | Linux | `node-modules-dense`, 80k | 115.6 ms in exp-202’s session, 79.5 ms in exp-201’s; paired −10.3% (H172), −4.3% (H169), −3.55% (H185), −4.91% (H186); end to end −9.75% from 126.9 ms at Q0, then −8.94% (H185 and H186); the release against 0.2.1 −14.09% | quiet | exp-181, exp-185, exp-195, exp-197, exp-199, exp-201, exp-202 |
| Default tree, peak RSS | Linux | generated, 1.0M | 57.3 MiB, from 292.7 MiB on 0.2.1 (−80%); exp-180’s screen 64 MB, from 306 | quiet | exp-180, exp-202 |
| `content-cache-hit` | macOS | metabrowser, ~146k | 778 ms, from 1,218 | uncontrolled | exp-108 to exp-132 |
| `content-cache-hit` | Linux | `linux-v6.12`, 92k | ~588 ms | quiet | exp-155 |
| `content-query`, 100 reports | macOS | metabrowser, 137k | 20.6 s, from 38.6; provisional | uncontrolled | exp-159 |
| Index tier ×floor | Linux | generated, 450k | 1.78× (gate 1.4×) | uncontrolled | exp-141 |
| Cold scan, campaign 1 | macOS | metabrowser, 60k | 290 ms, from 635 | not recorded | [exp-032](../experiments/exp-032-cumulative-effect-through-bounded-parallel-reconciliation.md) |

The campaign-1 row is the last time the original pre-work binary was re-measured against
the code of the day; no later checkpoint repeats that comparison, so it does not
describe the current engine.

The Linux default tree and summary rows give the release engine in its end-to-end cell
against 0.2.1 (exp-202) and the pdu track’s shipped arm in its cell against the
overnight round’s final head (exp-201), the paired steps that produced them, the round’s
paired change against the Q0 engine (exp-194 on `linux-v6.12`, exp-195 on
`node-modules-dense`), and exp-201’s own; the rows against the peers come from exp-202’s
tool cells, with the Q0 ratio from exp-175 and exp-176. exp-202 ran in a session about
30% slower than exp-201’s for every tool on the real trees.
exp-194 and exp-195 ran in the host’s slower, kernel-heavy regime, and exp-201 in a
faster one that also favoured fdu against pdu’s default by about six points.
Absolute levels on that host drifted by up to about 50–70% between cells over the night
(the Q0 engine 84.0 ms in exp-176, 126.9 ms in exp-195; pdu’s default 0.072 s after
H169, 0.121 s at the final cell), so a row’s absolute values need not agree with its
paired steps; the paired figures are the claims.

## The Loops in Order

Each row is one line of inquiry, and together they hold every artifact from exp-000 to
exp-202; [Every Round in Full](#every-round-in-full) expands each row.
The ledger has every verdict and
[the loop history](report-2026-08-14-performance-campaign-status.md) how the method
changed from phase to phase.

| Loop | Experiments | Platform | What it found |
| --- | --- | --- | --- |
| Campaign 1: walker, snapshot, revalidation | exp-000–039 | macOS | Parallel walk, `getattrlistbulk`, bounded reconciliation: cold scan −54.5% and warm revalidate −52.0% paired at the exp-032 checkpoint |
| Summary tier and openers | exp-040–046 | macOS | An exact summary without an index: −14.6% wall, −95% RSS ([exp-040](../experiments/exp-040-derive-an-exact-rich-summary-without-building-an-index.md)); H62–H65 refuted; shared openers (H70) still open |
| Content analysis | exp-047–050 | macOS | In-place UTF-8 decoding −12.0%; three rejections |
| Linux campaign 1 | exp-051–053, exp-060–065 | Linux | Consumer and snapshot cuts of 5–12% on generated trees; per-layer counters at no measurable cost; content roll-up −30.3% generated and −25.8% real ([exp-065](../experiments/exp-065-validate-the-content-roll-up-change-on-a-dense-real-tree.md)) |
| Transfer and scheduling | exp-054–059 | macOS | Linux work transferred (−15.7% warm revalidate); every alternative adaptive-worker controller regressed 36–61% |
| Campaign 2: default command, content map | exp-066–070, exp-104 | both | Skip the identical snapshot rewrite −10.6%; path-byte file map −31.0% on `content-cache-hit` |
| Streaming parity, detached scanner | exp-071–090 | macOS | A #51 regression halved, then opened-discovery and mutation cuts of 3–10%; most scanner reshapes rejected |
| H86 structural composite | exp-091–103 | both | Darwin composite landed (−33.6% cold scan with controls, −7.7% default tree); the Linux floor stage failed at 2.60× the floor against 1.4× ([exp-103](../experiments/exp-103-h86-linux-evidence-stage-relative-gates-pass-floor-gates-fai.md)) |
| Post-0.1.0 Darwin revisit | exp-105–137 | macOS | Six cache-hit restore accepts; one shared walk for unfiltered views −18.8% ([exp-137](../experiments/exp-137-share-one-every-entry-across-unfiltered-metric-views.md)) |
| Linux validation and iteration | exp-138–155 | Linux | Darwin wins transfer; leftovers read as the walk and I/O; H111 failed; batch recycle −5.0%; `d_type` skip −9.0% on `/usr`; PGO screen −8.4%, not adopted |
| Progress handle | exp-156–157 | macOS | No cost without a handle; the attached case is open (H151) |
| Multi-view `content-query` | exp-158–159 | macOS | Exact report oracle, then H153’s −47.0%, provisional |
| Linux tool comparison | exp-160–163 | Linux | H156 −3.2% and H160 −13.8% on the default tree; H157 and H158 rejected |
| macOS rerun of stack 141 | exp-164–165 | macOS | H156 and H160 wall unresolvable; RSS −26% |
| H159 listing recycle | exp-166–167, exp-188–190 | both | Neutral on macOS; per-directory saving, so absent on the sparse kernel tree and −8.6% on a dense one |
| H161 ignore-aware summary | exp-170–172, exp-187 | both | macOS RSS −69% and −58%; Linux wall −6.9% |
| pdu on a real tree | exp-173–174, exp-191 | Linux | `.gitignore` classification was the Linux default-command gap: H162 −47.0%, H163 −36.4%; H157 rejected again |
| Linux overnight loop | exp-175–186, exp-192–195 | Linux | Two regimes of wall time; H171 −29.6%, H172 −13.5% and H183 −7.6% on the default tree, H180 and H169 on the summary; walker count and H181 with H182 rejected; the default tree −39.00% end to end (exp-194); 2.4× pdu’s default to level |
| pdu track | exp-197–201 | Linux | H185 −3.55% and H186 −4.91% on the dense tree’s default tree, H188 with H189 −6.15% on the kernel tree’s summary; H187 rejected, the tree route’s consumer having slack; the default tree −3.05% and −8.94% end to end, ahead of both pdu modes and diskus on both real trees (exp-201) |
| Review rounds and the automount change | exp-196 | Linux | #161’s review put every route on the native reader (H184), a correctness change screened for non-regression: `warm-revalidate` −2.74%, the serial walk −5.14%; #163’s review made H185 prove a directory searchable before trusting its listing, keeping 83% and 89% of its `statx` saving |
| 0.3.0 release standing | exp-202 | Linux | The release engine against 0.2.1: the default tree −48.00%, −14.09% and −14.57% on the kernel, dense and generated trees; ahead of both pdu modes and diskus on all three, the narrowest lead +10% [+2%, +12%] over `pdu --max-depth 2` on the kernel tree |

## Every Round in Full

One entry per row of the table above, in the same order.
Each gives the question the round asked, its verdicts with the paired change the record
states on its primary job (wall time unless another metric is named), and where it left
the product. The ledger has every other job and metric.
Rounds from 2026-09-27 on also have dated sections below with their tables.

### Campaign 1: Walker, Snapshot, Revalidation

exp-000 to exp-039. macOS, 2026-08-10 to 2026-08-13, on a 60k-entry metabrowser
checkout, a 720,805-entry cache-pressure tree, and a 1.0M-entry live workspace.

**Question.** Where do a cold scan and a warm open spend their time on a real tree, and
which single changes cut that by at least 3%? exp-000 set the baseline on the
59,654-entry checkout.

**Accepted.**

- Walker and scheduling: a bounded parallel producer, H1, −50.03% `cold-scan-index`
  (exp-001); breadth-first order, H48, −0.58%, kept for its monotone partial results
  rather than for speed (exp-012); region-scheduled breadth-first, H49, peak RSS −3.77%
  (exp-013); adaptive worker depth under metadata-cache pressure, H31, first as sixteen
  workers, −11.72% (exp-015), then calibrated from the first 16k entries’ service time,
  −5.31% (exp-021), both on the 720k tree.
- macOS bulk metadata: `getattrlistbulk`, H3 with H26, −30.13% on the 720k tree
  (exp-022), and the same reader during full reconciliation, H53, −34.39%
  `warm-revalidate` (exp-026).
- Index, snapshot, and reconciliation: borrowed path components, H5, −9.40%
  `warm-revalidate` (exp-004); snapshot load through the parent, H10, −18.60%
  `warm-snapshot-load` (exp-005); direct reconcile off entry ids, H14, −7.09% (exp-007);
  extensions interned to integer ids, H18, −15.65% (exp-008); a single-pass checksum and
  parse, H32, −12.38% on the pre-registered load component (exp-009); bounded parallel
  reconciliation waves, H12 with H9, −59.53% `warm-revalidate` on the 720k tree
  (exp-030).
- Checkpoints against the pre-work binary `b565882`, on `cold-scan-index`: −48.91%
  (exp-006), −53.49% (exp-023), −52.84% (exp-027), and −54.53% (exp-032). After the
  composable command line merged, PR #8 against `main`: −42.26% `warm-revalidate`
  (exp-033), −30.46% on the 720k tree (exp-034), and −31.35% on the 1.0M workspace
  (exp-035).

**Rejected.** Parallel revalidation, H9, −2.59% (exp-002); skipping bootstrap
journalling, H8, +1.02% (exp-003); claim-list reconcile, H17, −0.03% (exp-010); one
ancestor merge per insert run, H13, −2.53% (exp-011); moved producer paths, H51, −0.44%
(exp-016); dormant reserve workers, H31, +2.01% `cold-scan-producer` (exp-017); a 100k
worker trigger, +1.23% (exp-019), and a 262k one, −1.71% (exp-020); root-relative opens,
H2 with H24, −0.07% (exp-024); sixteen workers after bulk metadata, H52, +19.19%
(exp-025); reused bulk staging, H54, +0.21% (exp-028); a 256 KiB bulk buffer, H55,
−1.80% (exp-029) and +2.22% on the 1.0M workspace (exp-039); 4096-directory
reconciliation waves, H56, +1.64% (exp-031); fixed worker counts on the 1.0M workspace,
H57, −1.30% for eight (exp-036); depth-first order, H4, +3.57% (exp-037); a
parent-relative `openat` frontier, H24 with H29, −0.69% (exp-038).

**Other verdicts.** exp-014, a baseline for H50, measured breadth-first against
depth-first on the shipped scheduler: −3.04% `cold-scan-producer`. exp-018 (H31, reserve
workers spawned after 100k entries, −4.04%) was superseded by exp-021.

**Standing after.** exp-032, one interleaved run against the original binary: cold index
54.53%, producer 60.05%, snapshot save 52.41%, warm revalidation 51.99%, and snapshot
load 35.66% faster; the cold scan of the checkout went from 635.4 to 289.6 ms.

### Summary Tier and Openers

exp-040 to exp-046. macOS, 2026-08-13, on 0.72M- to 0.98M-entry trees.

**Question.** Can the summary be answered exactly without building the reusable index,
and does anything below the walk (reduction inside workers, a derived macOS record,
directory-opener pools) beat it?

**Accepted.** An exact summary derived from the scan without an index, H59: −14.56%
`rich-summary-report` and peak RSS −95.28% (exp-040).

**Rejected.** Worker-local reduction, H62, −1.38% (exp-041); a derived macOS bulk
record, H63, +1.86% (exp-042); eight workers for the summary, H65, +0.67% (exp-043); a
specialized selected total, H64, −1.15%, which did not beat dumac (exp-044).

**Other verdicts.** Pipelined directory opens, H67 with H69, had an unusable interval,
[−31.04%, +33.91%], and were superseded (exp-045); a shared two-thread opener pool, H70,
cleared a short screen at −3.98% and is in progress, awaiting a quiet confirmation
(exp-046).

**Standing after.** The summary retains no index.
None of H62 to H65 moved wall by 2%, although worker-local reduction alone cut user CPU
36.23% and RSS 34.77% (exp-041): what remains is directory opens and kernel work.
H70 is still open.

### Content Analysis

exp-047 to exp-050. macOS, 2026-08-13, on the fdu checkout (307 entries) and a generated
2,000-file Markdown corpus.

**Question.** Where can content analysis (lines, SLOC, prose metrics) shed work?

**Accepted.** Decoding complete UTF-8 chunks in place, H82: −12.04% `markdown-prose`
(exp-050).

**Rejected.** Inline analysis of small trees, H79, +66.34% `content-basic`: the worker
pool was doing useful parallel I/O (exp-047); prose-collector gating for code, H80,
+1.50% `code-sloc` (exp-048); a bounded Markdown buffer reserve, H81, −3.55% with an
interval crossing zero (exp-049).

### Linux Campaign 1

exp-051 to exp-053 and exp-060 to exp-065. Linux, 4-vCPU Firecracker guests on
6.18.5-fc-v20 and 6.18.44-fc-v21, 2026-08-14 to 2026-08-23, on the generated
450,463-entry trees `meta450k` and `vm450k`, then two content subjects.

**Question.** Where `getdents64` and `statx` are the walk, do the index consumer and the
snapshot path yield the same kind of cuts, and what do per-layer counters cost?

**Accepted.** A one-slot parent memo, S1, −7.35% `cold-scan-index` (exp-051); per-layer
counters at +0.03%, which bounds their cost below about 3.3% (exp-052), and a runtime
toggle at −1.26% (exp-053); CRC-32C slicing-by-8, H88, −12.20% on the pre-registered
`cold-snapshot-save` component (exp-061); skipping unread journal capture on bootstrap,
H90, −5.06% (exp-062); sharing the index with the snapshot writer, H87, −10.50%
`cold-open-save` with peak RSS −35.26% (exp-063); hashed content roll-ups and indexed
type-rule tiers, H94 with H95, −30.31% `content-cache-hit` on a generated subject
(exp-064) and −25.78% on a dense real one (exp-065).

**Rejected.** A one-slot extension memo, H89, +1.59% (exp-060).

**Standing after.** Every accept but exp-065 rested on a generated tree, and exp-065
showed why that matters: the warm content cut transferred, while the cold half read
−2.38% on the dense tree against exp-064’s −13.40% on its sparse generated subject.

### Transfer and Scheduling

exp-054 to exp-059. macOS, 2026-08-14 to 2026-08-15.

**Question.** Does the Linux campaign transfer to macOS, and can an adaptive-worker
controller do better than the shipped one-shot controller on APFS?

**Accepted.** The Linux campaign on macOS: `warm-revalidate` −15.68%, cold scan +1.39%
and neutral (exp-054); its review fixes, −0.95%, no regression (exp-055); bounded scan
diagnostics, H97, −0.55%, inside the +3% margin (exp-056).

**Rejected.** On the adversarial `adaptive-fast-slow-100k` tree: repeated adaptive
windows, H98, +58.49% (exp-057); staged expansion, H99, +60.73% (exp-058); eight fixed
workers, H96, +35.55% (exp-059).

**Standing after.** The shipped controller stayed;
[the gap-closure report](report-2026-08-15-adaptive-worker-gap-closure.md) has the
policy analysis.

### Campaign 2: Default Command and Content Map

exp-066 to exp-070, macOS, 2026-08-23 to 2026-08-24, and exp-104, Linux, 2026-09-14.
Campaign 2 is
[the floor-anchored plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md).

**Question.** What does the default command, `fdu PATH`, cost end to end, which no
record had measured, and what does a warm content open spend?

**Baseline.** exp-066, on a 175k-entry rustup store: every repeated run rewrote a 13.9
MB snapshot it never read.

**Accepted.** Skipping the identical snapshot rewrite, H100, −10.61% `default-tree`
(exp-067); flushing the report before joining the snapshot writer, H101: time to first
byte −7.54% on a repeated run and −12.47% on a first run, total wall +1.24% and
unchanged (exp-068); a byte-ordered content file map, H102, −31.00% `content-cache-hit`
(exp-069); its separator fixes, −1.28%, non-inferior (exp-070).

**Rejected.** A byte-hashed content roll-up map on Linux, H103, +0.05%: instructions
fell and wall did not (exp-104).

### Streaming Parity and the Detached Scanner

exp-071 to exp-090. macOS, 2026-09-01, on the rustup store, an 11k-entry cargo registry
cache, and a 114k-entry metabrowser checkout.
H91 to H106 here reuse earlier ids; the loop guide lists the
[numbering collisions](../guides/performance-loop.md#numbering-collisions).

**Question.** PR #51’s streaming engine was slower than the pre-rewrite `main`; what
closes the gap without changing an answer?

**Blocked and baselines.** PR #51’s head halved its base’s cost, −49.55%
`cold-scan-index` (exp-071, accepted as mechanism), and stayed at +144.46% against the
pre-rewrite control (exp-073, blocked).
Removing the path-keyed ancestry overlay cut the engine component 66.65% and could not
ship without an equivalent proof (exp-072, blocked).
On the registry cache the residual was +7.68% `default-tree` (exp-074); scoped counters
measured +1.93% (exp-075), the correctness fixes +0.29% (exp-076), and scanner phase
counters, H103, −0.12% (exp-086).

**Accepted.** Detached consequences selected once per batch, H91, −6.57% `default-tree`
(exp-077); the eager compatibility projection removed, H92, −1.55% `delta-apply-large`
with peak RSS −7.09% (exp-078); scanner parents resolved before mutation, H93, −9.50%
`opened-discovery` (exp-079); oversized journal clones skipped, H94, −3.46%
`delta-apply-large` (exp-080); FullIndex scan diagnostics, H97, −3.48%, inside the +3%
overhead ceiling (exp-090).

**Rejected.** Borrowed impact paths, H95, −1.07% (exp-081); journal-owned scanner
commits, H96, −0.01% (exp-082); skipping unignored roll-up maintenance, H97, −1.61%
(exp-083); compact optional partitions, H98, −2.63% (exp-084), and with compact batches,
H99, −2.56% (exp-085); fused preparation and reduction, H104, −1.11% (exp-087);
coalesced causal fragments, H105, +0.13% (exp-088); suppressed causal publication, H106,
+0.68% on the producer component (exp-089).

**Standing after.** With exp-079 the stack matched or beat the pre-rewrite control; the
consumer representation was left to the H86 composite.

### The H86 Structural Composite

exp-091 to exp-103. macOS, 2026-09-01 to 2026-09-07, on the 113,794-entry checkout
(97,587 entries for exp-102), and a Linux evidence stage on 2026-09-02.

**Question.** Does a new consumer representation, H86, run as one structural composite
under a differential oracle, reach the floor targets the campaign-2 plan pre-registered?

**Accepted.** Five non-inferior steps: a shared walker, +1.13% (exp-092); transient
hashed parents, +0.83% (exp-093); borrowed roll-ups, +0.19% (exp-094); moved names,
−0.31% with allocated bytes −24% (exp-095); monomorphized consumption, +0.16% (exp-099).
Two speed-ups: fixed controls applied once per detached directory, −33.55%
`cold-scan-index` with controls on (exp-096), and compact child topology, −7.70%
`default-tree` with RSS −37.79% (exp-101). And a point lookup for public mutation
preflight, −49.78% `delta-apply-large` (exp-102).

**Rejected.** A dynamically dispatched consumer, +0.82% (exp-098); out-of-line directory
state, −0.83% despite RSS −24.63% (exp-100). The Linux evidence stage passed its
relative gates, −31.70% `default-tree`, and failed its floor gates: 2.60× the syscall
floor against 1.4×, and peak RSS 6.59× `arena_spike` against 3× (exp-103, which keeps
neither arm, since the candidate stays in the stack).

**Other verdicts.** The first pipelined builder, +2.48%, was superseded (exp-091); the
lifecycle-parity audit is in progress on a 17–22% retained-memory gap (exp-097).

**Standing after.** The composite landed on macOS. On Linux the index tier stayed far
above its floor gate, the question exp-141 took up.

### The Post-0.1.0 Darwin Revisit

exp-105 to exp-137; exp-113 is unused.
macOS, 2026-09-19, uncontrolled, on the ~146k-entry metabrowser checkout, the 77k-entry
rustup store, and the 159k-entry system frameworks.

**Question.** On the engine that shipped in 0.1.0, what is left in the default command
and in a warm content open?

**Accepted changes.** Six restore cuts on `content-cache-hit`: a bottom-up roll-up after
restore, H115, −9.69% (exp-112); a streamed sidecar parse, H120, peak RSS −10.13%
(exp-117); completeness from the restore count, H125, −8.03% (exp-124); restore without
classify, H129, −13.11% (exp-128); a restore walk that joins the parent’s path, H131,
−4.07% (exp-130); unused snapshot paths skipped, H133, −6.37% (exp-132). One shared walk
for the unfiltered metric views, H138, −18.76% `content-query` (exp-137).

**Accepted determinations.** Sixteen profiles and checks that changed no engine code,
each naming what remained: the installed command’s second `fdu PATH` still cold-scans
(H108, exp-107); an opened root’s second report takes 1.6 ms against 2,612 ms for a
one-shot (H117, exp-116), and the index’s own second report 1.7 ms against 2,078 ms
(H123, exp-119); the metadata walk is 97% of the default tree’s component on the
frameworks and 93% on the checkout, its leftover `open` and `getattrlistbulk` (H122,
exp-118 and exp-122; H128, exp-127); the restore mix after each cut (H121, exp-120;
H113, exp-123; H126, exp-125; H130, exp-129; H132, exp-131; H134, exp-133); opened
discovery against the first-pass walk (H127, exp-126); first-pass content is file I/O
(H135, exp-134); the first-run default tree is still the walk (H136, exp-135); and
`content-query` repeats one walk per view (H137, exp-136), which H138 then shared.

**Rejected.** Reading `.gitignore` by default, H107, +1.64% on the checkout (exp-106);
completeness by file count, H113, −7.59% with an interval crossing zero (exp-110); a
type-id `get_mut`, H114, −0.56% (exp-111); restore lookup without the candidates map,
H116, +8.70% (exp-114); insert-then-rebuild on first analysis, H118, −2.60% component
(exp-115); a first-pass I/O gate, H124, where every admitted open is needed (exp-121).

**Baselines.** The rustup store’s default tree at 149.8 ms (exp-105), the
`content-cache-hit` profile at 1,218.0 ms (H109, exp-108), and restore stage timers
(H112, exp-109), kept at +0.31%.

**Standing after.** `content-cache-hit` on the checkout went from 1,218.0 ms (exp-108)
to 778.0 ms (exp-132), over five separate pairs rather than one comparison.

### Linux Validation and Iteration

exp-138 to exp-155. Linux, a 4-vCPU KVM guest on 6.12.94+, 2026-09-20 to 2026-09-21,
mostly quiet, on `linux-v6.12`, the generated `linux-450k`, and a 208k-entry `/usr`.

**Question.** Do the Darwin wins transfer to Linux, what is the Linux leftover on each
job, and does the engine pass the floor gates (H111)?

**Accepted.** The Darwin cache-hit stack on Linux, H139, −22.48% `content-cache-hit`
(exp-138), and the shared view walk, H141, −17.60% `content-query` (exp-140); drained
batches returned to the walker that produced them, H147, −4.98% on the `--no-controls`
summary (exp-151); directory and symlink `statx` skipped by `d_type`, H72, −9.01% on
symlink-heavy `/usr` (exp-153); a PGO screen, H148, −8.35% `cold-scan-index`, accepted
with the control kept, since the profile is host-specific and was not adopted (exp-154).
Seven leftover determinations found no new userspace cut: the default tree (H140,
exp-139), the index tier (H143, exp-142), and the first run (H146, exp-147) read as the
`getdents64` and `statx` walk; first-pass content is file I/O (H142, exp-143); the
cache-hit and opened-discovery leftovers match Darwin’s (H144, exp-144; H145, exp-145;
H149, exp-155).

**Rejected.** H111’s floor gates: the index tier 1.78× the floor against 1.4×, peak RSS
5.20× `arena_spike` against 3×, and the aggregate tier 1.59× and 1.86× on the nominated
real trees against 1.25× (exp-141); the batch recycle against H85’s 20% mimalloc bar,
−4.98% (exp-150, the run exp-151 accepted at 3%); the `d_type` skip on `linux-v6.12`,
−1.63% (exp-152).

**Worker count (H84), recorded with neither arm kept.** Eight walkers are no 3% win on
the named jobs, +1.75% on the summary (exp-146); with `.gitignore` off they gain 10.06%
on `/usr` (exp-148), and with it on, the default, they regress 7.12% (exp-149).

**Standing after.** `content-cache-hit` on `linux-v6.12` about 588 ms (exp-138,
exp-155). The determinations read the default command as the walk; exp-173 later showed
that most of it was `.gitignore` classification running during the walk.

### Progress Handle

exp-156 to exp-157. macOS, 2026-09-24, uncontrolled, on the frameworks tree.

**Question.** Does the progress indicator cost anything, with and without a handle
attached?

**Accepted.** Without a handle, H150, −1.78%, non-inferior at +3% (exp-156).

**In progress.** With a handle polled every 80 ms, H151, +5.75% [−5.34%, +10.91%]:
neither non-inferior nor a cost on a busy host (exp-157).

### Multi-View `content-query`

exp-158 to exp-159. macOS, 2026-09-27, uncontrolled, on the 137k-entry checkout.

**Question.** Can one pass resolve each file’s metrics for every unfiltered view?

**Accepted.** An exact multi-view report oracle, H152, same binary in both arms
(exp-158).

**In progress.** One-pass shared metric resolution, H153, −47.01% `content-query`, 38.6
to 20.6 s over 100 reports, retained provisionally: its major-fault gate is inconclusive
and the quiet confirmation of 2026-09-28 failed to qualify (exp-159).

### Linux Tool Comparison

exp-160 to exp-163. Linux, a 4-vCPU Firecracker guest on 6.18.44-fc-v37, 2026-09-27,
quiet, on the generated million-entry tree.
[What the Linux Round Changed](#what-the-linux-round-changed) has the detail.

**Question.** Why does fdu’s indexed tree trail pdu and diskus on the generated tree?

**Accepted.** A large one-shot index freed off the answer path, H156, −3.19%
`default-tree` (exp-160); no snapshot for a one-shot metadata report, H160, −13.81%
(exp-163).

**Rejected.** A direct file fold, H157, −2.22% `cold-scan-index` (exp-161); held
leaf-only chunks, H158, +0.88% (exp-162).

### macOS Rerun of Stack 141

exp-164 to exp-165. macOS, 2026-09-28, uncontrolled, on the generated million-entry
tree.

**Question.** Do H156 and H160, accepted on Linux, gain or cost anything on macOS?

**Rejected, change kept.** H156, −1.00% (exp-164); H160, −3.09% with an interval
crossing zero and peak RSS −26.29% (exp-165). Both ship on their Linux accepts.

### H159 Listing Recycle

exp-166 to exp-167, macOS, and exp-188 to exp-190, Linux; 2026-09-28.

**Question.** Does returning drained listings to the walker that allocated them pay?

**Verdicts.** On macOS the unbounded pool, +1.32% with peak RSS up to +5.04%, was
superseded (exp-166), and the bounded pool, −1.41%, rejected with the change kept
(exp-167). On `linux-v6.12` it was rejected twice, −2.19% (exp-188) and +2.26%
(exp-189); on `node-modules-dense` it was accepted, −8.61% (exp-190).

**Standing after.** The saving is per directory: absent on the sparse kernel tree, 8.61%
on the dense one (exp-190).

### H161 Ignore-Aware Summary

exp-170 to exp-172, macOS, and exp-187, Linux; 2026-09-28.

**Question.** Can the default summary classify `.gitignore` while it counts, instead of
falling back to the full index?

**Accepted.** Peak RSS −69.12% on the 137k-entry checkout (exp-170) and −57.86% on the
77k-entry rustup store (exp-171), the pre-registered primary on macOS; wall −6.93% on
`linux-v6.12`, where the 50% RSS bar was missed (exp-187).

**Superseded.** Holding whole listings, peak RSS −17.07% (exp-172).

### pdu on a Real Tree

exp-173 to exp-174 and exp-191. Linux, fc-v37, 2026-09-28, quiet, on `linux-v6.12`.
[What the Linux Round Changed](#what-the-linux-round-changed) has the detail.

**Question.** On a real repository, what separates fdu’s default command from pdu’s?

**Accepted.** Allocation-free `.gitignore` matching, H162, −47.02% `aggregate-summary`
and −46.19% on the default tree (exp-173); control chains resolved once per listing,
H163, −36.43% and −35.86% (exp-174).

**Rejected.** H157’s file fold again, on the product job its rerun pre-registered; the
probe’s `cold-scan-index`, −5.51%, was secondary (exp-191).

**Standing after.** The two took the default tree from 590.1 ms (exp-173’s control) to
211.3 ms (exp-174’s candidate), still about three times pdu’s 70 ms (exp-174).

### The Linux Overnight Round

exp-175 to exp-186 and exp-192 to exp-195. Linux, fc-v49, 2026-09-29, quiet.
[Its dated section](#the-linux-overnight-round-2026-09-29) has the tables.

**Question.** Can fdu’s default `fdu PATH` beat pdu’s default on two real trees, with
`.gitignore` on and no answer changed?

**Baselines.** Four-arm A/A cells of the 0.2.1 engine on each subject (exp-175 to
exp-177).

**Accepted.** H171, −29.62% `default-tree` (exp-178); H175, −3.31% (exp-179); H172 with
H176, −13.48% on `linux-v6.12` (exp-180) and −10.30% on `node-modules-dense` (exp-181);
H180, −8.68% and −5.63% `aggregate-summary` (exp-183, exp-184); H169 phase 1, −6.25% and
−7.90% on the `--no-controls` summary (exp-185, exp-186); H183, −7.62% `default-tree`
(exp-193).

**Rejected.** Three walkers, H165, +10.25% (exp-182); H181 with H182, +0.20% (exp-192).
H166 was closed by its gate without a build.

**Standing after.** Against the Q0 engine the default tree was 39.00% faster on
`linux-v6.12` (exp-194) and 9.75% on `node-modules-dense` (exp-195); fdu’s default
command went from 2.4 times pdu’s default to level with it on the kernel tree.

### The pdu Track

exp-197 to exp-201. Linux, fc-v50, 2026-09-30, quiet.
[Its dated section](#the-pdu-track-2026-09-30) has the tables.

**Question.** Can fdu get ahead of every pdu mode on both real trees?

**Accepted.** H185, −3.55% `default-tree` on `node-modules-dense` (exp-197); H188 with
H189, −6.15% `aggregate-summary` on `linux-v6.12` (exp-198); H186, −4.91% on
`node-modules-dense` (exp-199).

**Rejected.** H187, −0.35% on `linux-v6.12`: the tree route’s consumer has slack
(exp-200).

**Standing after.** Against the overnight round’s final head, −3.05% and −8.94% on the
default tree (exp-201), with fdu ahead of pdu’s default, `pdu --max-depth 2`, and diskus
on both real trees.

### The Review Rounds and the Automount Change

exp-196. Linux, fc-v50, 2026-09-29 and 2026-09-30.
[Its dated section](#the-review-rounds-and-the-automount-change-2026-09-29-to-2026-09-30)
has the detail.

**Question.** Do the review findings’ fixes cost anything on the routes they move?

**Rejected, change kept.** Every route on the native reader, H184, a correctness change
recorded as a non-regression screen: `warm-revalidate` −2.74% (exp-196). R163-1’s
searchability proof for H185 was counted, not timed.

### The 0.3.0 Release Standing

exp-202. Linux, fc-v50, 2026-09-30, quiet.
[Its dated section](#the-030-release-standing-2026-09-30) has the tables.

**Question.** What does the release engine do end to end, against 0.2.1 and the peers?

**Baseline, two builds.** The default tree −48.00% on `linux-v6.12` against 0.2.1, and
ahead of both pdu modes and diskus on all three subjects (exp-202).

## What the Linux Round Changed

The 2026-09-27 and 2026-09-28 Linux work (a 4-vCPU Firecracker guest, ext4, quiet) began
as a peer comparison on the generated tree and ended by finding a larger cost that tree
could not show.
[The pdu brief](../research/research-2026-09-28-pdu-and-the-linux-peer-gap.md) has the
detail.

**On a real repository the default command was bound by `.gitignore` classification, not
the walk.** The generated tree holds no ignore rules, and the published summary contract
ran with `--no-gitignore`, so neither exercised it.
On Linux v6.12 the default tree took 590 ms with `.gitignore` and 82 ms without, in the
same quiet run (exp-173); user CPU nearly equalled wall because one consumer thread did
the matching, at 78 allocations per entry.
Two things had hidden it.
On macOS the walk costs several µs per entry, so the same matching stayed off the
critical path: on the metabrowser checkout, reading `.gitignore` cost 22% more user CPU
and a wall change indistinguishable from zero, +1.6% [−4.0%, +4.4%]
([exp-106](../experiments/exp-106-default-gitignore-observation-versus-no-controls-on-metabrow.md)).
And on Linux the earlier leftover determination read 96% of this same job as its walk
phase and concluded the rest was the syscall floor
([exp-139](../experiments/exp-139-linux-walk-leftover-is-still-the-getdents64-plus-statx-floor.md));
that phase timer spans classification running on the consumer during the walk, and no
`.gitignore`-off arm was measured.
The lesson generalizes: a phase share cannot attribute work that runs concurrently
inside the phase, and a placebo with the suspected work turned off is the cheap check.

**Two changes removed most of it.** H162 matches without allocating: −47.0% on the
default summary and −46.2% on the default tree (exp-173). H163 resolves each directory’s
governing ignore files once for all its entries: another −36.4% and −35.9% (exp-174).
Both placebos include zero, and neither change moved the tree without ignore rules.
What remains, about 130 ms of the tree, is matching on one thread.
The
[0.2.2 design study](../research/research-2026-09-29-linux-default-tree-point-solution.md)
counted it per thread: 81% of the consumer’s instructions are a linear scan of about 111
patterns per entry, most of them literal names or `*.suffix`, which bucketed matching
(H171) replaces with lookups; a prototype cut those instructions 63% with identical
answers. H164, moving matching onto the walkers, follows only if a residual still binds.
The overnight round that followed landed H171 in a revised form (exp-178).

**The index-tier gap on the generated tree is user space.** fdu, pdu, and diskus make
the same system calls, and fdu spends less kernel time; it spends more user time and its
walkers park on a shared queue (14,668 voluntary context switches against pdu’s 17).
Under a preloaded jemalloc the unchanged binary ran level with pdu (screen), which
located the cost in allocations freed across threads.
In the same screen H159’s structural fix recovered about half of that allocator cost;
H166, H167, and H170 were the proposed next steps (H166 was later closed by its gate).

**Accepted and rejected in this round:**

- **H156**, freeing a large one-shot index off the answer path: −3.19% [−4.88%, −1.79%]
  ([exp-160](../experiments/exp-160-linux-one-shot-index-release-off-the-answer-path-clears-3-on.md)).
- **H160**, no snapshot for a one-shot metadata report: −13.81% on the default tree and
  −32.7% on a first run
  ([exp-163](../experiments/exp-163-linux-auto-cache-policy-stops-one-shot-snapshot-writes-clear.md)).
- **H161** on Linux: wall −6.93% [−11.29%, −0.17%] on Linux v6.12; the pre-registered
  50% RSS cut was met on the generated tree (314 to 8.6 MiB) and missed on Linux v6.12
  (−22.9%)
  ([exp-187](../experiments/exp-187-linux-h161-ignore-aware-transient-summary-clears-wall-rss-ba.md)).
- **H159**: rejected twice on Linux v6.12, −2.19% and then +2.26% on top of H162 and
  H163
  ([exp-188](../experiments/exp-188-linux-h159-listing-recycle-misses-on-linux-v6-12-10-6-on-the.md),
  [exp-189](../experiments/exp-189-linux-h159-rejected-again-on-linux-v6-12-after-h162-and-h163.md));
  accepted on the directory-dense tree (exp-190). Its saving is about 1.25 µs per
  directory on the generated tree, and the kernel tree averages 16 entries per directory
  against 8.5 on the dense one.
- **H157**, folding files straight into their parent: rejected on its probe job
  (−2.22%), then again on the product job its rerun pre-registered.
  Allocations fell about 40% both times
  ([exp-161](../experiments/exp-161-linux-direct-file-fold-and-owned-names-miss-3-on-cold-scan-i.md),
  [exp-191](../experiments/exp-191-linux-h157-file-fold-cuts-allocations-but-misses-on-the-prod.md)).
- **H158**, holding leaf-only chunks to cut consumer wakes: futex calls fell from
  105,732 to 17,938 and wall did not move
  ([exp-162](../experiments/exp-162-linux-detached-leaf-listing-hold-cuts-futex-wakes-but-not-wa.md)).

**A correction to the peer tables.** pdu counts the root as depth 1, so the
`--max-depth 1` job both comparisons ran printed only a total, not a rendered tree.
Both reports carry the correction, and the harness now passes `--max-depth 2`.

## The Linux Overnight Round (2026-09-29)

One unattended night on a 4-vCPU Firecracker guest (Intel Xeon at 2.1 GHz, Linux
6.18.44-fc-v49, ext4 on virtio, warm cache), run under
[the overnight plan](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md).
Its Status table is the summary of the night; this section says what the records add up
to. The goal was fdu’s default `fdu PATH` faster than pdu’s default on two real trees,
with `.gitignore` on and no answer changed.
Every verdict below comes from a quiet, interleaved, paired cell, most at 20 pairs.
Before each cell, the product command line’s text, JSON, and JSONL output was
byte-compared between control and candidate over all three subjects, and every
comparison matched.
[The research brief](../research/research-2026-09-29-linux-peers-matchers-and-hot-path.md)
has the peer, matcher, and hot-path evidence behind the queue.

**The review found two regimes.** With `.gitignore` on, the default tree kept about two
of four vCPUs busy: one consumer thread matched rules alone for the second half of the
run, and wall followed that thread.
With `.gitignore` off, the same command kept 3.3 busy, and wall followed total CPU
divided by about 3.4, plus a 5–10 ms tail.
That set the order of the queue: first take classification off the critical path (H171,
H175), then shrink the consumer’s index build (H172). After that, only removing CPU from
the walk moves wall; moving work between threads does not.

**The baselines set the noise rules.** Quiet four-arm cells of the 0.2.1 engine, each
with two copies of the same binary, 12 pairs:

| Subject | Default tree | `--no-controls` | Default summary | `--no-controls` | Record |
| --- | ---: | ---: | ---: | ---: | --- |
| `linux-v6.12` | 181.9 ms | 89.9 ms | 149.5 ms | 77.8 ms | [exp-175](../experiments/exp-175-linux-fc-v49-baseline-default-tree-182-ms-2-4x-pdu-with-a-fa.md) |
| `node-modules-dense` | 84.0 ms | 85.5 ms | 76.6 ms | 71.5 ms | [exp-176](../experiments/exp-176-linux-fc-v49-baseline-on-node-modules-dense-default-tree-84-.md) |
| `linux-balanced-1m` | 1,358.6 ms | 1,372.2 ms | 1,234.3 ms | 1,210.2 ms | [exp-177](../experiments/exp-177-linux-fc-v49-baseline-on-the-generated-million-entry-tree-de.md) |

One same-binary comparison, the `linux-v6.12` default summary, read −3.74%
[−12.33%, −0.59%]: an accept by the rule’s arithmetic with both arms identical.
From then on, a candidate predicted below 10% was measured at 20 pairs or recorded as a
screen, and a placebo that excluded zero by more than 3% blocked its verdict.
A review finding fixed the harness as well: a cell with any invalid sample now reads
inconclusive, and the recorder refuses such an accept (`fdu-c2c6`).

**Accepted.** All at 20 pairs; every placebo in these cells included zero, and answers
were identical.

| Change | Subject and job | Paired change [95%] | Record |
| --- | --- | --- | --- |
| H171, bucketed `.gitignore` matching | `linux-v6.12`, default tree | −29.62% [−33.58%, −26.11%] | [exp-178](../experiments/exp-178-linux-h171-bucketed-gitignore-matching-cuts-the-default-tree.md) |
|  | `linux-v6.12`, default summary | −25.45% [−28.15%, −21.74%] | exp-178 |
| H175, control chains derived from the parent’s, on H171 | `linux-v6.12`, default tree | −3.31% [−7.43%, −0.84%] | [exp-179](../experiments/exp-179-linux-h175-derived-control-chains-take-another-3-off-the-def.md) |
| H172 with H176 and F6e, the exact transient tree tier | `linux-v6.12`, default tree | −13.48% [−18.85%, −6.31%] | [exp-180](../experiments/exp-180-linux-h172-exact-transient-tree-tier-cuts-the-default-tree-1.md) |
|  | `node-modules-dense`, default tree | −10.30% [−15.09%, −7.20%] | [exp-181](../experiments/exp-181-linux-h172-transient-tree-tier-cuts-the-default-tree-10-on-n.md) |
| H180, summary-route walker trims | `node-modules-dense`, default summary | −8.68% [−11.49%, −4.66%] | [exp-183](../experiments/exp-183-linux-h180-summary-walker-trims-cut-the-default-summary-9-on.md) |
|  | `linux-v6.12`, default summary | −5.63% [−15.15%, −2.75%] | [exp-184](../experiments/exp-184-linux-h180-summary-walker-trims-cut-the-default-summary-6-an.md) |
| H169 phase 1, a Linux-native directory reader | `node-modules-dense`, summary `--no-controls` | −6.25% [−14.02%, −1.23%] | [exp-185](../experiments/exp-185-linux-h169-native-directory-reader-cuts-the-summary-6-10-and.md) |
|  | `linux-v6.12`, summary `--no-controls` | −7.90% [−9.55%, −4.74%] | [exp-186](../experiments/exp-186-linux-h169-native-directory-reader-cuts-the-summary-8-9-on-l.md) |
| H183, cheap pre-checks for the residual rules | `linux-v6.12`, default tree | −7.62% [−10.41%, −5.28%] | [exp-193](../experiments/exp-193-linux-h183-cheap-matcher-pre-checks-cut-the-default-tree-8-o.md) |

- **H171** compiles each `.gitignore` into literal-name and ends-with maps, anchored
  rules grouped by segment count, and a residual list behind cheap pre-checks, and
  answers with the highest matching index.
  Full glob evaluations fell from about 110 per entry to 0.0019. The controls-on tree
  came to 1.11 times its own `--no-controls` arm, inside the predicted 1.15. fdu’s
  ignored set agreed with `git check-ignore --no-index` on every path of `linux-v6.12`
  and of a 324,015-path built overlay.
- **H175** derives each listing’s control chain from its parent’s instead of probing a
  map once per ancestor.
  It was accepted at the margin, as predicted, and the summary did not move, also as
  predicted.
- **H172** applies when a one-shot tree request proves nothing reads the index.
  It keeps every directory and only the K = ⌈100/min-share⌉ largest files, and folds the
  rest into their directory’s totals.
  H176 (no per-extension maps under the tier) and F6e (the fused post-walk pass) are
  part of it and not separately claimed.
  The kernel tree’s index holds 5,930 entries instead of 92,473; on the generated tree,
  a screen, wall fell 3.20% and peak RSS 79%.
- **H180** decides the control spelling from the listed name bytes and moves each path
  into its operation instead of cloning it.
  Walker instructions fell 28–33%. With `.gitignore` off the summary gained 12.76% on
  `linux-v6.12`, but its −1.12% on `node-modules-dense` did not clear.
- **H169 phase 1** reads `getdents64` into a reused buffer, uses the names in place, and
  stats each entry with `statx` relative to the directory, passing `AT_NO_AUTOMOUNT`. It
  sits behind four audited `unsafe` expressions on Linux glibc.
  `fstat` calls fell from 5,773 to 4 on the kernel tree, and the per-entry `statx` no
  longer triggers automounts on the reader’s path (`fdu-puk7`; the review of #161 then
  put every other route on the reader too, `fdu-d2fn`, H184, exp-196). The default
  summary gained 9.7% and 9.4%. The default tree, controls on, gained 4.29%
  [−8.81%, −0.90%] on `node-modules-dense` and 2.06% [−7.38%, +1.83%], not clearing, on
  `linux-v6.12`, against a predicted 6–8%. The records’ frontmatter, and so the ledger,
  carry the `--no-controls` tree pair the deciding job used: −1.67% and −1.79%, neither
  clearing.
- **H183** computes a 32-class byte set of each name once per entry and rejects a
  residual rule by a mask test and its first and last literal bytes before any string
  comparison; the survivors compare with inline byte loops.
  Found by a callgrind of the H169 head, it cut the consumer from 436M to 238M
  instructions and matching’s `memcmp` from 101M to zero.
  Both placebos, `--no-controls` and `node-modules-dense`, included zero.

**Rejected and closed:**

- **H165**, the walker count, screened on the H172 head at 12 pairs
  ([exp-182](../experiments/exp-182-linux-walker-count-after-h172-three-walkers-regress-six-and-.md)):
  three walkers regressed 10.25% on `linux-v6.12` and 21.95% on `node-modules-dense`,
  and six and eight did not clear on both real subjects.
  `PORTABLE` is unchanged: after H172 the walk sets the time, and more walkers on four
  vCPUs do not shorten it.
- **H181 and H182**, conditional queue wakes and a hash-ordered listing sort, measured
  as one bundle
  ([exp-192](../experiments/exp-192-linux-h181-conditional-queue-wakes-and-h182-hash-ordered-lis.md)).
  The queue’s condvar wakes fell from 1,426 to 3–10 per run and consumer instructions
  about 3%, but the default tree moved +0.20% [−2.04%, +1.91%] on `linux-v6.12` and
  −1.40% [−7.42%, +3.42%] on `node-modules-dense`. Neither is merged.
  It is H158’s lesson again: work cut from a thread the clock is not waiting on does not
  move wall.
- **H166**, waking one walker per new directory, was closed by its gate without a build:
  walkers waited for work 0.6% of their time on its deciding subject,
  `node-modules-dense`, and 2.2% on `linux-v6.12`.

**The standing against the peers.** Tool-harness cells with the `fdu-default-tree`
contract: `fdu --color never PATH` against pdu 0.24.0’s default
(`pdu --silent-errors PATH`), pdu with `--max-depth 2`, and diskus 0.9.0, all quiet and
interleaved.
Each percentage is the peer’s paired change against the adjacent fdu run, so
a negative figure means the peer took less time.

| Cell | Pairs | Subject | fdu | pdu default | pdu `--max-depth 2` | diskus |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| Q0, the 0.2.1 engine (exp-175) | 12 | `linux-v6.12` | 0.19 s | 0.079 s, −58% | 0.074 s, −59% | 0.085 s, −55% |
| Q0 (exp-176) | 12 | `node-modules-dense` | 0.086 s | 0.077 s, −11% | 0.070 s, −20% | 0.075 s, −12% |
| After H172 (exp-180 evidence) | 12 | `linux-v6.12` | 0.084 s | 0.078 s, −8% | 0.070 s, −14% | 0.079 s, −6% |
| After H172 | 12 | `node-modules-dense` | 0.076 s | 0.077 s, +0% | 0.066 s, −12% | 0.075 s, −2% |
| After H169 (exp-185 evidence) | 20 | `linux-v6.12` | 0.077 s | 0.072 s, −8% [−10%, −4%] | 0.067 s, −10% | 0.073 s, −4% |
| After H169 | 20 | `node-modules-dense` | 0.080 s | 0.081 s, +1% [−3%, +7%] | 0.076 s, −4% | 0.082 s, +1% |
| Final head, H183 included (exp-194 evidence) | 20 | `linux-v6.12` | 0.122 s | 0.121 s, +1% [−2%, +2%] | 0.114 s, −1% [−7%, +1%] | 0.125 s, +2% [−1%, +9%] |
| Final head | 20 | `node-modules-dense` | 0.118 s | 0.119 s, +1% [−2%, +4%] | 0.113 s, −7% [−10%, −4%] | 0.117 s, −2% [−4%, +2%] |

Only these within-cell ratios are evidence.
Across cells the host drifted by up to about 50–70% over the night: the same Q0 binary’s
default tree read 84.0 ms on `node-modules-dense` in exp-176 and 126.9 ms in exp-195,
and pdu’s own default on `linux-v6.12`, an unchanged binary, went from 0.072 s after
H169 to 0.121 s at the final cell.
Read within cells, fdu’s default command went from 2.4 times pdu’s default to level with
it on `linux-v6.12`, and from 11% behind to level on `node-modules-dense`; diskus is
level on both. The night’s target, at most 1.25 times on `linux-v6.12` and the dense
tree’s gap closed, was met after H172. The destination, faster than pdu’s default on
both real trees, was not: fdu reached parity, and pdu with `--max-depth 2` stays 7%
ahead on the dense tree.

**The round end to end.** One paired cell of the Q0 engine against the final head
(exp-194, exp-195, 20 pairs) put the default tree 39.00% [−42.99%, −34.93%] faster on
`linux-v6.12` (200.3 to 119.8 ms) and 9.75% [−11.84%, −7.33%] faster on
`node-modules-dense`, with the default summary 26.25% and 7.02% faster.
On the generated million-entry screen the tree gained 9.08% and the summary 19.80%, and
the tree’s peak RSS fell from 292 to 62 MiB. `.gitignore` handling now costs the kernel
tree 1.6% of its blind walk, from 52% (both ratios of medians; paired, the Q0 engine’s
blind arm was 31.98% faster than its controls-on arm).
Every median fell short of its pre-registered range, in the slower regime below: the two
default trees by 1 and 0.25 points, and the two default summaries by about 4 and 5
points.

The final tool cells, run just before the end-to-end cell, found the host in that slower
regime: every tool took about 0.12 s, and 85–90% of each tool’s CPU was kernel time
(85–88% for fdu, pdu’s default and diskus, and 89–90% for pdu with `--max-depth 2`).
There fdu used less CPU than pdu’s default and diskus on both trees (445 ms against 461
and 486 on `linux-v6.12`), but made more voluntary context switches (608 against pdu’s
default 128), the handoff between walkers and consumer that H181 cut without moving
wall. fdu kept 3.65 cores busy against pdu’s default 3.81 on `linux-v6.12`, and 3.54
against 3.72 on the dense tree.
It did less work and came out level on wall, likely because of a longer serial tail
after the walk, which has not yet been profiled (`fdu-j4p7`).

**What is left.** After H169, a callgrind of the default tree on `linux-v6.12` counted
436M consumer instructions with `.gitignore` against 103M without; `memcmp` (114M) and
the residual rules’ pre-checks (`Checks::admit`, about 97M) were most of the difference.
H183 removed most of it (exp-193): the consumer fell to 238M instructions and the
default tree 7.62% [−10.41%, −5.28%], leaving that tree 1.086 times its own
`--no-controls` arm.
With `.gitignore` off, and on the dense tree, wall follows the walk’s total CPU, the
target of H179 and H177.

## The pdu Track (2026-09-30)

The overnight round left fdu level with pdu’s default and behind pdu with
`--max-depth 2` on the dense tree.
[The uniformly-faster brief](../research/research-2026-09-29-uniformly-faster-than-pdu.md)
attributed that standing with load-independent counts and registered H185–H190 to put
fdu ahead of every pdu mode on both real trees.
Every cell ran quiet, interleaved and at 20 pairs on the same kind of 4-vCPU Firecracker
guest (Linux 6.18.44-fc-v50, ext4, warm cache), each change against the head before it,
with answers proved identical before each cell.

| Change | Subject and job | Paired change [95%] | Record |
| --- | --- | --- | --- |
| H185, directory and symlink kinds taken from the listing on the folded tree route | `node-modules-dense`, default tree | −3.55% [−7.85%, −2.57%] | [exp-197](../experiments/exp-197-linux-h185-describes-each-directory-once-on-the-folded-tree-.md) |
| H188 with H189, byte-wise summary fold and pre-sized control reads | `linux-v6.12`, default summary | −6.15% [−7.94%, −1.80%] | [exp-198](../experiments/exp-198-linux-h188-byte-wise-summary-fold-and-h189-pre-sized-control.md) |
| H186, tree rows admitted before they are built, a folded index released detached | `node-modules-dense`, default tree | −4.91% [−6.94%, −2.86%] | [exp-199](../experiments/exp-199-linux-h186-admits-tree-rows-before-building-them-the-default.md) |
| H187, sort only what the folded tree keeps (rejected) | `linux-v6.12`, default tree | −0.35% [−3.61%, +2.19%] | [exp-200](../experiments/exp-200-linux-h187-sorts-only-what-the-folded-tree-keeps-a-38-consum.md) |

H185 and H186 were each not resolvable alone on `linux-v6.12`; exp-201 measures their
sum there. H187 cut the tree route’s consumer instructions 20% and 38% and moved wall on
neither tree: on four vCPUs that consumer has slack, so after H185 and H186 the next
wall on the tree route comes from the walkers’ kernel time or the serial tail.
The shipped H185 stats each listing’s children until one stat succeeds, which proves the
directory searchable, before it takes kinds from the listing (review finding R163-1); it
keeps 83% and 89% of exp-197’s `statx` saving, a difference not re-measured on wall.

**End to end and the standing
([exp-201](../experiments/exp-201-linux-the-pdu-track-end-to-end-the-default-tree-3-and-9-fast.md)).**
Against the overnight round’s final head in one paired cell, the default tree is 3.05%
[−5.81%, −1.03%] faster on `linux-v6.12` and 8.94% [−12.29%, −5.04%] faster on
`node-modules-dense`, and the default summary 6.14% faster on `linux-v6.12`. In the same
session each peer was paired 20 times with the shipped `fdu` product binary; positive
means the peer took longer:

| Subject | fdu | fdu, final head | pdu default | pdu `--max-depth 2` | diskus |
| --- | ---: | ---: | ---: | ---: | ---: |
| `linux-v6.12` | 0.084 s | 0.088 s, +4% [+2%, +7%] | 0.094 s, +13% [+10%, +15%] | 0.088 s, +3% [+1%, +8%] | 0.094 s, +12% [+8%, +16%] |
| `node-modules-dense` | 0.079 s | 0.087 s, +12% [+7%, +13%] | 0.092 s, +15% [+13%, +20%] | 0.086 s, +10% [+4%, +13%] | 0.090 s, +11% [+10%, +15%] |

About six points of the lead over pdu’s default are that night’s regime: the final head
alone led pdu’s default by about 6% in the same run, where exp-194 had them level
(anchor-normalized, +9.6% and +5.6%). The lead over pdu `--max-depth 2` is the track’s
own, since the final head was level with it on the kernel tree and 2.4% behind on the
dense tree; on the kernel tree it is at the edge of what 20 pairs resolve.
The generated million-entry tree and macOS were not measured against the peers in this
track.

## The Review Rounds and the Automount Change (2026-09-29 to 2026-09-30)

The pull requests that carried the Linux work were reviewed before they merged, and two
findings changed code that a record had already measured.
Neither was a speed change, and each ships on correctness.

**Every route on the native reader (R161-2, `fdu-d2fn`, H184).** The review of
[#161](https://github.com/jlevy/fdu/pull/161) found that H169 phase 1 had put
`AT_NO_AUTOMOUNT` on the parallel walk’s stats alone, so on a tree holding an unmounted
autofs trigger directory the same request answered differently by route and by worker
count. The fix lists every route (the serial and concurrent walks, revalidation,
reconciliation, opened discovery) through the native reader, and stats with the same
flags the paths a route verifies by itself; musl needed no code.
Its non-regression screen ran quiet at 20 pairs on `linux-v6.12`, with the #161 layer
head as control
([exp-196](../experiments/exp-196-linux-every-route-lists-through-the-native-reader-and-no-sta.md)):

| Job | Control | Every route on the reader | Change [95%] |
| --- | ---: | ---: | --- |
| `warm-revalidate` | 554.2 ms | 542.4 ms | −2.74% [−3.96%, +0.28%] |
| `opened-discovery` | 2,108.3 ms | 2,158.7 ms | +1.19% [+0.56%, +3.51%] |
| `default-tree` | 109.1 ms | 110.2 ms | +0.42% [−2.67%, +4.11%] |
| The serial walk, `aggregate-summary --no-controls --threads 1` | 344.9 ms | 328.4 ms | −5.14% [−6.63%, −2.44%] |

The reconciliation component’s time fell 3.56% [−6.98%, +0.76%], an interval that
crosses zero: the reader’s saving on a route that had taken the portable listing.
Discovery’s own component moved +0.06%, so the +1.19% on its wall lies in the journal
drain and validation the job times after the walk, and is worth a profile before it is
called a cost. The serial walk, exp-185’s portable placebo, is now the reader.
`strace` shows glibc’s per-`opendir` `fstat` gone (5,773 calls to 4) and every `statx`
of a tree entry carrying `AT_NO_AUTOMOUNT`. The record is `rejected` with the candidate
kept: nothing was accepted on speed.

**A directory that lists but refuses search (R163-1, `fdu-wigy`).** The review of
[#163](https://github.com/jlevy/fdu/pull/163) found that exp-197’s identical answers
held on the three subjects, none of which has a directory with mode `0400`, and not on
every tree: there H185’s measured build admitted a subdirectory and a symlink from
`d_type` where the full index reports each child as an error.
The shipped policy (`0e59c387`) stats a listing’s children until one stat succeeds,
which proves the directory searchable, and only then takes kinds from `d_type`. It makes
72,075 `statx` calls on `node-modules-dense` and 87,656 on `linux-v6.12`, against 70,416
and 87,006 as measured, so it keeps 83% and 89% of exp-197’s saving
([exp-197](../experiments/exp-197-linux-h185-describes-each-directory-once-on-the-folded-tree-.md),
amended). The difference is below what a 20-pair cell resolves and was not timed alone;
the release cell includes it.

**Also in the release, first timed by exp-202.** The stability layer
([#164](https://github.com/jlevy/fdu/pull/164)) adds a checked running total per file on
every route (the every-route overflow check), and reads `.gitignore` byte-order marks
and embedded NUL bytes as git reads them (`fdu-ifci`). exp-202’s record lists these,
with H184 and the R163-1 latch, as what the release adds to exp-201’s engine.

## The 0.3.0 Release Standing (2026-09-30)

The last measurement before the release: the release engine (`b82f26e1`) end to end
against 0.2.1 (`c1644575`) and against the peers, on every subject the host has
([exp-202](../experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md)).
It ran quiet on a 4-vCPU Firecracker guest (Linux 6.18.44-fc-v50, ext4, warm cache), 20
pairs and 3 warm-ups per cell, with no invalid sample.
It is a baseline record: no decision rests on it, and it states what the shipped engine
does rather than what the development heads before it did.

**Against 0.2.1, the engine.** Probe cells, paired change in wall time:

| Subject | Job | 0.2.1 | Release | Change [95%] |
| --- | --- | ---: | ---: | --- |
| `linux-v6.12` | `default-tree` | 208.6 ms | 109.9 ms | −48.00% [−50.45%, −44.79%] |
| `linux-v6.12` | `aggregate-summary` | 184.6 ms | 128.1 ms | −34.63% [−40.37%, −27.19%] |
| `node-modules-dense` | `default-tree` | 135.0 ms | 115.6 ms | −14.09% [−21.53%, −11.50%] |
| `node-modules-dense` | `aggregate-summary` | 115.3 ms | 99.5 ms | −12.44% [−15.85%, −11.28%] |
| `linux-balanced-1m` | `default-tree` | 1,109.1 ms | 947.8 ms | −14.57% [−18.46%, −12.71%] |
| `linux-balanced-1m` | `aggregate-summary` | 1,018.5 ms | 834.2 ms | −16.65% [−20.07%, −13.97%] |

The development records compounded had predicted about −41%, −18% and −13% for the
default tree; each prediction chains cells from different sessions, so the direct
figures are the ones to quote.
On the default tree user CPU fell 75%, 28% and 28% and system CPU far less (−1.6%,
−5.6%, −8.0%): what remains is the walk’s kernel time.
The million-entry default tree’s peak RSS fell 80.43%, from 292.7 to 57.3 MiB.

**Against the peers.** Tool cells with the release’s `fdu --color never PATH` as the
anchor; each competitor is paired 20 times with the adjacent anchor run, and positive
means it took longer:

| Subject | Release `fdu` | 0.2.1 `fdu` | pdu default | pdu `--max-depth 2` | diskus |
| --- | ---: | ---: | ---: | ---: | ---: |
| `linux-v6.12` | 0.110 s | 0.215 s, +94.7% [+85.8%, +112.4%] | 0.124 s, +15.3% [+12.0%, +19.3%] | 0.118 s, +9.7% [+1.8%, +12.2%] | 0.123 s, +16.8% [+3.3%, +20.5%] |
| `node-modules-dense` | 0.106 s | 0.126 s, +20.5% [+18.3%, +22.4%] | 0.126 s, +18.2% [+15.2%, +23.0%] | 0.119 s, +11.5% [+1.9%, +18.9%] | 0.117 s, +12.1% [+9.3%, +16.5%] |
| `linux-balanced-1m` | 0.951 s | 1.131 s, +20.6% [+17.9%, +21.7%] | 1.196 s, +25.3% [+20.5%, +27.7%] | 1.125 s, +18.8% [+15.0%, +19.7%] | 1.171 s, +23.9% [+20.4%, +28.4%] |

Every interval excludes zero, and the release’s standard output was byte-identical to
0.2.1’s in every sample.
On the million-entry tree the release held 58.5 MiB at peak against 0.2.1’s 293.8 MiB
and pdu’s default 93.4 MiB.

**What the session adds.** Every tool’s median on the real trees was 29–38% above its
exp-201 figure, so this was a slower session on the same kind of guest.
The lead over pdu’s default is about where exp-201 left it (15% and 18%, against 13% and
15%); the leads over `pdu --max-depth 2` (10% and 12%, against 3% and 10%) and over
diskus are wider. The run carries no exp-201 engine, so it does not separate the
release’s later changes from the session; they were expected to cost nothing, not to
gain. On the million-entry tree 0.2.1 was level with `pdu --max-depth 2`, +0.8%
[−2.4%, +1.7%], where the 2026-09-29 figures put pdu about 11% ahead of it, so about ten
points of the release’s 18.8% lead there are the session’s, by an estimate across
sessions. The release’s involuntary context switches rose from 56 to 273 on the kernel
tree’s default tree, which fails the qualification block’s switch gate, as exp-198
recorded for the summary.

## Qualifications on Current Results

These do not overturn a verdict; they say what a verdict rests on.

- **H156 and H160 were accepted on a generated tree only.** The loop asks for a
  nominated real tree in any accept set, and exp-160 and exp-163 ran only on the
  generated balanced tree, as did the Linux peer tables.
  The page’s per-platform section counts 7 of 28 Linux improvements decided on generated
  trees; every accept since 2026-09-29 was decided on a real tree.
- **H161’s Linux verdict met one of its two bars.** It was accepted on wall; its 50% RSS
  bar was missed on the deciding subject, where classification holds about 20 MiB more
  than a walk without it (`fdu-nyj8`).
- **H159’s deciding subject changed after two misses.** The dense tree was registered
  before exp-190 ran and fits the per-directory mechanism, but it was chosen after the
  first subject had failed.
  exp-167, the macOS cell, is recorded as rejected with the change kept, as exp-164 and
  exp-165 record H156’s and H160’s macOS cells.
- **The leftover determinations on Linux v6.12 predate the `.gitignore` finding.** H140
  (exp-139) and H146
  ([exp-147](../experiments/exp-147-linux-first-run-leftover-is-still-the-walk.md))
  attributed the default command to the walk; exp-173 shows most of it was
  classification. Both registry rows carry that caveat, and a new determination needs a
  `--no-controls` arm.
- **Every macOS accept since 0.1.0 is uncontrolled.** They stand as paired evidence on a
  busy host; none has a quiet replication.
- **The 2026-09-29 round ran on one virtualized host.** Every cell from exp-175 to
  exp-186, and exp-192 to exp-195, ran on the same 4-vCPU Firecracker guest, and the pdu
  track’s, exp-196 to exp-201, and the release standing, exp-202, on the same kind of
  guest at a later kernel build.
  Its walker-count screen and its tree-tier effects are evidence about four vCPUs, ext4
  on virtio, and a warm cache, not about bare metal or wider hosts.
- **None of the 2026-09-29 changes has been measured on macOS.** H171, H175, H172, H180,
  and H183 are portable code and run there; H169’s reader is Linux glibc only.
  Nor has any of the pdu track’s: H186, H188, and H189 are portable, and H185 is a no-op
  there, since the macOS listing carries every child’s attributes.
  Their macOS effect is unknown.
- **H169’s tree-route gain is below its prediction.** It was accepted on the
  `--no-controls` summary, whose −6.25% on `node-modules-dense` and −7.90% on
  `linux-v6.12` also fell short of the predicted −8% to −12%, the second by a tenth of a
  point. The default tree, controls on, gained 4.29% on `node-modules-dense` and 2.06%,
  not clearing, on `linux-v6.12`, against a predicted 6–8%. The tree route stats every
  entry, directories included, and the rest of its time is the kernel’s `statx` and the
  consumer, which the reader does not touch.
- **The release cell does not separate the release’s later changes from its session.**
  exp-202 carries no exp-201 engine and ran in a session 29–38% slower for every tool on
  the real trees. H184 was screened alone on the #161 head (exp-196) but not on exp-201’s
  engine; the R163-1 latch and the #164 changes were never timed alone.
- **The generated tree screens only.** `linux-balanced-1m` decided nothing on
  2026-09-29. H172’s −3.20% wall and −79% peak RSS on it are a screen, and its base
  moved from 1,359 to 1,748 ms between two cells of the same night.
- **The Linux floor scoreboard is stale.** It was last derived on 2026-09-20 (exp-141).
  H147, H72, H156, H160, H161, H159, H162, H163, H171, H175, H172, H180, H169, H183,
  H185, H188, H189, and H186 have landed or been accepted since, and the loop re-derives
  the ×floor after an accepted change before trusting the queue.

## Dead Ends Worth Knowing

Each of these was measured and rejected, and each has a mechanism that explains why, so
none should be retried without a new one.

- **Rearranging the Linux syscall layer.** Batching is bounded at 9% and ran 6–8× slower
  through io_uring; eliding the terminating `getdents64` is under 1% of wall; narrower
  records were refuted four ways (H62–H65).
  [The campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md)
  records them as closed.
- **Smarter adaptive worker controllers on APFS.** Repeated windows, staged expansion,
  and higher fixed counts regressed 36–61%
  ([exp-057](../experiments/exp-057-reject-repeated-adaptive-worker-windows-on-apfs.md);
  [the gap-closure report](report-2026-08-15-adaptive-worker-gap-closure.md)).
- **An allocator dependency.** mimalloc won 23% on the aggregate tier and raised its
  peak RSS 139% (H74); recycling buffers missed H85’s 20% bar
  ([exp-150](../experiments/exp-150-linux-h85-recycle-misses-the-20-mimalloc-bar.md)).
  Structural fixes were kept instead (H147, H159); the jemalloc screen reopens the
  question for the maintainer.
- **More Linux walkers while the consumer is the bottleneck.** `--threads 8` regressed
  the default summary 7.1% on `/usr`
  ([exp-149](../experiments/exp-149-linux-default-usr-aggregate-threads-8-regresses-do-not-lower.md))
  and helped with `.gitignore` off.
  H165 re-screened the count once classification was cheap (exp-182): three walkers
  regress 10–22%, and six and eight do not clear on both real subjects.
- **Cutting counts off the critical path.** Fewer allocations (H157, H114), fewer wakes
  (H158, H181), and fewer consumer instructions (H182) did not move wall where the saved
  work was not what the clock waited on (exp-192).
- **Restore trims on the content cache hit.** Parse-speed, file-count completeness, and
  allocation trims (H113, H114, H116, H103) were each rejected; the mix is apply and
  candidate install
  ([exp-144](../experiments/exp-144-linux-cache-hit-leftover-after-landed-stack.md)).
- **Restarting the H86 rewrite.** After the floor stage failed, the Linux leftover is
  the walk plus retained-index RSS
  ([exp-142](../experiments/exp-142-linux-h111-leftover-is-still-walk-floor-plus-rss.md)),
  not another representation change.
- **Shipping the PGO screen as measured.** Its −8.35%
  ([exp-154](../experiments/exp-154-linux-pgo-screen-clears-3-on-index-and-revalidate.md))
  was trained and measured on one tree, so it is a ceiling; the profile is host-specific
  and is not checked in (`fdu-pdne`).

## Campaign 2 Termination

[Campaign 2](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) defined how
a tier closes: its ×floor on nominated real subjects reaches a threshold, or two
consecutive re-screens after a landing name no mechanism worth 3%. No tier has been
recorded as closed.

| Tier | Regime | Status |
| --- | --- | --- |
| Index | Linux | 1.78× the floor against 1.4×, RSS 5.20× against 3× (exp-141); bare-metal remeasure open (`fdu-xde5`) |
| Aggregate, `.gitignore` off | Linux | 1.59× on `linux-v6.12` and 1.86× on `/usr` against 1.25× (exp-141), before H147 and H72; re-derive |
| Content cache hit | Linux | Two consecutive leftover determinations named no new 3% cut (exp-144, exp-155); closure not recorded |
| Any tier | macOS | No floor denominator: the floor program is Linux-only (`fdu-33ri`, `fdu-9hdc`) |

The `.gitignore`-on default command is not a campaign-2 tier.
At 0.2.1 it held the largest measured Linux gap, 211 ms against 81 ms with `.gitignore`
off.
After the 2026-09-29 round the controls-on default tree was 1.6% above its blind arm
in exp-194 (119.8 against 117.9 ms, a ratio of medians); in exp-193’s cell it was 8.6%
(67.4 against 62.1 ms).

## Open Work

Grouped by topic; the order to run them in is
[Current Pickup](../guides/performance-loop-runbook.md#current-pickup-2026-09-30).

- **Next on Linux** (epics `fdu-8a8r` and `fdu-faqa`), in order:
  1. **H169 phase 3**, directories opened relative to the parent’s descriptor, which
     needs an fd budget sized to the breadth-first frontier, and **H177**, a per-listing
     name arena: walker-side cuts, which exp-200 ranks above any consumer cut.
     The narrowest leads are over pdu `--max-depth 2` on the real trees, +10%
     [+2%, +12%] and +12% [+2%, +19%] in exp-202, and +3% [+1%, +8%] on `linux-v6.12` in
     exp-201’s session.
  2. **H178**, the consumer walking when its channel is empty, re-predicted at −1% to
     −3%, under the three-stage qualification a scheduling policy takes.
  3. **The full generated-tree peer table** on the shipped engine: exp-202 measured
     pdu’s two modes, diskus and 0.2.1 there, not the other peers of
     [the Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md).
  4. Behind their gates: **H190**, a consumer-only cut on the tree route, which has no
     wall to buy after exp-200; **H179**, which after H185 applies only to the
     full-index route; **H164**’s tree route (`fdu-emqf`) and **H174** (`fdu-sfse`).
- **Follow-ups from the reader and the matcher:**
  - `fdu-q7hf`, the reader’s public diagnostics fields, and `fdu-ifci`, a UTF-8 BOM and
    an embedded NUL in `.gitignore` read as git reads them, are in
    [#164](https://github.com/jlevy/fdu/pull/164) for 0.3.0;
  - `fdu-d2fn`, done: every route lists through the native reader and every stat of a
    listed child passes `AT_NO_AUTOMOUNT` (H184, exp-196); musl never needed it, and
    `fdu-puk7` was closed for the reader’s path by exp-185;
  - the benchmark half of the matcher survey (`fdu-p6vc`).
- **Confirm on real trees:** H156 and H160 (`fdu-b9ga`); H162 and H163 on macOS
  (`fdu-dv07`), and the 2026-09-29, pdu-track and release changes there (the platform
  review’s cells, exp-203 onward); the Linux ×floor scoreboard (`fdu-z2h6`).
- **H167** (`fdu-lwsy`), **H168** (`fdu-mw2s`), **H170** (`fdu-lz25`): directory tokens
  in place of the path-keyed directory map, re-measured after H172 as its row asks; no
  per-file extension `String`, on the routes that still keep the full index; and a
  per-worker summary fold, after a review reconciles it with H62–H65.
- **`fdu-nyj8`**: attribute the 20 MiB the classifying summary holds on Linux v6.12.
- **H153** confirmation on a quiet host (`fdu-9e9d`), its Linux replication (H154,
  `fdu-wbhe`), and a post-H153 profile (H155, `fdu-83wn`).
- **H151**: the attached progress handle, rerun only when the quiet gate holds.
- **Regime gaps:** PGO pipeline adoption (`fdu-pdne`), a bare-metal Linux host
  (`fdu-lf3v`, `fdu-tk1b`), the quiet-host peer cell (`fdu-ow8y`), and a macOS floor
  (`fdu-33ri`).
- **Record gaps:** the cross-platform coverage matrix (`fdu-uxl0`) and the
  per-improvement report sections (`fdu-72bn`), in
  [the record and report plan](../specs/active/plan-2026-08-15-fdu-performance-record-and-report.md).

## Reading the Charted Page

The page shows every number; this report says what they mean.

**Over time.** Two stacked panels on one axis of experiments in the order they ran.
The top panel is the fdu performance index
([the index spec](../specs/active/plan-2026-10-05-fdu-performance-index.md)): one score
across twelve components, each a history cell (`performance-evidence/history/`) timing
thirteen milestone builds, from the pre-work binary `b565882b` through each major kept
optimization to 0.3.0, interleaved against 0.3.0 on an Apple M1 Pro’s internal SSD.
Every value is a multiple of 0.3.0’s, on a log scale.
A chooser switches the panel from the unified score to any single component, and fades
the experiment bars that do not count toward the chosen metric.

- **The macOS score, 12 of 12 components:** 1.71× better [1.68×, 1.74×] from 0.2.0, the
  first build with every component, to 0.3.0.
- **The partial macOS score, 4 of 12 components** (cold cache, default tree, scale,
  memory; 40% of the weight): 4.73× better [4.60×, 4.87×] from the pre-work binary.
- **Exploratory, uncontrolled host.** CPU was above the 25% quiet gate at many sample
  boundaries in most cells, so differences of about 10% between adjacent builds are
  within the noise. A score quoted in release notes waits on a quiet re-time at 20 rounds
  (`fdu-bkj2`). Linux, half of the combined index, is not yet measured.

Two of the cells are the history cells this page first drew, now the cold-cache and
scale components:

- **The generated 1M-entry tree:** 70.4 s to 6.54 s, 10.8× (paired 10.76×, 95%
  [8.88×, 11.65×]). The fall is H1 (5.3×) and bulk metadata (a further −48%). Every
  later build is within noise of 6.5 s, because the macOS metadata calls set the time on
  a tree four times the vnode limit.
  The later work shows in peak RSS, 452 to 66 MiB, and user CPU, 2.4 to 1.0 s.
- **The Linux v6.12 source tree:** 543 ms to 163 ms, 3.3× (paired 3.35×,
  [3.24×, 3.39×]). Runtime rises to 430 ms at 0.1.0, when the default command starts
  reading its 358 `.gitignore` files, then falls to 191 ms at 0.2.1 (H162, H163), 177 ms
  at H171, and 163 ms at 0.3.0. Builds before 0.1.0 read no `.gitignore` and do less
  work.

An earlier cell, on the external USB SSD, is kept under `history/superseded/`: there the
kernel and USB path set the wall time after campaign 1. The driver,
`benchmarks.realtree.history`, checks every build’s answer (content totals included)
before timing, and `make perf-report-check` re-derives every cell from its committed run
artifact.

The bottom panel is every experiment’s paired change on its own primary metric: green
for the 52 changes kept at least 3% better, each counted once (47 on wall time, 3 on
peak memory, 2 on a pre-registered component timer), red for changes tried and not kept,
and grey for everything else.
Grey includes 22 accepted records that measure a change already counted, listed with
reasons as `REMEASUREMENTS` in `report_html.py`: cumulative checkpoints, post-merge and
transfer validations, the same candidate measured on a second tree or platform, a
rewrite measured against its own regression, leftover determinations, and screens that
kept no arm or kept the control.
The effects are not multiplied into a runtime: each was measured on its own job and
tree, and compounding the 47 kept wall-time changes would claim about 7,700×.

**Absolute.** Wall time at five cumulative checkpoints on one 60k-entry macOS tree, each
re-measuring the original binary against the code of the day in one interleaved run:

| job | before | after | ratio of medians | spread of the unchanged binary |
| --- | ---: | ---: | ---: | ---: |
| `cold-scan-index` | 628 ms | 290 ms | −54% | 8% |
| `cold-scan-producer` | 959 ms | 457 ms | −52% | 35% |
| `cold-snapshot-save` | 645 ms | 333 ms | −48% | 21% |
| `warm-revalidate` | 804 ms | 442 ms | −45% | 13% |
| `warm-snapshot-load` | 324 ms | 207 ms | −36% | 22% |

“Before” is the median of the five re-measurements of the original binary and “after”
the last checkpoint.
The last column is the range the unchanged binary covered across those five runs, the
scale any step between checkpoints has to be read against; on the producer job it is
wider than several steps.
The paired effects at the final checkpoint are in exp-032.

Beside it, every Linux cell that measured one engine against a later one end to end on
the default tree (exp-194, exp-195, exp-201, exp-202), each row with its own two arms.
Rows are separate sessions on a host that drifted between them, so each row compares
only its own bars.

**Relative.** Every experiment’s paired effect on its primary job with its 95% interval,
against the −3% accept threshold.
A baseline that compares two builds is drawn and tabulated with its change, as the
end-to-end and release cells are (exp-194, exp-195, exp-201, exp-202); a baseline of one
build against itself has none to show.

**Scale.** Cold-scan cost per entry per subject, largest first, from 307 entries to 1.01
million. Milliseconds compare only within a subject; microseconds per entry compare
across them. Generated subjects are drawn apart from real ones, because a generated tree
hides about 15 points of fdu’s distance from the floor
([the floor report](report-2026-08-23-metadata-walk-floor.md)).

**By platform.** Verdict counts and real and generated subject families per platform,
then the accepted changes still in the product whose deciding run measured an
improvement, with that run’s absolute arms on its own subject.

**Mechanism.** Accepted individual changes that moved their primary job at least 5%,
with total CPU beside wall.
Moving work off the critical path and deleting it both shorten wall; only CPU tells them
apart, and a change that deletes CPU keeps its wall saving only where that work was on
the critical path: exp-064’s saving converted to wall at 0.95 on a sparse generated tree
and 0.29 on dense real source (exp-065).

### Paired and Marginal Figures Are Different Numbers

The absolute values are the median of each arm on its own.
The relative values are the median of the *paired* differences, each candidate trial
against the control trial interleaved beside it.
When the host drifts mid-run the two disagree: exp-005’s `cold-scan-index` reads +2.8%
by dividing its medians and −3.9% paired.
The paired figure controls for drift, so every verdict uses it; the page publishes both
and derives neither from the other.
The drift is real: on the 60k subject the reference tool `dust`, whose binary never
changed, measured 210 to 327 ms across eleven runs.

### Accepted and Faster Are Different Sets

The page computes both.
Some real improvements were not accepted: below the threshold, superseded, or decided on
a different pre-registered job (exp-191). Some accepted verdicts claim noninferiority,
instrumentation, or a leftover determination rather than speed; exp-052 and exp-053 were
accepted on intervals of [−3.3%, +3.8%] and [−3.0%, +1.4%] because they bounded what the
counters cost.

## Keeping It Current

After recording an experiment with `make perf-record`, run `make perf-ledger` and
`make perf-report`, and commit the artifact with both generated files.
`make check` re-derives both and fails on drift, and `perf-evidence-check` fails a
record whose headline is not its own measurement.
Stamp a new preparation date only when republishing for readers,
`make perf-report PREPARED=2026-09-29`; the date lives in the projection so regenerating
does not redate the page.
Revise this report when a loop closes or a standing result changes, and give each new
round an entry in [Every Round in Full](#every-round-in-full).
The rules a new figure must respect are in
[the performance loop](../guides/performance-loop.md#publishing-the-evidence).

`explorations/benchmarks/realtree/timeline.py` reads every artifact through the
softschema validator, the path the ledger uses, so an artifact that no longer satisfies
its contract fails the build instead of contributing a wrong row; `report_html.py` draws
the page from the projection without touching an artifact.
Charts are hand-written inline SVG: the shapes are few and known, and a chart library
would have to be pinned and audited for the life of the project.

## What Is Not Measured

- **Cold cache.** Every run is warm-steady; nothing here describes a first read from
  disk.
- **Bare-metal Linux.** Every Linux host is virtualized, which leaves the device-latency
  questions (H28, H73, the cold thread constants) open.
- **Other machines.** macOS is one M1 Pro; Linux is 4-vCPU Xeon guests.
  Which tuning constants that evidence supports is in
  [the platform tuning guide](../guides/platform-tuning.md).
- **macOS since 2026-09-28.** The last macOS record is exp-172; no macOS cell has
  measured the 2026-09-29 round, the pdu track, or the 0.3.0 engine.
- **Windows.** It builds and passes tests; it has not been benchmarked.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
