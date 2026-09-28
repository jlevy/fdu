# Plan: macOS Performance Rerun

**Date:** 2026-09-27

**Author:** fdu project

**Status:** Draft. Measurement is explicitly on hold while the user selects further
integrations.
Only the main-branch merge, focused correctness smoke checks, and this plan
are authorized now. No new performance result is claimed.

## Overview

Measure the final integrated engine on internal APFS storage, with builds and dependency
caches on the external SSD. Separate basic disk usage, full analysis, and repeated
report construction: they are different workloads and cannot share a speedup headline.
The current preparation merges `origin/main` at `02ab4cf5` into the H153 branch.
That revision is a preparation checkpoint, not the frozen revision for the official run.

## Goals and Non-Goals

- Confirm H153 against the same final feature stack without H153, with its exact oracle
  and existing resource gates intact.
- Obtain current macOS CLI measurements only for the workload cells approved before
  launch, recording the complete commands and cache semantics.
- Account separately for building, fixture preparation, measurement, evidence
  publication, local validation, and CI wait.
- Do not run timings, generate large corpora, tune the engine, lower statistical gates,
  or update README speed claims during this preparation.
- Do not extrapolate macOS measurements to Linux or a report-component improvement to
  scan/full-analysis performance.

## Background: Where the Time Went

The committed artifacts distinguish measured command latency from matrix elapsed time.

| Recorded workload | Per-command headline | Actual matrix duration | Work behind the duration |
| --- | --- | --- | --- |
| H153, exp-159 | 38.6 s control / 20.6 s candidate whole probe; about 299 / 120 ms per report | 967.743 s, or 16.1 min | Two arms, 3 warmups and 12 timed pairs; each process scans, analyzes lines, constructs 100 four-view reports, validates, and tears down |
| September 26 peer comparison | fdu median 6.0 s | 5,071.342 s, or 84.5 min | Nine competitors, each paired with fdu: 270 processes including warmups; BSD du, ncdu, and GNU du take approximately 49–62 s each |

These durations come from the raw run files, not an estimate of the previous chat’s
elapsed time. Builds, repeated full handoff gates, source-package rebuilds, evidence
generation, and waiting for CI are additional costs.
No complete phase-by-phase timing was retained for that work, so its individual costs
must not be invented.

For planning, the command-time subtotal of an adjacent-pair matrix is
`(warmups + trials) × sum(anchor seconds + peer seconds)` over its peers.
At the old medians, dust plus dumac alone account for about 419 command-seconds, versus
about 4,199 command-seconds for all nine peers.
These are workload-cost calculations, not new timings or a completion promise;
fingerprints, host observations, validation, and changed host conditions add cost.
On macOS each measured invocation also has one-second CPU observations at both
boundaries.

## Design

### Storage and source identity

During preparation, set `TMPDIR`, `CARGO_TARGET_DIR`, `UV_CACHE_DIR`, and
`UV_PROJECT_ENVIRONMENT` to task-specific external paths.
Verify the external volume is mounted and writable first; never fall back to internal
builds. Give each checkout its own Cargo target.
Reuse that checkout’s verified target rather than rebuilding every dependency in a new
directory, and preserve immutable copies of the resulting binaries.

During measurement, the subject, benchmark-owned temporary directory, and any measured
snapshot/sidecar state must be on the internal APFS volume.
Set a bounded internal `TMPDIR` for that phase, as the benchmark README prescribes;
restore external build settings before any later compilation.
Keep results outside the subject, retain unique evidence in the repository, and do not
clean it as disposable build scratch.
An external executable is acceptable; reading the benchmark tree or measured cache on
the external SSD is not this experiment.

Reuse an existing internal subject; do not generate another million-entry corpus by
default. Check mount/device identity rather than assuming a path is internal.
The preparation check found only about 5.4 GiB internal and 17 GiB external free; these
are transient observations, not a reserved budget.
Before launch, estimate cache/output growth from the selected jobs and preserve headroom
for the OS. If insufficient, pause and resolve storage explicitly.
No RAM disk, automatic Trash emptying, or unrelated cleanup is part of this plan.

### Workload cells and evidence scope

| Cell | What it answers | Baseline and method | Launch condition |
| --- | --- | --- | --- |
| H153 confirmation | Does shared per-file resolution improve repeated multi-view reports? | Final integrated candidate versus the same stack with H153 removed, retaining H152’s exact report oracle in both; existing `content-query`, 100 reports/process, 3 warmups, 12 fixed interleaved pairs | Required for resolving `fdu-9e9d`; nominated dense internal real tree with at least 50,000 entries, preferably the recorded metabrowser subject if unchanged |
| Basic disk usage | What does the current command actually cost? | Distinguish cache-off indexed tree (`fdu` comparison contract) from bare `fdu-default-tree`, which includes snapshot writing; compare identical contracts across the chosen baseline and candidate | Pick the user-visible contract before running; do not call cache-off data a default-command result |
| Full analysis | What does an end-to-end analyzed report cost? | Separate cold `--cache off --analyze all` and an explicitly specified multi-view request; compare exact stable report content at matched arguments | First provide/review an end-to-end adapter and oracle: the existing `content-query` job is not this measurement, and `compare_tools` has no full-analysis contract |
| Closest peers | How does current metadata performance compare with dust and dumac? | Existing adjacent-pair tool harness, pinned binaries/work classes, 3 warmups and 12 pairs per peer | Optional first comparison; no claim that different tool outputs are equivalent |
| Full peer table | Refresh all nine historical comparison rows | Existing full matrix, including serial floors and ncdu | Explicitly select this costly publication workload; do not run it for each optimization |

