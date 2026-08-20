# fdu Performance Evidence: Absolute and Relative

A charted view of the 64 experiment artifacts in
[docs/project/experiments/](../experiments/), covering both what the tool costs in
milliseconds and what each change did to that cost.

Read it at [performance-evidence/index.html](performance-evidence/index.html).
Regenerate it with `make perf-report`; it is built from the artifacts and must never be
edited by hand.

[The ledger](report-2026-08-10-fdu-performance-experiments.md) remains the
per-experiment record and the place to read one experiment in full.
This is the view across all of them.

## Why this exists alongside the ledger

The ledger reports every experiment as a percentage, which is the right unit for
deciding whether a change earned its place and the wrong one for answering how fast the
tool is. A reader could finish it knowing that cold scan improved 54% without ever
learning that the number moved from 628 ms to 290 ms.

The two questions also need different evidence, and conflating them is the specific way
this data can be read into a wrong answer.

## What each figure answers

**Absolute.** Wall time in milliseconds at five cumulative checkpoints on one macOS
tree. Each checkpoint re-measured the original pre-work binary against the code of the
day, in one interleaved run, so every before-and-after pair comes from a single sitting
rather than from two numbers taken days apart.

| job | before | after | change | spread across the five re-measurements |
| --- | ---: | ---: | ---: | ---: |
| `cold-scan-index` | 628 ms | 290 ms | -54% | 8% |
| `cold-scan-producer` | 959 ms | 457 ms | -52% | 35% |
| `cold-snapshot-save` | 645 ms | 333 ms | -48% | 21% |
| `warm-revalidate` | 804 ms | 442 ms | -45% | 13% |
| `warm-snapshot-load` | 324 ms | 207 ms | -36% | 22% |

The last column is the load-bearing one.
It is the range the *unchanged* binary itself covered across those five runs, and it is
the scale any movement between checkpoints has to be read against.
On the producer job it is 35%, which is wider than several of the steps.

**Relative.** Every experiment’s paired effect on its primary job with its 95% interval,
sorted by effect, against the -3% accept threshold.
The axis is sized to the data and nothing is clipped.

**Scale.** Cost per entry per subject, largest tree first.
Milliseconds are only comparable against the tree that produced them and this record
spans 307 entries to 1.01 million, so the check that the speed-up is not a small-tree
artifact has to be made in normalised units.

**Mechanism.** The fifteen individual accepted changes that moved their primary job at
least 5%, with total CPU beside wall time.
Both moving work off the critical path and deleting it shorten wall time, and only CPU
distinguishes them; twelve of the fifteen deleted work outright.

## The trap this report is built to avoid

An absolute figure and a relative figure on one page invite the reader to divide the
first into the second.
That is wrong here.

The absolute values are the median of each arm on its own.
The relative values are the median of the *paired* differences, each candidate trial
against the control trial interleaved beside it.
When the host drifts mid-run the two diverge: across this record they differ by more
than two percentage points on 21% of measurements, and sometimes differ in sign.
exp-005’s `cold-scan-index` reads +2.8% by dividing its medians and -3.9% paired.

The paired figure is the one that controls for drift, so it is the one every verdict
used. Both are published and neither is derived from the other.
That the host drift is real rather than theoretical is visible in the record without any
fdu build involved: on the 60k subject the reference tool `dust`, whose binary never
changed, measured between 210 ms and 327 ms across eleven runs.

## Two things the counts do not say on their own

Thirty-one changes were kept and thirty-one measured an improvement whose interval
excluded zero, and those are not the same thirty-one.

Five improved and were not kept — below the threshold, superseded, or unfinished.
Five were kept although their interval crossed zero, because what they bought was not
speed. exp-052 and exp-053 are the clearest: instrumentation accepted on intervals of
[-3.3%, +3.8%] and [-3.0%, +1.4%], which is a claim that the cost is undetectable, not
that anything got faster.

## How it is built

`benchmarks/realtree/timeline.py` reads every artifact through the softschema validator
— the same path the ledger uses, so an artifact that no longer satisfies its contract
fails the build rather than quietly contributing a wrong row — and projects it into
`performance-evidence/timeline.json`. `benchmarks/realtree/report_html.py` draws the
page from that projection and touches no artifact.

The projection is committed so a reviewer can diff what the page claims rather than only
how it looks. Charts are hand-written inline SVG for the same reason the crate’s
serializers are hand-written: the shapes are few and fully known, and a chart library
would have to be pinned, audited, and carried for the life of the project to draw four
figures.

## What is not measured here

Every number is one Apple M1 Pro and one Linux VM, with the page cache warm throughout
because dropping it needs root.
Nothing here describes a genuinely cold disk.
Which tuning constants that evidence actually supports, and which are inherited without
it, is in [the platform tuning guide](../guides/platform-tuning.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
