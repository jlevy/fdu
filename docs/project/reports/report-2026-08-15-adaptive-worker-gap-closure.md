# Adaptive Worker Scaling: What the Model Proves and What Hardware Must Still Decide

**Date:** 2026-08-15

**Author:** fdu project, with Claude Code assistance

**Status:** Current

## Summary

fdu’s automatic scan mode decides once whether to unlock its reserve workers, using the
first 16,384 entries that *finish*. This report shows, deterministically and on any
platform, that the decision is a function of completion order rather than of the tree:
the same directory tree, with the same total work, reaches opposite decisions depending
on which chunks happen to complete first.

That is the defect [`fdu-8evu`](#beads) was filed against.
It is now characterized rather than argued about, and the characterization is a test
that runs in CI on every platform instead of a stopwatch reading on one host.

**No production behavior changed.** Selecting a replacement controller requires the
held-out Apple Silicon and local-APFS confirmation the epic pre-registered, and that
measurement has not been run.
Under the epic’s own decision rules a documented no-change outcome is a valid result,
and this is one. What did change is observability: the policy now records its own
history, and a walk that never measured anything says so instead of looking like a walk
that measured and chose to hold.

## Who this is for

Anyone picking up the adaptive-worker workstream, and anyone tempted to retune
`ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY` or replace automatic mode with a fixed thread
count. Read [the design principles](../architecture/fdu-design-principles.md) first; the
rule that governs everything below is that a measurement is evidence about its own
regime.

## 1. The defect

`DirectoryQueue::release` in [`scan.rs`](../../../crates/fdu/src/scan.rs) folds each
completed chunk’s entry count and worker time into one shared calibration.
When the accumulated entries first reach 16,384, it compares mean per-entry service time
against a 30 µs threshold, expands the worker pool if the filesystem looks slow, and
then sets the calibration to `None` — permanently.

Two properties follow, and neither is a tuning question:

1. **The decision is order-sensitive.** Chunks complete in whatever order the filesystem
   and the workers produce them, which on a heterogeneous tree has nothing to do with
   traversal order. A prefix drawn from a shallow, cache-warm region and a prefix drawn
   from a deep, cold one describe the same tree and disagree about it.

2. **The decision is never revisited.** A tree whose slow phase begins after the window
   closes runs its entire slow phase on the starting pool.
   The evidence that would change the answer arrives, and there is nothing left to
   receive it.

The field report that opened this workstream — a partial, heterogeneous
`Application Support` scan producing materially different effective concurrency between
runs — is what both properties predict.

## 2. What the model proves

The characterization lives in `scan::tests::completion_order` and drives the shipped
`WorkerCalibration` directly, so it cannot drift from the policy it describes.
Each test replays an explicit completion order — a list of chunks, each with an entry
count and a worker-time cost — through the real decision code.

The load-bearing case holds the tree constant and varies only completion order:

| Completion order | Whole-walk cost | Shipped decision |
| --- | ---: | --- |
| Fast phase completes first | 46 µs/entry | **hold** the initial pool |
| Fast and slow interleaved | 46 µs/entry | **scale up** |

Both traces contain the same four fast chunks and four slow chunks.
Both are latency-bound walks by the policy’s own 30 µs threshold.
The policy answers one of them wrongly by its own criterion, and which one depends on
scheduling.

A second test extends the slow phase to 400 chunks: 1% of the walk decides the worker
policy for the other 99%, and the walk-wide cost of 89 µs/entry — three times the
trigger — never reaches the decision.

These are deterministic facts about the algorithm.
They need no host, no APFS, and no quiet machine, which is exactly why they belong in
the test suite rather than in a benchmark run.

## 3. What the model cannot decide

The model says the current policy answers an ill-posed question.
It does not say what the right answer is worth.

A trailing-window candidate is screened alongside the shipped policy in the same test
module.
It reaches one verdict for both completion orders above and detects the late slow
phase two chunks after it begins.
That is evidence about *order-robustness* only.
It is not evidence that the candidate is faster, and the following are all unmeasured:

- wall-clock effect on any real tree;
- the cost of re-evaluating the window per chunk release against the current single
  comparison;
- interaction with the macOS `getattrlistbulk` path and its fallback;
- whether 30 µs remains the right threshold under a window that slides;
- behavior on Intel Macs, on non-APFS volumes, and on Linux, where the warm floor is
  about 1.5 µs per entry and the trigger may never fire at all.

The candidate is therefore screening output, kept in test code and absent from the
shipped walker. Promoting it requires the pre-registered held-out matrix: screening and
confirmation on disjoint samples, a +3% paired noninferiority bound whose interval upper
bound stays at or below +3%, and pre-registered resource thresholds.

## 4. The evidence boundary

Every claim in this report is either deterministic or explicitly absent.
No timing claim is made, because no timing measurement was taken.

The work was carried out on virtualized Linux x86_64 with 4 vCPUs on ext4. That regime
cannot produce claim-grade evidence for this epic for three independent reasons, any one
of which is sufficient:

- **Wrong filesystem.** The 30 µs threshold and the 16,384-entry window were measured on
  APFS. ext4’s warm service time is roughly twenty times below the trigger.
- **Wrong hardware.** The epic’s claims are scoped to Apple Silicon.
  Heterogeneous performance and efficiency cores are part of what the reserve interacts
  with, and an x86_64 Xeon does not model them.
- **Wrong host class.** A shared virtualized runner cannot support a paired comparison
  at 3% resolution. This is the same reason no timing gate runs in `make check`.

GitHub’s `macos-latest` runners do not close this gap.
They are shared, virtualized, and unquiet; a timing gate there measures the runner.

So the ledger gains no entry from this work.
Every entry in [the experiment ledger](report-2026-08-10-fdu-performance-experiments.md)
carries a control, a candidate, and a paired confidence interval, and an entry without
those would be the shape of evidence without the substance.
The correct record for a workstream that produced no measurement is a report saying so.

## 5. What shipped

Three changes, none of which alters a scan’s behavior:

- **Policy history in the artifacts.** A new `adaptive scan policy` counter group
  records the chunks, entries, and worker microseconds the calibration actually
  consumed, plus the reserve expansions performed.
  Recording is off by default and gated behind the existing `FDU_COUNTERS` toggle; the
  counter update happens outside the queue lock so a disabled counter cannot lengthen
  the critical section it observes.

- **Failing closed on an unobservable policy.** A walk that ends before its window fills
  now increments `walks left undecided`. Previously such a walk was indistinguishable in
  the artifacts from one that measured the filesystem and chose to hold the pool — an
  absence of evidence reading as a decision.

- **The characterization tests** described above, which pin the current behavior and
  will fail loudly if a future change alters it silently.

## 6. What remains

The following are open and cannot be closed from a non-Apple-Silicon host.
They are stated as prerequisites rather than as work items so the next person does not
mistake a rerun of the model for progress on them:

| Prerequisite | Why it needs hardware |
| --- | --- |
| Profile the frozen reproduction | The diagnosis in §1 is analytical; a profile on the failing regime is what confirms or falsifies it |
| Backend and topology signals | `getattrlistbulk`, its fallback, and APFS directory topology exist only on macOS |
| Apple Silicon hardware bounds | Performance and efficiency core behavior under interactive-host pressure |
| Controller screening confirmation | Screening is done; confirmation needs disjoint held-out samples on the target regime |
| Release CLI matrix against dust | A claim-grade comparison needs a clean installed binary on a quiet Mac |

Until those run, the shipped policy stands as-is: characterized, instrumented, and
unchanged.

## Beads

This report closes the documentation and observability portion of epic `fdu-5rpt` —
Close the adaptive-worker evidence gap on Apple Silicon/APFS. The conditional
implementation bead `fdu-8evu` is resolved with **no production behavior change**, which
its acceptance criteria admit as a valid outcome when no candidate is independently
confirmed.

## References

| Document | What it covers |
| --- | --- |
| [Design principles](../architecture/fdu-design-principles.md) | Why a measurement is evidence only about its own regime |
| [Performance loop](../guides/performance-loop.md) | The protocol a controller change would have to pass |
| [Platform tuning](../guides/platform-tuning.md) | Which constant rests on which measurement |
| [Experiment ledger](report-2026-08-10-fdu-performance-experiments.md) | Every timing verdict, including the rejections |
| [Instrumentation playbook](../guides/performance-instrumentation-playbook.md) | How to instrument without distorting the measurement |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
