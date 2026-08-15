# Adaptive Worker Gap Closure on Apple Silicon and APFS

**Date:** 2026-08-15

**Author:** fdu project

**Status:** Complete; no production controller change

## Summary

The reported behavior is real but was initially described too strongly.
Automatic scan mode makes a one-time worker decision from the first 16,384 entries to
*finish*, so a heterogeneous tree can expose different policy histories when completion
order changes. That is order sensitivity.
It is not, by itself, evidence that holding six workers is a performance defect.

On the measured Apple M1 Pro and local APFS corpus, unlocking more workers was the
harmful outcome. Repeated-window and staged controllers detected the late slow phase,
expanded toward sixteen workers, and made wall time 58.49% and 60.73% worse.
Fixed eight, ten, and sixteen workers also lost decisively to six.
Profiling attributes the result to more kernel work, lock wait, and scheduler pressure
rather than to an unexploited latency-bound frontier.

The production controller therefore remains unchanged.
The branch adds the evidence system that was missing: bounded policy/backend traces,
deterministic completion-order models, phase-checked corpora, fail-closed statistics,
installed-command provenance, a validated dust adapter, and exact release-surface
checks. No experimental controller is selected by the shipped CLI or standard library
scan APIs.

The release comparison is exact but not claim-confirming on this session’s host.
A quiet held-out native cell had two host-pressure invalidations.
Clean uncontrolled diagnostic cells found native fdu 43.10% faster than dust and
wheel-installed fdu 41.70% faster, but those results are not substituted for the
predeclared quiet-host gate.

## Scope and decision table

All timing in this report is scoped to one bare-metal Apple M1 Pro with 8 performance
cores, 2 efficiency cores, 32 GiB memory, AC power, normal thermal pressure, local APFS,
and a warm-steady operating-system cache.
Positive change means the candidate is slower.

| Question | Evidence | Decision |
| --- | --- | --- |
| Does completion order affect the one-shot answer? | Pure injected-order model and complete runtime traces | Yes; record it as sensitivity, not presumed harm |
| Did the observed late slow phase need more workers? | 12-pair discovery screen plus fixed controls | No; every count above six regressed |
| Should repeated windows ship? | +58.49% wall, 95% CI [+49.94%, +66.38%] | Reject |
| Should staged, ready-work-gated expansion ship? | +60.73% wall, 95% CI [+49.80%, +67.33%] | Reject |
| Should the 30 µs threshold or worker caps change? | No higher-count arm survived discovery | No |
| Did diagnostics perturb the measured scan? | -0.55% wall, 95% CI [-1.09%, +0.17%] | Accept the opt-in trace |
| Is the quiet installed-CLI release matrix confirmed? | Exact semantics, but 2 of 24 native timed processes invalidated by host pressure | Inconclusive; no positive release claim |

## What prompted the investigation

A live, partial scan of a heterogeneous application-data tree completed 398,926 entries
and 39,936 directory reads with twelve explicit errors.
The shipped policy held at six workers after an early 28.5 µs/entry window; later
windows measured approximately 43.5, 50.5, and 30.5 µs/entry.
All readable directories used the macOS bulk backend.

That observation established a plausible mechanism but could not support a timing claim.
The subject was mutable, permission-bearing, and machine-specific.
It did, however, identify the missing evidence: the old aggregate counters could not
show the decision ordinal, later service windows, ready work, active workers, or backend
fallbacks.

## Why the earlier evidence missed it

Experiments exp-015 through exp-036 established useful worker bounds and the macOS bulk
reader, but they did not answer this question:

- They measured aggregate outcomes on mostly uniform or natural trees without retaining
  the policy’s bounded history.
  A held walk and an unmeasured walk could look alike.
- The policy discarded its calibration after one decision, so later service changes were
  invisible even when counters were enabled.
- Filesystem generation order was sometimes treated informally as a phase shape, but
  concurrent completion order is not a property of a generated tree.
- The bulk backend changed the worker knee.
  A pre-bulk large-tree result favoring deeper concurrency was not evidence that the
  same count remained useful after directory metadata became batched.
- Earlier fdu-versus-dust comparisons proved exact output and process timing for their
  work classes, not which automatic policy path fdu took.

The repair is methodological rather than a retune: observe the actual policy path,
verify real phases after the run, separate discovery from confirmation, and refuse a
claim when the host or trace contract is incomplete.

## Evidence added

### Bounded runtime trace

`fdu-scan-diagnostics-v1` records available, initial, maximum, live, and peak workers;
window entry ordinals and service signals; decisions and requested workers; ready and
in-flight directories; handoff backlog and high-water mark; and macOS bulk, fallback,
and portable directory counts.
It is capped at 256 policy events.
Missing platform data is `null` with a reason, and truncation invalidates claims that
need the omitted history.

