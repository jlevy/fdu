# fdu Performance Evidence

**Revised:** 2026-09-29, covering exp-000 through exp-191: 177 artifacts, since exp-113,
exp-168–169, and exp-175–186 are unused ids.

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
loop found, which results carry qualifications, and what is open.
The protocol is [the performance loop](../guides/performance-loop.md); the next action
is in [Current Pickup](../guides/performance-loop-runbook.md#current-pickup-2026-09-27);
the peer-tool rankings are in the tool comparisons
[on macOS](report-2026-09-26-fdu-live-tool-comparison.md) and
[on Linux](report-2026-09-27-fdu-linux-tool-comparison.md).

## Where Each Platform Stands

A number is evidence about the platform, host, tree, and cache state it was measured on.
Every figure below names them.
**Quiet** means the host held the loop’s one-second CPU-busy gate (at most 25%) for the
whole cell (Linux cells before 2026-09-27 used load per core at most 0.25);
**uncontrolled** means it did not, so the cell is exploration rather than a claim.
All runs are warm-steady: dropping the page cache needs root, and nothing here describes
a cold disk.

### macOS

137 of the 177 experiments ran on one Apple M1 Pro (10 cores, 32 GiB), bare metal, APFS,
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
  H159’s bounded form is wall and RSS neutral here
  ([exp-167](../experiments/exp-167-macos-h159-bounded-listing-recycle-is-rss-and-wall-neutral-l.md)).
  H162 and H163 have not been measured on macOS.

### Linux

Virtualized 4-core guests, Intel Xeon and ext4 where recorded: a KVM host on Linux
6.12.94+ for exp-138–155, and Firecracker guests on 6.18.x for the cells before and
after. Six kernel builds, 40 experiments, counted in
[the ledger’s regime table](report-2026-08-10-fdu-performance-experiments.md#regime-coverage).
Most Linux cells since 2026-09-20 are quiet.

- **Installed command, generated tree.** On the same 1,000,001-entry tree,
  `fdu --no-gitignore --view summary` was the fastest tool measured at 0.94 s (pdu +8%,
  diskus +12%). Building the index took 1.25 s, about 23% longer than pdu and 21% longer
  than diskus ([Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md),
  2026-09-28, quiet, measured before H159, H161, H162, and H163).
- **Default command, real source tree.** On Linux v6.12 (92,474 entries, 358
  `.gitignore` files) the default tree fell from 590 to 211 ms and the default summary
  from 505 to 167 ms
  ([exp-173](../experiments/exp-173-linux-h162-allocation-free-gitignore-matching-halves-the-def.md),
  [exp-174](../experiments/exp-174-linux-h163-per-listing-control-chains-cut-another-third-from.md),
  quiet probe). With `.gitignore` off the same binary takes 81 and 71 ms; pdu takes 70 ms
  (screen). What remains is classification on one thread.
- **Directory-dense trees.** Returning drained listings to the walker that allocated
  them (H159) cut the default tree 8.6% on a real 80k-entry `node_modules` tree
  ([exp-190](../experiments/exp-190-linux-h159-listing-recycle-clears-3-percent-on-a-real-direct.md),
  quiet) and 10.6% on the generated tree.
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
[#155](https://github.com/jlevy/fdu/pull/155)). The peer comparisons have not been
re-measured on 0.2.1. 0.2.2 is planned around H171 and H172 (see
[Open Work](#open-work)).

## Standing Results by Tier

The latest figure each tier holds, from the arm that stayed in the product.
Absolute values compare only within one row: each is its own subject on its own host.

| Tier and job | Platform | Subject | Latest | Regime | Source |
| --- | --- | --- | --- | --- | --- |
| CLI indexed tree, `--cache off` | macOS | generated, 1.0M entries | 6.4 s; dumac +9% | uncontrolled | [macOS comparison](report-2026-09-26-fdu-live-tool-comparison.md) |
| CLI indexed tree, `--cache off` | Linux | generated, 1.0M | 1.25 s; pdu 1.02 s | quiet | [Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md) |
| CLI summary, `--no-gitignore` | Linux | generated, 1.0M | 0.94 s; fastest measured | quiet | [Linux comparison](report-2026-09-27-fdu-linux-tool-comparison.md) |
| Default tree, `.gitignore` on | Linux | `linux-v6.12`, 92k | 211 ms, from 590 | quiet | exp-173, exp-174 |
| Default summary, `.gitignore` on | Linux | `linux-v6.12`, 92k | 167 ms, from 505 | quiet | exp-173, exp-174 |
| Default summary, peak RSS | macOS | metabrowser, 137k | 11.0 MiB, from 35.9 | uncontrolled | exp-170 |
| Default tree | Linux | `node-modules-dense`, 80k | 93 ms, from 106 | quiet | exp-190 |
| `content-cache-hit` | macOS | metabrowser, ~146k | 778 ms, from 1,218 | uncontrolled | exp-108 to exp-132 |
| `content-cache-hit` | Linux | `linux-v6.12`, 92k | ~588 ms | quiet | exp-155 |
| `content-query`, 100 reports | macOS | metabrowser, 137k | 20.6 s, from 38.6; provisional | uncontrolled | exp-159 |
| Index tier ×floor | Linux | generated, 450k | 1.78× (gate 1.4×) | uncontrolled | exp-141 |
| Cold scan, campaign 1 | macOS | metabrowser, 60k | 290 ms, from 635 | not recorded | [exp-032](../experiments/exp-032-cumulative-effect-through-bounded-parallel-reconciliation.md) |

The campaign-1 row is the last time the original pre-work binary was re-measured against
the code of the day; no later checkpoint repeats that comparison, so it does not
describe the current engine.

## The Loops in Order

Each row is one line of inquiry.
The ledger has every verdict and
[the campaign status report](report-2026-08-14-performance-campaign-status.md) the
history through 2026-08-23.

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

**The index-tier gap on the generated tree is user space.** fdu, pdu, and diskus make
the same system calls, and fdu spends less kernel time; it spends more user time and its
walkers park on a shared queue (14,668 voluntary context switches against pdu’s 17).
Under a preloaded jemalloc the unchanged binary ran level with pdu (screen), which
located the cost in allocations freed across threads.
In the same screen H159’s structural fix recovered about half of that allocator cost;
H166, H167, and H170 are the proposed next steps.

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

## Qualifications on Current Results

These do not overturn a verdict; they say what a verdict rests on.

- **H156 and H160 were accepted on a generated tree only.** The loop asks for a
  nominated real tree in any accept set, and exp-160 and exp-163 ran only on the
  generated balanced tree, as did the Linux peer tables.
  The page’s per-platform section counts 7 of 16 Linux improvements decided on generated
  trees.
- **H161’s Linux verdict met one of its two bars.** It was accepted on wall; its 50% RSS
  bar was missed on the deciding subject, where classification holds about 20 MiB more
  than a walk without it (`fdu-nyj8`).
- **H159’s deciding subject changed after two misses.** The dense tree was registered
  before exp-190 ran and fits the per-directory mechanism, but it was chosen after the
  first subject had failed.
  exp-167, the macOS cell, is now resolved as rejected with the change kept, the
  encoding exp-164 and exp-165 use for H156’s and H160’s macOS cells.
- **The leftover determinations on Linux v6.12 predate the `.gitignore` finding.** H140
  (exp-139) and H146
  ([exp-147](../experiments/exp-147-linux-first-run-leftover-is-still-the-walk.md))
  attributed the default command to the walk; exp-173 shows most of it was
  classification. Both registry rows now carry that caveat, and a new determination needs
  a `--no-controls` arm.
- **Every macOS accept since 0.1.0 is uncontrolled.** They stand as paired evidence on a
  busy host; none has a quiet replication.
- **The Linux floor scoreboard is stale.** It was last derived on 2026-09-20 (exp-141).
  H147, H72, H156, H160, H161, H159, H162, and H163 have landed or been accepted since,
  and the loop re-derives the ×floor after an accepted change before trusting the queue.

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
  H165 revisits this now that classification is cheaper.
- **Cutting counts off the critical path.** Fewer allocations (H157, H114) and fewer
  wakes (H158) did not move wall where the saved work was not what the clock waited on.
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

The `.gitignore`-on default command is not a campaign-2 tier, and it now holds the
largest measured Linux gap: 211 ms against 81 ms with `.gitignore` off.

## Open Work

Grouped by topic; the order to run them in is
[Current Pickup](../guides/performance-loop-runbook.md#current-pickup-2026-09-27) and
the pdu brief’s recommendations.

- **0.2.2 Linux parity** (epic `fdu-8a8r`,
  [the plan](../specs/active/plan-2026-09-29-linux-parity-0.2.2.md)):
  - the `.gitignore` matching survey (`fdu-p6vc`), first;
  - **H171** (`fdu-sdul`), bucketed matching, for the default command on real trees;
  - **H172** (`fdu-dp98`), an exact transient tree tier, for the indexed gap on the
    generated tree;
  - **H173**, the live residual set, only if H171’s counters call for it.
- **H164** (`fdu-emqf`): classify `.gitignore` on the walker threads, now sequenced
  after H171 for whatever residual remains.
- **Confirm on real trees:** H156 and H160 (`fdu-b9ga`); H162 and H163 on macOS
  (`fdu-dv07`); the Linux ×floor scoreboard (`fdu-z2h6`).
- **H165** (`fdu-c11z`): revisit the Linux walker count once classification no longer
  binds the consumer.
- **H166** (`fdu-i6nk`) and **H167** (`fdu-lwsy`): a walk queue that does not park
  walkers, and directory tokens in place of the path-keyed directory map, the index-tier
  pair for the generated tree.
- **H168** (`fdu-mw2s`), **H169** (`fdu-leja`), **H170** (`fdu-lz25`): no per-file
  extension `String`; directory opens relative to the parent descriptor; a per-worker
  summary fold.
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

**Relative.** Every experiment’s paired effect on its primary job with its 95% interval,
against the −3% accept threshold.

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
Revise this report when a loop closes or a standing result changes.
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
- **Windows.** It builds and passes tests; it has not been benchmarked.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
