# The Linux Floor Cell for H86, and Why the Floor Gates Reject

2026-09-02. Linux, 4-core KVM Intel Xeon @ 2.80 GHz, 15 GiB RAM, ext4, virtualized.
Subject: the generated `balanced` recipe at 450,001 entries (56,251 directories, 393,750
files), engine digest `e6da0498…`.

This note records the two denominators that
[exp-103](../experiments/exp-103-h86-linux-evidence-stage-relative-gates-pass-floor-gates-fai.md)
rejects against, because they are reusable beyond that one verdict and because a floor
claim is only as good as the cell its denominator was measured in.

## Why Two Denominators and Not One

The campaign’s Linux gate names two different floors, and conflating them is the easiest
way to get this wrong.

`parfloor stat` is the **parallel syscall floor**: N workers over a shared directory
queue doing raw `getdents64` plus one `statx` per entry, retaining nothing.
It answers “what does the kernel cost on this tree at this worker count”, and it is the
denominator for the 1.4x index-wall gate.

`arena_spike` is the **consumer-side floor**: the same syscall load and worker count,
retaining an index-shaped result in worker-local arenas with one bottom-up roll-up.
It answers “what does keeping the answer cost on top of that”, and it is the denominator
for the 3x peak-RSS gate.

## Preparation, Chosen Before Any Timing Was Observed

Both cells use the pre-registered low-churn warm-steady preparation: three complete
warmups of the same binary immediately before every retained sample, with no full-index
builder and no deliberate memory-churn process between the last warmup and the sample.
Twelve samples were retained for each, at four workers, with wall time and peak RSS
recorded per sample and CPU time not recorded.
Nothing was partitioned after the fact, and no timing cluster was selected after the
run.

Tallies were checked against the independent oracle before either cell was used as a
denominator. Both report `files=393750`, `bytes=358665192` and `allocated=1344430080`
exactly; both report `dirs=56250` against the oracle’s 56,251, which is the documented
off-by-one where the root is not its own child.

## The Cells

| Cell | Wall median | `p95/median` | `max/min` | Peak RSS median |
| --- | --- | --- | --- | --- |
| `parfloor stat`, 4 workers | 316.4 ms | 1.231 | 1.391 | 10.7 MiB |
| `arena_spike`, 4 workers | 362.8 ms | 1.058 | 1.204 | 30.5 MiB |

Both are stable. This matters procedurally, not just aesthetically: the campaign plan
says that if the prepared `arena_spike` cell has `max/min` above 2.0, its floor and RSS
ratios are reported as unresolved and **cannot accept or reject H86**. At 1.204 the
escape hatch does not apply, so the ratios below are resolved and the rejection stands
on them.

## What the Candidate Measures against Them

Candidate `5d7b86f`, immediate control `c6380f7`, twelve paired interleaved trials, zero
invalid samples. The index tier is `default-tree`, the job the campaign-2 plan and the
floor report measure it with, so it carries the verdict and `cold-scan-index` supports
it.

| Job | Variant | Wall | x syscall floor | Peak RSS | x spike RSS |
| --- | --- | --- | --- | --- | --- |
| `default-tree` | control | 1,189.7 ms | 3.76 | 313.6 MiB | 10.28 |
| `default-tree` | candidate | 821.7 ms | 2.60 | 200.9 MiB | 6.59 |
| `cold-scan-index` | control | 1,905.6 ms | 6.02 | 303.6 MiB | 9.96 |
| `cold-scan-index` | candidate | 1,537.0 ms | 4.86 | 153.5 MiB | 5.03 |

The gates are 1.4x on index wall and 3x on peak RSS. Both fail on both jobs.
H86 moved them substantially and did not reach them.
The experiment artifact records how these binaries differ from what #52 ships, and why
none of the differences is a route to the gates.

## What the Cells Say about the Residual, and What They Cannot

`parfloor` is 316 ms and `arena_spike` is 363 ms.
Retaining an index-shaped result over raw parallel enumeration therefore costs about 15%
of wall time on this tree, roughly 46 ms on top of a 316 ms syscall bill.
The candidate’s `default-tree` is 822 ms: 2.60x the floor in total, 505 ms or 1.60x of
the floor above it, and 2.26x `arena_spike`.

That points the residual at consumer-side work.
[The Linux first measurements](research-2026-08-13-linux-first-measurements.md) point
the same way: their strace census found fdu issuing the same `statx`, `getdents64`, and
`openat` counts as `dut` and `diskus`. It matters because the campaign has repeatedly
been drawn back to enumeration strategies such as inode ordering, queue depth, and
`io_uring` batching; the census and this cell’s 15% spike-over-floor gap both point away
from them, though neither rules them out.