The trace is an opt-in internal scan contract.
The performance probe exposes it with `--diagnostics`; the installed CLI emits it only
when `FDU_SCAN_DIAGNOSTICS=1`, on a tagged stderr transport that leaves normal human and
machine output unchanged.

Twelve paired scans bounded the enabled trace’s wall effect at -0.55% [-1.09%, +0.17%].
See [exp-056](../experiments/exp-056-bound-adaptive-scan-diagnostics-overhead.md).

### Deterministic policy model

The controller tests inject completion histories directly.
They cover fast/slow order reversal, completion censorship from slow in-flight work,
alternating phases, late activation, narrow frontiers, delayed handoff consumption,
exactness, bounded workers, disconnect, and shutdown.
The legacy fixture demonstrates the one-shot sensitivity without leaving a failing test.

The statistical contract now distinguishes:

- an **outcome** such as held or scaled;
- an **order-sensitivity signature**, such as held before a later slow window; and
- **structural harm**, which requires an impossible or explicitly harmful history.

This distinction corrected an important error found during the campaign: “held before a
later slow window” is evidence that completion order matters, not proof that holding was
slower.

### Phase-checked corpora

The generated families now include fast-prefix/slow-suffix, slow-prefix/fast-suffix,
alternating, many-small-directory, few-wide-directory, wide, and deep shapes.
Every corpus has a versioned manifest and independent semantic oracle.

Several nominal phase recipes did not complete in their generated order on APFS. The
harness invalidated those phase claims rather than relabeling the traces.
This is a useful negative result: topology is reproducible; filesystem enumeration and
concurrent completion order are not.

The controller screen used a frozen 100,001-entry corpus with 60,314 directories, 39,687
files, two explicit topology regions, no mutation, no baseline drift, exact engine
digest, and trace-verified fast-then-slow completion windows.

## Profile before controller work

The shipped profile held six workers.
Its first window was fast and five later complete windows were slow.
All 60,314 directory reads succeeded through `getattrlistbulk`; none fell back to the
portable backend. Kernel/syscall frames accounted for 73.71% of 31,430 stack samples,
while fdu scan code accounted for 1.22%.

The repeated-window profile expanded to sixteen workers at the second window.
Kernel and syscall frames rose to 80.72% of 60,911 samples, handoff high-water
increased, and aggregate lock wait moved from negligible to hundreds of milliseconds.
The profile therefore falsified the proposed diagnosis that the late region primarily
lacked parallelism. It showed more open/bulk work and contention after expansion.

These profiles are attribution evidence, not stopwatch claims.
Their source revision, binary hash, counters, trace, stacks, and command are retained
together.

## Controller and hardware screen

The final discovery run used twelve interleaved pairs after three warmups and a fixed-N
stopping rule. It had no invalid samples, semantic mismatches, trace gaps, corpus
mutation, or baseline drift.
The host was recorded as uncontrolled, so the run may eliminate large regressions but
cannot confirm a winner.

| Variant | Median wall | Versus shipped | 95% interval | Result |
| --- | ---: | ---: | ---: | --- |
| Shipped one-shot | 1.872 s | baseline | — | Retain |
| Fixed 6 | 1.878 s | +0.76% | [-0.06%, +1.18%] | Practically level |
| Fixed 8 | 2.532 s | +36.29% | [+33.95%, +38.39%] | Inferior |
| Fixed 10 | 2.918 s | +57.49% | [+49.91%, +66.62%] | Inferior |
| Fixed 16 | 3.022 s | +61.48% | [+58.76%, +78.95%] | Inferior |
| Repeated windows | 2.963 s | +58.49% | [+49.94%, +66.38%] | Inferior |
| Staged and gated | 2.988 s | +60.73% | [+49.80%, +67.33%] | Inferior |

Repeated windows increased aggregate CPU 151.57%, system CPU 161.05%, peak RSS 6.40%,
minor faults 6.57%, and involuntary context switches 616.76%. Staged expansion showed
the same mechanism: +157.89% CPU, +167.72% system CPU, +6.01% RSS, and +657.55%
involuntary context switches.
Both failed the pre-registered wall and resource gates.

A controlled-interactive replication with two synthetic load workers preserved the
direction—repeated +49.49% and staged +39.85%—but 20 samples crossed the host-pressure
envelope. It is retained as an invalidated diagnostic, not confirmation.

