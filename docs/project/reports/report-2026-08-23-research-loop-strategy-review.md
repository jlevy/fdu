# Research Loop Strategy Review: Running Campaign 2 Unattended on macOS

**Date:** 2026-08-23

**Status:** Review. Decision input for the campaign-2 plan, which owns prioritization;
the ordering proposed here is written into that plan’s macOS agenda section once
adopted.

## Overview

The question this review answers: after PRs #38 through #45, does the performance loop
and the campaign-2 strategy make sense for an agent to iterate overnight, unattended, on
this host — an M1 Pro on APFS, which is where the project’s performance bar is set?

The measurement machinery is ready.
The strategy was not yet pointed at this host, the instructions existed as an 844-line
protocol rather than a runbook, and the tracker disagreed with the record in six places
that `tbd ready` would have handed an agent as work.
Three things had to change before an unattended run was worth its tokens, and two of
them are done in this branch:

1. **A macOS ordering of the campaign.** The plan’s phases are anchored on a Linux floor
   that cannot be computed here, and its Phase A is Linux-only except for two items.
   [The macOS agenda](#the-macos-agenda) below is that ordering, with the case against
   each item.
2. **A runbook with verified commands and autonomy rules.**
   [The runbook](../guides/performance-loop-runbook.md) is one round start to finish on
   this host, every command run once while writing it, plus what an agent may not do
   without a person.
3. **A subject set that cannot be satisfied on paper.** Done in the review fixes to PR
   #45: a subject decides only when dense and at least 50,000 entries, and this host now
   has three deciding subjects.

## Where the Loop Stands

| Component | State | Evidence |
| --- | --- | --- |
| Protocol and accept rule | Sound: paired, interleaved, oracle-checked, 3% gate with a bootstrap interval, structural track with composite gates, floor-normalized budgets | [the performance loop](../guides/performance-loop.md) |
| Record | 66 artifacts (exp-000 to exp-065) when this review was written, 41 at 12 trials, every one `os_cache: warm-steady`; 57 macOS, 9 Linux. The agenda below records what the first night added | `docs/project/experiments/` |
| Views | Ledger and evidence page generated from artifacts; drift-checked in CI since PR #38 (`perf-test`, `perf-schema-check`, `perf-ledger-check`, `perf-report-check`) | `.github/workflows`, the **Performance evidence** job |
| Instruments | Per-layer counters; scan diagnostics; the aggregate-tier job with a tallies oracle, the nominated subject set and the tail statistic (PR #45) | `explorations/benchmarks/realtree/` |
| Subjects on this host | Three deciding: the 60k metabrowser clone (source checkout — and campaign 1’s own reference tree, exp-032), `~/.rustup` at 175k (package cache), the sealed `/System/Library/PrivateFrameworks` at 159k (system prefix). One screening: the 5.8k cargo registry | `docs/project/reports/nominated-subjects-darwin-arm64.json` |
| Cumulative result | Every accepted change against the pre-work baseline: `cold-scan-index` −54.5% [−55.3%, −53.7%] on the 60k tree (exp-032) | [the ledger](report-2026-08-10-fdu-performance-experiments.md) |

What the record does not hold, and an agent should know before trusting a gap in it:

- No artifact measures the default command, `fdu <dir>`. The nearest proxy,
  `cold-scan-index`, excludes the render and the snapshot write.
- No artifact is `held-out`; 16 record a `campaign_stage` at all, as `discovery` or
  `exploratory`. The artifact schema promotes `campaign_stage` but not `host_regime`, so
  even a quiet-host run would not say so in the ledger.
- The Linux campaign’s largest single win — snapshot load −51.9% [−53.2%, −51.0%] from
  loading beneath the parent already held (`4cc157d`) — is in the registry under H10 and
  in the 2026-08-14 campaign status report, but has no experiment artifact.
  The ledger’s “every experiment” is true of the macOS campaign and not quite of the
  Linux one.
- The `--purge` cold path has never produced an artifact; every number is warm-steady.

## What Would Have Misled an Unattended Agent

Each of these was checked against the code, the tracker, or a run rather than taken from
the documents.

**The floor is a Linux floor, so the campaign’s arithmetic does not apply here yet.**
Every threshold in the plan — 1.25× aggregate, 1.4× index — and its termination rule are
defined against `parfloor.c`, which uses `SYS_getdents64` and `statx` and does not build
on Darwin. On this host no ×floor can be computed, so an agent following the plan
literally cannot tell whether a tier is closed, and `fdu-33ri` (the scoreboard) was open
with a named obstacle and no decision.
The floor report itself says `getattrlistbulk` “changes the interface floor itself”, so
the macOS floor is a different program: a parallel `getattrlistbulk` walk that retains
nothing and produces the five tallies, checked by the tallies oracle PR #45 landed — a
floor that skips work is a fake denominator.
That is `fdu-9hdc`, and until it exists the agenda below is ordered by what is
measurable here rather than by ×floor.

**Phase A is Linux-only except for two items.** The cold thread policy (`fdu-tk1b`)
needs a Linux cold regime; PGO (`fdu-pdne`) and hardware CRC-32C (`fdu-6kyn`) are
portable. Of those two, hardware CRC-32C is a policy question rather than an experiment:
ARMv8 intrinsics are `unsafe`, and this repository keeps exactly one `unsafe` exception
behind a platform gate; a crate instead adds an always-on dependency to a core crate
whose list is deliberately short.
Either answer is defensible; neither should be chosen by an agent at 3 a.m.

**The default path is unmeasured and has two known defects.** `plan_report` never reads
the snapshot for a metadata query (`read_snapshot` is `analysis_requested`, because
revalidation stats every entry regardless), yet the cold-scan path writes one on every
run above the threshold (`cold_scan_save_targets` returns `SaveTargets::all()`). So on
the default path the snapshot is write-only cost except for `--cache only` and content
runs, and it is rewritten byte-identically when nothing changed.
Exploratory measurement on `~/.rustup`: `--cache auto` 0.24–0.27 s against `--cache off`
0.20 s warm, with the 13.9 MB snapshot’s mtime advancing on every run — a fifth to a
third of a warm default run.
The rendered report is also held in an 8 KiB `BufWriter` until the snapshot’s
`F_FULLFSYNC` and the index teardown complete (`cli.rs:598`, `:603`, `:1423`). Neither
can be judged until a ledger job measures `fdu <dir>` (`fdu-mx1w`), which is why that
job is first on the agenda.
The write itself is deliberate: in the catalog-evicted regime `--cache only` answers in
0.13 s where a scan costs 0.54 s, and `fdu-hvs5` rejected not persisting at all on that
evidence. The defect is the rewrite, not the write.

**The tracker disagreed with the record.** `fdu-91ts` was open though it landed in
`4cc157d`; `fdu-cckr` (mimalloc) was open at P1 though H74 is “confirmed for one tier,
not adopted”; `fdu-9ydj` and `fdu-4xtm` were the same no-oracle probe item at P3 and P1;
the plan’s Phase C “`fdu-cq7t` follow-on” — the content-tier instance of H86 — had no
bead though `fdu-cq7t` was closed; `fdu-926e` was carried as a priority though the plan
says re-scope or close; `fdu-tk1b` still recorded a blocker PR #45 had removed.
All six are reconciled in this branch (`fdu-02vv`), the campaign’s beads carry the
`campaign-2` label and the runnable-here subset carries `macos-agenda`, so the queue is
findable among the roughly two hundred open issues.

**The subject set could be satisfied by labeling.** Before the PR #45 review fixes, a
5,838-entry cargo registry cache labeled `source-checkout` gave the set its “can decide”
status. A smoke run on that tree — the same binary against itself, three trials —
returned ACCEPT at −23.72%, which is what a 3% gate looks like when the subject runs in
33 ms with 5 ms of spawn inside it.
The smallest subject ever to resolve a verdict on this record is 60k, so the floor is
50,000 entries and density, and the ranking rule (three characters) counts deciding
subjects only.

**The protocol’s raw commands were stale.** The guide’s “Running it” section invokes
`uv run --no-project python -m benchmarks.realtree`, which stopped resolving when the
harness moved under `explorations/`; the Make targets set `PYTHONPATH` and `--project`
correctly. The runbook uses the targets, and the guide’s commands are corrected in this
branch.

**This host is quiet only at night, and the first night it was not quiet at all.** Load
average runs 12–28 by day; the quiet gate is at most 25% instantaneous CPU busy before
and after every sample, which is achievable only when nothing else is running —
including the agent’s own builds.
Every artifact so far is exploratory or discovery; an unattended night is the one time
this host can produce a quiet-regime cell, provided measurement starts after the last
build finishes and `PERF_HOST_REGIME=quiet` is declared so a noisy sample is invalidated
rather than averaged in.
On 2026-08-23 the gate refused (CPU busy 44.3%): an `ANECompilerService` process had
held a core at ~99% for over a day and another session was running, so every artifact
from that night is `uncontrolled` and says so.
Restarting a system service is the owner’s decision, which is why the runbook tells an
agent to record the condition and go on rather than to clear it.

## The macOS Agenda

Three tiers. The first is what an unattended agent runs, in order; the second is what it
may run when the first is exhausted; the third is not started without a person, and the
reason is stated for each.

### Tier 1: run unattended, in this order

1. **`fdu-mx1w` — a `default-tree` probe job, recorded as a baseline.** Everything
   user-visible is measured through it and nothing ever had been.
   The job drives the real one-shot path (`prepare_report` with cache `auto`, tree view
   at its default depth, the text renderer, the save joined) with the tallies oracle,
   and a `JOBS` override lets a round name its jobs instead of running all six.
   *Against:* it is harness work and produces no speedup.
   *Answer:* an hour, and both items below are unmeasurable without it.
   **Landed** (exp-066): on the 175k rustup store the repeated run rewrote a 13.9 MB
   snapshot on 24 of 24 trials, and render plus write was about 70 ms of a 375 ms run.
2. **`fdu-2um8` — skip the identical snapshot rewrite on the cold-scan path.**
   Serialization is deterministic, so byte equality with the file on disk is exact and
   the cache cannot go stale.
   Predicted on the default job: warm wall down 15–30% on `~/.rustup`, RSS down ~20 MiB,
   with `cold-scan-index` unchanged.
   *Against:* the comparison costs a read of the existing snapshot, and a hash would be
   cheaper than a byte compare on a tree that did change.
   *Answer:* a page-cache read of 14 MB costs a few milliseconds, and the rewrite it
   replaces measured 40–70 ms; if the saving falls short of the prediction, that read is
   where to look. **Landed** (exp-067, H100): `default-tree` −10.61% [−14.85%, −6.05%] at
   16 trials, the other jobs unchanged, RSS flat, the tail narrowed from 1.15× to 1.05×.
   The prediction was optimistic by a third: the write is about 40 ms of the 70, the
   render and teardown the rest.
3. **`fdu-n75m`, part 1 only — flush the rendered report before joining the snapshot
   writer.** Changes no bytes and no work, only when the user sees them; recorded on the
   default job even though it needs no accept verdict.
   Parts 2 and 3 (dropping the index off the exit path; whether a checksummed,
   atomically renamed cache file needs `F_FULLFSYNC`) are durability decisions and stay
   in Tier 3. **Landed** (exp-068, H101): time to first byte −7.54% [−8.55%, −5.18%] on
   a repeated run and −12.47% [−15.66%, −9.84%] on a first run, total wall unchanged;
   the report reaches the terminal 41–49 ms before the process exits.
   Measured with `spikes/ttfb.py`, because no probe job can see when bytes reach a
   terminal.
4. **`fdu-pdne` — the PGO screen (H93), measure only.** One afternoon by the plan’s
   estimate; on macOS it needs the `llvm-tools` rustup component for `llvm-profdata`.
   *Against:* adopting PGO changes the release pipeline.
   *Answer:* the screen records a number and changes no build configuration; adoption is
   a separate decision with the interval in hand.
   **Skipped the first night:** `llvm-tools` is not installed, and under that night’s
   host noise (guard intervals ±13–23 points) a user-space gain bounded by the
   user-space share of a warm macOS wall, about 25%, could not resolve.
   Needs a quiet host and `rustup component add llvm-tools`.
5. **`fdu-78q6` — the content sidecar restore path (H83).** Same re-derivation shape the
   snapshot loader had, same class of fix expected; measured on the metabrowser clone
   (52,717 files, dense at 0.88) with `perf-content-compare`. *Against:* the warm
   content tier is a smaller share of what users run than the default path.
   *Answer:* it is the largest unexamined item on its tier and is platform-neutral, so a
   macOS verdict transfers.
   **Re-profiled** on the metabrowser clone: path comparison is 33% of the warm content
   open (`compare_components` 17%, `Components::next` 12%), the `BTreeMap<PathBuf, _>`
   residue the bead’s re-screen named; exp-069 (H102) keys that map by path bytes.
   See the ledger for its verdict.
   The next increment is the 8% of `Path::hash` and SipHash on the roll-up map; the
   structural form is `fdu-jxhk`.

### Tier 2: instruments, when Tier 1 is exhausted

6. **`fdu-9hdc` — the macOS floor.** A parallel `getattrlistbulk` walk with the tallies
   oracle, a worker-count sweep on the three deciding subjects, and the ×floor row for
   `aggregate-summary` and `cold-scan-index`. Then `fdu-33ri` can ship the scoreboard as
   two tables with the regime difference recorded.
   Until this exists, no tier can be declared closed here.
7. **`fdu-4xtm`** (no-oracle probe mode and engine-scoped counters), **`fdu-5yjk`**
   (scan diagnostics on the FullIndex plan), **`fdu-0pzh`** (channel occupancy, measure
   only). All three are pre-work for H86 and change nothing a user runs.
8. **Promote `host_regime` into the artifact schema**, so a quiet-regime cell is visible
   in the ledger and not only in the run JSON.

### Tier 3: needs a person

- **`fdu-xde5` — H86, the campaign’s centerpiece.** A representation change across the
  consumer, run once as a composite on the 542-line precedent of exp-022; on macOS its
  wall payoff is regime-bound — invisible when the catalog is evicted, about 25% fully
  warm, about 6% at 902k — and its macOS payoff is RSS. Multi-session, supervised.
- **`fdu-jxhk` — the content-tier instance of H86.** One module rather than the whole
  consumer, oracle-checked by the content digest; the one structural item an unattended
  agent could reasonably attempt last, on the metabrowser clone.
  Listed here rather than in Tier 2 because it is still a representation change.
- **`fdu-6kyn` — hardware CRC-32C.** The `unsafe`-versus-dependency policy question
  above.
- **`fdu-9716` — the `searchfs` spike (H77).** Volume-wide catalog enumeration with an
  EBUSY failure mode; the only mechanism under macOS’s per-directory open floor, and a
  decision about running it against this machine’s volume.
- **FSEvents journal scoping** (`fdu-2cdv`, `fdu-3tun`, `fdu-rvje`, `fdu-6ld9`). The
  only lever on any platform under the stat floor, and after the default-path fixes the
  whole remaining warm gap on this host.
  A correctness surface (the delta contract), scheduled as Phase D; its Phase 0 spike
  belongs in a supervised session rather than a default deferral.
- **Phase E** — bare metal, peer cells, quiet-host confirmation: other hosts.

## The Strategy, and the Case Against It

**Floor-normalized budgets are the right change.** A relative loop cannot say how much
is left, and campaign 1 ran sixty-four times without that number.
*Against:* the floor is a Linux floor, and on APFS the bulk interface moves it; the
campaign’s “aggregate tier is nearly finished” is plausible here — fdu’s summary ties
dumac, which is close to a practical floor — but unmeasured.
Tier 2’s first item is what makes the arithmetic portable.

**The structural track is right, and it is exactly what an unattended agent should not
run.** Forcing a representation change through per-piece 3% gates would reject a
measured ~4× seven times; the composite is the honest unit.
It is also a 542-line change with pre-registered targets on four metrics, which is a
supervised session by construction.

**The real-subject rule is right and is now enforced** by size and density rather than
by character labels; the 5.8k demonstration is the reason.

**Termination is right and currently unevaluable here**; see the floor.

**Re-posing the warm story is right, and it has a consequence the plan under-weights.**
The stat floor means a warm default run on this host cannot beat Ω(N) stats without
journal scoping; once the default-path defects are fixed there is nothing else in the
plan that moves a warm `fdu <dir>` on macOS. The plan schedules the journal last.
*For promoting it:* on the host the bar is set on, it is the only remaining warm lever.
*Against:* it is the largest and most platform-specific item in the plan and touches the
delta contract. The recommendation is neither to start it unattended nor to leave it as
the default deferral: schedule its Phase 0 spike for the first supervised session after
Tier 1 lands.

**What a night should produce.** One branch, one pull request updated after every
experiment, no merge to `main`; each experiment is one commit carrying its artifact and
the regenerated views, with the code only when accepted.
Exp ids are one sequence: a second agent running the same night reserves a range first.
The morning’s review reads the ledger diff, not the session transcript.

## Decisions This Review Asks For

1. Adopt the macOS agenda ordering above; it is written into the campaign-2 plan.
2. Build the `getattrlistbulk` floor as the macOS floor (`fdu-9hdc`) and let `fdu-33ri`
   ship two scoreboards with the regime difference recorded.
3. Decide the `unsafe`-versus-dependency policy for `fdu-6kyn` before it runs.
4. Decide the fsync policy question in `fdu-n75m` part 3.
5. Schedule the FSEvents Phase 0 spike for the first supervised session after Tier 1.

## References

- [The campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) —
  owns prioritization; carries the macOS agenda
- [The runbook](../guides/performance-loop-runbook.md) — one unattended round on this
  host
- [The performance loop](../guides/performance-loop.md) — the protocol and the registry
- [The metadata-walk floor report](report-2026-08-23-metadata-walk-floor.md) — the Linux
  denominator and why it does not transfer
- [The ledger](report-2026-08-10-fdu-performance-experiments.md) and
  [the evidence report](report-2026-08-20-fdu-performance-evidence.md)
- Beads: `fdu-d4kg` (the agenda epic), `fdu-fm41`, `fdu-02vv`, `fdu-9hdc`, `fdu-jxhk`,
  and the `macos-agenda` label

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