No default `perf-content-compare` invocation: it selects eight jobs, not just H153. Set
`JOBS=content-query` explicitly.
Do not run the six-job metadata default unless the question requires every cell.
Bare `--analyze all` selects Code and Documents; only one eligible metric view is
present, so it does not enter H153’s shared pass.

The generated balanced-million tree can refresh that specific historical peer table, but
cannot establish a general real-tree ranking or decide H153. Such claims require the
nominated real-subject policy in the performance loop.
Select and fingerprint the actual internal subjects after the integration freeze, not
during this planning pass.

### Faster execution without weaker evidence

1. Freeze source, dependencies, and harness once after all chosen integrations.
   Build each required release artifact once.
   Do not build profiling binaries unless a named attribution question requires them.
2. Run focused correctness checks before measurement.
   Keep one full `make check` handoff after final code/evidence publication, not between
   cells or during measurement.
   A real failure still requires diagnosis and affected validation; this is not a
   waiver.
3. Verify one subject per selected cell and reuse immutable binaries.
   Avoid broad subject scans and all-job defaults in the inner loop.
   Keep required immediate pre/post fingerprints and every sample’s correctness oracle.
4. Retain 100 queries for the H153 confirmation so it answers the existing experiment.
   A future 10-query diagnostic could shorten exploratory feedback, but changes setup
   amortization and whole-process resource behavior.
   It needs a separately named, tested job and cannot silently replace confirmation
   evidence.
5. Preflight AC power, normal thermal state, memory pressure, and the unchanged 25%
   quiet-host CPU threshold.
   Run no builds or competing benchmark/agent work during a cell.
   If the host cannot hold quiet, stop and record the failure to qualify rather than
   silently switching the official run to uncontrolled.
6. Record phase start/end times and the planned process count.
   Set an execution budget before launch from historical cost plus a separately labeled
   calibration, if needed.
   A budget abort preserves partial evidence and is inconclusive, never an early accept.

The H153 acceptance metric remains paired whole-probe wall time, with the original
resource rules including major-fault non-regression.
Report component timing separately.
Keep fixed N, no optional stopping, invalidated observations, unchanged fingerprints,
paired bootstrap intervals, tails, and binary/source provenance.
Do not combine the old and new observations or alter the old exp-159 verdict
retrospectively without new qualifying evidence.

## Implementation and Validation Plan

- [x] Complete the main merge and focused query/renderer/CLI smoke checks; publish this
  plan without running timings.
- [ ] User selects remaining integrations and explicitly authorizes the official run.
- [ ] Freeze final candidate and baseline source identities.
  The H153 control differs only by removing shared resolution; an older pre-feature
  binary is not a valid control.
- [ ] Select approved cells, internal subjects, work contracts, and resource budget.
  Resolve full-analysis adapter/oracle work separately if that cell is requested.
- [ ] Build verified immutable artifacts externally, preflight correctness and host
  conditions, then run only the approved fixed-N cells on internal storage.
- [ ] Record every attempted cell, publish generated evidence once, validate the final
  handoff, and update README claims only to the scope the new evidence establishes.

Preparation validation passed Rust formatting, 139 query tests (including shared versus
independent metric reports), the focused Code-renderer tests, and 100 CLI tests, plus
documentation formatting and whitespace checks.
Build output stayed external.
No timing matrix, corpus generation, or release benchmarking ran.

No additional public API, engine algorithm, or benchmark implementation changes are
proposed beyond the upstream merge.
At the user’s request, local validation was limited to these smoke checks, not another
full `make check`; the full final handoff gate remains required for the later official
run. Ordinary CI validates correctness and historical evidence; it does not perform the
official host timing experiment.

## Open Decisions

- Which additional PRs belong in the frozen candidate?
- Is the first publication limited to H153 and basic CLI data, or does it include
  full-analysis adapter work and peers?
  Recommend dust/dumac first if peers are included.
- Which existing internal subject is stable and fits the cache-space budget?
- Is a new matched baseline available for each product-level claim, or should that cell
  report current absolute performance without claiming an incremental speedup?

Tracking: `fdu-6h2q` owns preparation; `fdu-9e9d` owns H153 confirmation and remains on
hold until the user authorizes measurement.
Assign the next free experiment ID only after coordinating with the shared registry at
launch; this plan reserves none.

## References

- [Current pickup](../../guides/performance-loop-runbook.md#current-pickup-2026-09-27)
- [Performance loop and gates](../../guides/performance-loop.md)
- [Benchmark storage and command contracts](../../../../explorations/benchmarks/README.md)
- [H153 experiment](../../experiments/exp-159-share-content-metric-resolution-across-views.md)
- [H153 raw run](../../experiments/evidence/exp-159/run.json)
- [Historical peer comparison](../../reports/report-2026-09-26-fdu-live-tool-comparison.md)
- [Historical peer raw run](../../reports/fdu-live-tool-comparison-result-2026-09-26.json)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