No controller survived discovery, so selecting a winner and then running a held-out
confirmation matrix would be both unnecessary and statistically misleading.
Throughput gradient and reversible-parking variants were also screened out analytically:
on the observed persistent slow suffix, any design must first incur the expansion whose
fixed controls already establish as harmful; later parking cannot outperform never
expanding.

See
[exp-057](../experiments/exp-057-reject-repeated-adaptive-worker-windows-on-apfs.md),
[exp-058](../experiments/exp-058-reject-staged-adaptive-worker-expansion-on-apfs.md),
and
[exp-059](../experiments/exp-059-reject-higher-fixed-worker-counts-on-mixed-phase-apfs.md).

## Installed CLI and dust qualification

The release harness pins Homebrew dust 1.2.4 by version, executable hash, bottle and
source hashes, formula revision, target, license, and exact command.
Its adapter parses one allocated-byte total, normalizes hard-link semantics, rejects
symlink/error-bearing or incomplete work, and checks an independent tree oracle before
accepting timing.

Both supported fdu surfaces were built from clean commit `1d70c62`:

- a Cargo-installed native CLI; and
- an isolated wheel-installed console script plus its hashed ABI3 native extension.

Each attestation proves the version and hashes, a real cache-off summary scan, and clean
bash and zsh resolution without changing user shell configuration.

The 12-pair uncontrolled diagnostics were exact and stable:

| Installed surface | fdu median | dust median | fdu wall change | 95% interval | fdu peak RSS change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native Cargo CLI | 1.525 s | 2.681 s | -43.10% | [-44.00%, -42.46%] | -90.59% |
| Python wheel CLI | 1.559 s | 2.670 s | -41.70% | [-42.54%, -40.24%] | -78.36% |

Both cells had zero invalid samples, semantic mismatches, oracle mismatches, mutations,
or baseline drift. The native cell cleared every diagnostic resource gate.
The Python cell’s major-fault and voluntary-context-switch intervals were inconclusive
around zero, which is recorded rather than coerced into a pass.

The quiet held-out native cell also had exact output and a -43.21% diagnostic wall
effect, but two fdu processes crossed the 25% host-pressure ceiling.
Its fixed-N matrix was therefore incomplete and the release qualification is
inconclusive. No uncontrolled number in this section is a replacement for that missing
quiet confirmation.

## Correctness, errors, and platform behavior

The ordinary suite checks exact results and termination under every controller model,
explicit thread counts, consumer disconnect, and queue shutdown.
macOS traces cross-check 60,314 bulk attempts and successes with zero fallbacks on the
frozen subject; portable platforms report their own counts or an unavailable reason.

Permission behavior was not weakened for benchmarking.
A deterministic fixture proves that a partial scan exits 2 by default, machine output
retains every error, human output warns, and `--allow-partial` is the only path to
success.
Error-bearing and live TCC-specific samples remain diagnostic and cannot support
speed claims. The dust adapter similarly invalidates warnings, nonzero exits, timeouts,
unparsable totals, and semantic mismatches.

## What shipped and what did not

Shipped:

- a bounded, versioned, opt-in diagnostic contract;
- aggregate counter and trace cross-checks;
- deterministic controller and liveness tests;
- phase-stress corpus recipes and post-run phase verification;
- fixed-N, paired, fail-closed policy and resource decisions;
- instantaneous macOS CPU-pressure boundaries rather than lagging load-average gates;
- stale-binary rejection, build/host provenance, installed-command attestation, and the
  pinned dust adapter.

Not shipped:

- repeated-window, staged, gradient, or parking controllers;
- new worker counts, thresholds, platform branches, dependencies, or unsafe code;
- any change to automatic scan behavior or explicit `--threads` semantics;
- a positive quiet-host release-performance claim from this session.

## Residual limits and follow-up

This result does not establish performance on Intel Macs, non-APFS filesystems, remote
or removable storage, Windows, Linux, or controlled-cold macOS state.
The ordinary cross-platform suite protects exactness; it does not turn one M1 Pro
measurement into a portable speed claim.

The P2 shared-opener experiment remains separate because opener threads and scanner
workers must share one total concurrency budget.
Cold-cache qualification remains deferred until the dedicated APFS-volume protocol
exists. Neither changes this epic’s no-controller-change decision.

## References

| Document | Role |
| --- | --- |
| [Design principles](../architecture/fdu-design-principles.md) | Product invariants and evidence scope |
| [Performance loop](../guides/performance-loop.md) | Measurement and decision protocol |
| [Instrumentation playbook](../guides/performance-instrumentation-playbook.md) | Bounded trace design |
| [Platform tuning](../guides/platform-tuning.md) | Constant-to-evidence mapping |
| [Experiment ledger](report-2026-08-10-fdu-performance-experiments.md) | Generated experiment history |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
