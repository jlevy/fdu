# The Linux Floor Scoreboard: Automating the Denominator

**Date:** 2026-08-28

**Status:** Instrument landed; first numbers recorded

## Overview

Campaign 2 orders work by each tier’s measured distance to the parallel syscall floor,
which makes that distance the scoreboard.
[The floor report](report-2026-08-23-metadata-walk-floor.md) established it by hand,
once.
`fdu-33ri` asked for `make perf-floor` so that every accepted change re-runs it and
the campaign’s termination criteria become checkable rather than asserted.

This lands the Linux half.
It also reproduces the by-hand report’s headline ratio on an independent host, which is
the first time any of those numbers has been measured twice.

## Why only the Linux half, and why it refuses

`parfloor.c` — the denominator every ×floor threshold in campaign 2 is defined against —
issues `SYS_getdents64` and `statx` directly, and neither has a Darwin equivalent.
The bead recorded this as a design decision rather than an effort estimate, and it is: a
macOS scoreboard needs either a `getattrlistbulk` port of the floor (`fdu-9hdc`) or a
different floor set with the regime difference recorded.

So on a non-Linux host `make perf-floor` exits with that sentence rather than measuring
something else.
Substituting a different denominator while still printing a column headed
`×floor` would make the scoreboard assert something it has not measured, and the bead is
explicit that this is a choice for the plan and not one a harness may make by falling
back.

## What it measures

Three instruments plus the tiers, interleaved, with the shared tally oracle enforced on
every trial:

| Instrument | Role | What it is |
| --- | --- | --- |
| `parfloor stat` | floor | Raw `getdents64` + one `statx` per entry into four accumulators. The denominator. |
| `parfloor enum` | reference | The same walk with the metadata call removed: a search tool’s floor, not a disk-usage tool’s. |
| `arena_spike` | ceiling | An index-shaped result in arena records. What H86 (`fdu-xde5`) is trying to reach. |
| `aggregate` | tier | `perf_probe summary` — five exact tallies, no retained index. |
| `index` | tier | `perf_probe scan-index` — full walk with metadata into a complete index. |

`peerwalk` is deliberately absent: it takes third-party dependencies the shipped crate
does not have, and its README says it is never built by `make`. The ecosystem anchor is
a question for the floor *report*, not for a scoreboard that has to run unattended.

**What is timed is each instrument’s own internal elapsed time, not the spawn wall.**
Process startup, argument parsing and JSON rendering are harness cost, and the gap is
not small — see below.

## The numbers

Host: 4 vCPU x86_64 Linux container, 15 GiB, kernel 6.18.44. 30 trials, 3 warmups,
interleaved, quiet regime (load < 0.05/core at entry).
Commit `b75bf85`.

### `/usr` — 75,976 entries

| Instrument | Role | Median | ×floor | ns/entry | spread | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 17.89 ms | 0.46 | 236 | 1.21 | 15 MiB |
| `parfloor-stat` | **floor** | 39.26 ms | **1.00** | 517 | 1.20 | 15 MiB |
| `aggregate` | tier | 62.21 ms | **1.58** | 819 | 1.34 | 15 MiB |
| `index` | tier | 110.60 ms | **2.82** | 1,456 | 1.46 | 53 MiB |
| `arena-spike` | ceiling | 153.75 ms | 3.92 | 2,024 | **4.25 ⚠** | 15 MiB |

### `/opt` — 47,819 entries

| Instrument | Role | Median | ×floor | ns/entry | spread | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `parfloor-enum` | reference | 17.09 ms | 0.58 | 357 | 1.17 | 15 MiB |
| `parfloor-stat` | **floor** | 29.53 ms | **1.00** | 618 | 1.15 | 15 MiB |
| `arena-spike` | ceiling | 32.30 ms | **1.09** | 676 | 1.17 | 15 MiB |
| `aggregate` | tier | 41.38 ms | **1.40** | 865 | 1.21 | 15 MiB |
| `index` | tier | 86.50 ms | **2.93** | 1,809 | 1.53 | 36 MiB |

## What these say

**The aggregate tier reproduces the by-hand report exactly.** The floor report measured
1.59× on `/usr`; this measures **1.58×** on `/usr`, on different hardware, through an
independently written harness.
That is the first independent reproduction of any ×floor number in this campaign, and it
is worth more than the third decimal place: the denominator, the tier and the arithmetic
all transfer.

**The index tier is where the money is, confirmed.** 2.82× and 2.93× against a 1.40×
threshold, on both subjects, with the widest spread of any tier.
That is H86’s case restated on a third host.

