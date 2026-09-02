# The Linux floor cell for H86, and why the floor gates reject

2026-09-02. Linux, 4-core KVM Intel Xeon @ 2.80 GHz, 15 GiB RAM, ext4, virtualized.
Subject: the generated `balanced` recipe at 450,001 entries (56,251 directories, 393,750
files), engine digest `e6da0498…`.

This note records the two denominators that
[exp-102](../experiments/exp-102-h86-linux-evidence-stage-relative-gates-pass-floor-gates-fai.md)
rejects against, because they are reusable beyond that one verdict and because a floor
claim is only as good as the cell its denominator was measured in.

## Why two denominators and not one

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

## Preparation, chosen before any timing was observed

Both cells use the pre-registered low-churn warm-steady preparation: three complete
warmups of the same binary immediately before every retained sample, with no full-index
builder and no deliberate memory-churn process between the last warmup and the sample.
Twelve samples were retained for each.
Nothing was partitioned after the fact, and no timing cluster was selected after the
run.

Tallies were checked against the independent oracle before either cell was used as a
denominator. Both report `files=393750`, `bytes=358665192` and `allocated=1344430080`
exactly; both report `dirs=56250` against the oracle’s 56,251, which is the documented
off-by-one where the root is not its own child.

## The cells

| Cell | Wall median | `p95/median` | `max/min` | Peak RSS median |
| --- | --- | --- | --- | --- |
| `parfloor stat`, 4 workers | 316.4 ms | 1.231 | 1.391 | 10.7 MiB |
| `arena_spike`, 4 workers | 362.8 ms | 1.058 | 1.204 | 30.5 MiB |

Both are stable. This matters procedurally, not just aesthetically: the campaign plan
says that if the prepared `arena_spike` cell has `max/min` above 2.0, its floor and RSS
ratios are reported as unresolved and **cannot accept or reject H86**. At 1.204 the
escape hatch does not apply, so the ratios below are resolved and the rejection stands
on them.

## What the candidate measures against them

Candidate `5d7b86f`, immediate control `c6380f7`, twelve paired interleaved trials, zero
invalid samples.

| Job | Variant | Wall | x syscall floor | Peak RSS | x spike RSS |
| --- | --- | --- | --- | --- | --- |
| `cold-scan-index` | control | 1,905.6 ms | 6.02 | 303.6 MiB | 9.96 |
| `cold-scan-index` | candidate | 1,537.0 ms | 4.86 | 153.5 MiB | 5.03 |
| `default-tree` | control | 1,189.7 ms | 3.76 | 313.6 MiB | 10.28 |
| `default-tree` | candidate | 821.7 ms | 2.60 | 200.9 MiB | 6.59 |

The gates are 1.4x on index wall and 3x on peak RSS. Both fail on both jobs.
H86 moved them substantially and did not reach them.

## The part worth reusing

`parfloor` is 316 ms and `arena_spike` is 363 ms.
Retaining an index-shaped result over raw parallel enumeration therefore costs about 15%
on this tree — roughly 46 ms on top of a 316 ms syscall bill.
The candidate’s `default-tree` is 822 ms.

So the remaining Linux gap is about 2.6x, essentially all of it consumer-side, and none
of it in the syscall layer.
That is worth stating plainly because the campaign has repeatedly been drawn back to
enumeration strategies — inode ordering, queue depth, `io_uring` batching — and this
cell says the enumeration layer is already close to spent on a warm VM. `walkspike`’s
variant ranking remains the right tool for the cold and bare-metal regimes, which this
cell does not address.

## What this cell is not

It is a shared cloud KVM with an agent process resident, recorded as `exploratory` stage
and `uncontrolled` host regime.
It is strong enough to reject an absolute floor claim, because that claim is a ratio
against denominators measured on the same host in the same session and both denominators
are tight. It is not a quiet-host verdict and does not substitute for one.

Per [platform-tuning.md](../guides/platform-tuning.md), inode-ordered statting and any
queue-depth claim cannot be settled on a VM, and nothing here attempts to.
This note makes no bare-metal claim and adopts no constant.

## Raw samples

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