This cell does not locate the residual.
Neither floor cell recorded CPU time, and the candidate’s `default-tree` spends about
64% of its CPU in the kernel (median system CPU 1,347 ms of 2,120 ms).
Without the floors’ own kernel time, the cell cannot say how much of the 505 ms is
kernel work beyond the floor’s and how much is user-space consumer work.
A floor cell that records per-sample CPU for both tools, beside wall time and RSS, would
settle it. `walkspike`’s variant ranking remains the right tool for the cold and
bare-metal regimes, which this cell does not address.

## Deviations from the Pre-Registration

The H86 pre-registration in
[the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md#h86-preregistration-one-decision-two-evidence-stages)
is followed with three exceptions.

- **The aggregate gate was not evaluated.** The Linux stage also requires aggregate wall
  at or below 1.25x the floor on the nominated real subjects.
  This stage measured no real subject and no aggregate job, so that gate is still open.
- **The subject is not the primary one.** The pre-registered primary 450k Linux subject
  is the 450,463-entry `gen_tree.py` tree; this stage used a new 450,001-entry tree from
  the corpus generator’s `balanced` recipe.
  Both are generated, and the campaign-2 plan’s corpus rule found a generated corpus
  understating fdu’s distance from the floor, so neither subject favors rejection.
- **fdu’s worker count was neither pinned nor recorded.** The harness passes no
  `--threads`, so fdu ran its automatic policy, which on a 4-core host starts four
  workers, the same count as the floor tools.
  That match holds only because the host has fewer cores than fdu’s six-worker cap: on
  an 8- or 16-core host the same protocol would compare six fdu workers against an
  N-worker floor. The policy’s adaptive reserve can add workers beyond the initial four,
  and whether it did here was not recorded either.
  A future floor cell should pin fdu’s `--threads` to the floor tools’ worker count and
  record it.

## What This Cell Is Not

It is a shared cloud KVM with an agent process resident, run as an `exploratory` stage
under an `uncontrolled` host-pressure regime.
The experiment schema has no host-pressure field, so that regime, and the statement that
both floor cells ran on the same host in the same session as the fdu arms, are the
operator’s account rather than recorded values.
[The performance loop](../guides/performance-loop.md) limits an uncontrolled run to
exploration and discovery, so the rejection rests on its margins rather than on the
regime. Against the slowest retained `parfloor` sample, 424.4 ms, the candidate’s
`default-tree` is still 1.94x the floor, and against the largest `arena_spike` RSS
sample, 30.6 MiB, both jobs stay above 5x; the experiment artifact works through how far
a candidate trial would have to stray to change that.

It is not a quiet-host verdict and does not substitute for one.
Per [platform-tuning.md](../guides/platform-tuning.md), inode-ordered statting and any
queue-depth claim cannot be settled on a VM, and nothing here attempts to.
This note makes no bare-metal claim and adopts no constant.

## Missing Records

The pre-registration requires all raw samples, `p95/median`, and `max/min` for every
arm. The floor cells meet it, and their samples follow.
The fdu arms do not: their run JSON lived only on the measurement VM, which no longer
exists, and was never committed, so no per-trial sample survives for either arm of any
job. The experiment artifact keeps what was derived from it and says what that leaves
unverified. Recording `max/min`, and committing each run’s JSON beside its artifact, is
tracked as `fdu-c4jr`.

## Raw Samples

### `arena_spike` retained samples

| Sample | Wall (ms) | Peak RSS (MiB) |
| --- | --- | --- |
| 1 | 359.8 | 30.5 |
| 2 | 362.3 | 30.3 |
| 3 | 344.9 | 30.5 |
| 4 | 358.6 | 30.5 |
| 5 | 348.8 | 30.6 |
| 6 | 415.4 | 30.3 |
| 7 | 370.6 | 30.5 |
| 8 | 382.2 | 30.5 |
| 9 | 372.8 | 30.5 |
| 10 | 383.8 | 30.5 |
| 11 | 363.3 | 30.5 |
| 12 | 356.5 | 30.4 |

### `parfloor stat` retained samples

| Sample | Wall (ms) | Peak RSS (MiB) |
| --- | --- | --- |
| 1 | 305.2 | 10.7 |
| 2 | 315.6 | 10.7 |
| 3 | 314.0 | 10.7 |
| 4 | 309.9 | 10.7 |
| 5 | 317.2 | 10.7 |
| 6 | 309.7 | 10.7 |
| 7 | 310.5 | 10.7 |
| 8 | 389.3 | 10.7 |
| 9 | 366.0 | 10.7 |
| 10 | 367.6 | 10.7 |
| 11 | 424.4 | 10.7 |
| 12 | 385.9 | 10.7 |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