**`arena_spike` is bimodal on the larger subject, and that is new.** On `/opt` it
measures 1.09× — within noise of the 1.06× the floor report recorded, so the ceiling
reproduces too. On `/usr` it splits into a ~63 ms mode and a ~150 ms mode, selected by
how much memory the preceding process churned, and its median is then whichever mode the
run landed in more often.
This matters because H86’s pre-registered targets include peak RSS ≤3× `arena_spike` and
a tail-spread bound: if the ceiling itself has two modes on a subject of this size, both
targets need to name which mode they mean.

The scoreboard flags it rather than quoting it.
`p95/median` — the tail statistic the loop already records — reads a reassuring **1.16**
for that same distribution, because both humps are individually narrow.
`max/min` reads **4.25**. A median describes a distribution with one hump, and the cheap
tell that there are two is the spread, so the harness reports it and marks anything at
or past 2×.

**Harness cost is larger than engine cost on the index tier.** The `index` job’s spawn
wall exceeds its own component timer by **149 ms against a 110 ms component** on `/usr`
— so 57% of a spawn-timed index measurement is not engine work at all.
The probe computes its engine-digest oracle over the whole retained index, which is
exactly the cost `fdu-4xtm` (`--no-oracle` mode) exists to remove.
The loop guide’s warning that the oracle has measured 31.9% of a profile understates it
for this tier. Any index-tier number timed around a process rather than inside one is
measuring the oracle as much as the engine.

## The profile, read twice

The harness-cost finding above is a wall-time measurement on a shared container, so it
was checked against something that does not care how busy the host is: a callgrind
instruction count, which is deterministic.

`valgrind --tool=callgrind` over `perf_probe scan-index --root /usr/include` (4,635
entries, profiling build, 90,639,628 Ir total).
Read flat first:

| Ir | Share | Function |
| ---: | ---: | --- |
| 14,679,236 | 16.20% | `Sha256::compress` (perf_probe.rs) |
| 11,944,960 | 13.18% | `Sha256::compress` (core/intrinsics) |
| 8,622,768 | 9.51% | `Sha256::compress` (core/num/uint_macros) |
| 4,735,734 | 5.22% | `_int_free` (libc) |
| 4,066,880 | 4.49% | `_int_malloc` (libc) |
| 2,836,928 | 3.13% | `Sha256::compress` (core/cmp) |

Then `--tree=caller`:

```
39,129,076 (43.17%)  < perf_probe.rs:<perf_probe::Sha256>::digest (9,332x)
```

**The same cost, one reason.** The flat view splits `Sha256::compress` across four
source attributions — 16.20%, 13.18%, 9.51%, 3.13% — none of which is alarming on its
own and each of which reads as a different line of code.
The caller tree collapses all four into a single caller, `Sha256::digest`, called 9,332
times for **43.17% of the profile**.

This is the failure mode the loop guide warns about, in miniature: a flat profile
attributes cost to a function, not to a reason, and this campaign has already been
misled once by reading only the flat view (`fdu-926e`, where a flat 34% became 11.11%
inclusive). Here it fragments one reason into four modest-looking rows.

And that 43.17% is not the engine.
It is the probe computing its own engine-digest oracle — the independent check that the
scan produced the right answer — over the retained index before exiting.
Two independent instruments now agree the index tier’s measurement is dominated by its
own verification: 57% of spawn wall, 43% of instructions.
The loop guide’s recorded 31.9% is an underestimate for this tier.

The aggregate tier is not affected the same way: its harness overhead is 2.1 ms against
a 62 ms component, about 3%, because the `tallies` oracle is five integers rather than a
multiset hash over every entry.
The two tiers are therefore *unequally* contaminated, which matters beyond each one
separately: a cross-tier reading taken from spawn-timed numbers is skewed by the
difference, not merely inflated by a constant.

## What this does not support

- **No macOS claim.** The denominator does not exist there yet (`fdu-9hdc`).
- **No accept verdict.** This is a ratio of two absolute numbers, not a paired
  comparison; `perf-compare` decides whether a change is kept and this does not.
- **No peer ranking.** `peerwalk` is not run here.
- **Neither subject is nominated.** `/usr` and `/opt` are the dense real trees this
  container has. `/usr` clears the deciding bar on size (75,976 ≥ 50,000); `/opt` does
  not (47,819) and screens.
  Neither is in a committed nominations document, because the nominations file is
  per-host and gitignored and this host is ephemeral.
  The ratios are reproducible from any Linux host with the same command; the absolute
  milliseconds are not.
