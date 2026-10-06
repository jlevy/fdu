# Feature: The fdu Performance Index

**Date:** 2026-10-05

**Author:** fdu project, with Claude Code assistance

**Status:** Draft, for review.
The suite, the weights, and the reference build are decisions the maintainer makes
before any index run; until then nothing here is a claim.

## Overview

One pre-registered number that says how fast fdu is across everything it is for, with
every component it is made of published beside it.
The **fdu performance index** is a weighted sum of log speedups over a fixed suite of
components. Each component is one user-facing scenario on one platform, or peak memory,
and each speedup is measured within one interleaved session against a fixed reference
build. The charted page shows the index as its headline line, and a chooser switches the
same chart to any single component.

## Goals

- **One score that reflects what matters.** It covers the default command on a real
  repository, scale, the summary view, repeat runs with the cache, content analysis, and
  memory, on macOS and on Linux.
- **Every improvement can show somewhere.** A kept change appears in the index or in a
  named component. Today the 1M-tree line cannot show Linux, summary, cache, or
  `.gitignore` work, which is why the green bars and the line disagree.
- **No chaining across sessions.** Every ratio comes from one interleaved session.
  Compounding per-experiment effects claims more than 20,000×, so the index never does
  it.
- **Fixed before measuring.** The suite, the commands, the weights, and the reference
  build are recorded before the first run and change only by a versioned revision.
- **Every component stays visible.** The page’s chooser shows each component on the same
  axis, so no regression hides inside a good average.

## Non-Goals

- **Peer comparison.** The index measures fdu against itself.
  Rankings against dust, dumac, pdu, diskus, and the rest stay in the tool comparisons.
- **The accept rule.** Single changes are still decided by their own paired cell and the
  3% rule. Using the index as a gate on changes is an open question, not this spec.
- **Cold disk and bare metal.** The index is warm-steady and uses the hosts available;
  cold-cache and bare-metal components wait for hosts that can measure them.
- **Tail latency.** `p95_over_median` at 12 to 20 rounds is an order statistic, not a
  stable population figure.
  It stays in the records, outside the index.

## Background

- **The loop judges each change on its own job, tree, and platform.** That is right for
  a verdict and wrong for a summary.
  Of the 74 changes kept at least 3% better:
  - 60 were judged on jobs other than a full scan: the summary view, the content cache,
    revalidation, content queries, and snapshots;
  - 30 were measured on Linux;
  - 13 re-measure earlier changes.
- **The history cells of 2026-10-05** time 13 milestone builds in one session on two
  trees on the internal SSD
  ([the evidence report](../../reports/report-2026-08-20-fdu-performance-evidence.md)).
  - The generated 1M-entry tree: 10.8×, all of it from H1 and bulk metadata.
    Every later build is level, because macOS metadata calls set the time on a tree four
    times the vnode limit.
  - The Linux v6.12 source tree: 3.3×, with a rise at 0.1.0 when `.gitignore` reading
    arrives and steps down after it.
  - Neither cell sees the summary view, the cache, content, or Linux.
- **The design principles name the dimensions:**
  - every output surface is a benchmark job;
  - a warm path that loses to a cold scan is a defect;
  - a speedup bought with memory is not free;
  - a measurement is evidence about its own regime
    ([fdu design principles](../../architecture/fdu-design-principles.md#performance)).
- **The index’s form is not new.** The 2026-08-14 performance explorer (branch
  `codex/performance-research-white-paper`) computed a weighted latency index over
  Linux/macOS × cold/warm cells as `1 − exp(Σ w · ln(candidate ÷ control))`. That is the
  same form, over fewer cells.

## Design

### The Formula

For build *b*, component *c* with weight *w_c* (weights sum to 1), and reference build
*R*:

- **component ratio** *r_b,c* = median wall time of *b* on *c* ÷ median wall time of *R*
  on *c*, both from the same interleaved session;
- **index** *I_b* = exp(Σ_c *w_c* · ln *r_b,c*). It is a relative runtime: 1.0 is the
  reference build, and 0.5 means half its time on the weighted suite;
- **score** = 1 ÷ *I_b*, the speedup shown on the page (“3.4× the first build”).

The exponent is a weighted sum of log speedups, which is the weighted sum the score is
built on.

**Why not a plain weighted sum?**

- A sum of raw times lets the slowest benchmark set the score: a 70 s scan would
  outweigh every 160 ms job combined.
- A weighted arithmetic mean of ratios treats a 2× slowdown (+100%) and a 2× speedup
  (−50%) unequally.
- Summing logs is scale-free and symmetric: doubling one component’s speed and halving
  another’s cancel exactly.

**Interval.** The score’s interval comes from a paired bootstrap that resamples the
rounds of each component’s cell, recomputes every component ratio, and recombines them.

**Memory.** Peak RSS enters the same way, as a ratio of peak RSS to the reference build
on the memory component’s scenarios.

### The Suite

Each component is a command on a fixed tree, run as each build shipped it: the cache off
for first runs; `.gitignore` read by default where the build supports it, and not
equalized where it does not.
Trees:

- **K**, the Linux v6.12 source tree from `git clone --depth 1 --branch v6.12`: about
  92k entries and 358 `.gitignore` files, below macOS’s vnode limit;
- **G**, the generated 1M-entry tree from the `balanced` recipe.

Default weights per platform:

| Component | Command and tree | Why it matters | Weight |
| --- | --- | --- | ---: |
| Default command on a repository | `fdu PATH`, first run, on K | What most users run, on what most users have | 30% |
| Scale | `fdu PATH`, first run, on G | Very large trees, where peers compete hardest | 15% |
| Summary | `fdu --view summary PATH` on K | The du-replacement total | 15% |
| Repeat run | `fdu PATH` a second time, with its cache, on K | The cache’s promise: a warm path must beat a cold scan | 15% |
| Content | `fdu --view code PATH`, first run, on K | Code and document metrics, fdu’s differentiator | 15% |
| Memory | Peak RSS of the two default-command components | A speedup bought with memory is not free | 10% |

- **Platforms.** macOS and Linux each carry 50% of the total.
- **Missing platform.** A macOS-only index is published as such and never presented as
  the full index.
- **Missing capability.** A build without a component’s capability (content views
  arrived after the pre-work binary) has no ratio for that component.
  A history chart’s index is computed over the components every plotted build supports,
  renormalized, and says which it omits.
  The release index uses every component.

### Measurement

- **One history cell per tree and platform.** All builds in the comparison are timed
  interleaved, anchored on the reference build:
  - order alternated, 3 warm-ups, at least 12 rounds, 20 where the cell stays under an
    hour;
  - answers checked before timing within each capability group;
  - trees and binaries on internal storage (results may go elsewhere);
  - the harness’s quiet gate unchanged, and the regime recorded.
- **Where.** macOS cells run on the maintainer’s Mac.
  Linux cells run on a Linux host, for example a cloud session handed the same builds
  and manifest.
- **Reference build.** Default: the latest release, v0.3.0, re-pinned only by a
  versioned revision of this spec.
  Each cell’s anchor is the reference build, so every ratio is within-session.

### The Page

- **The chooser.** The runtime-over-time chart gets a metric chooser:
  - **Unified score** (default): the index line per platform, plus the combined line
    when both platforms exist;
  - each component on its own;
  - memory.
- **The headline** states the unified score from the first build to the current release,
  with its interval.
- **The experiments panel** marks each bar by the components its primary job maps to.
  A bar whose job maps to no visible component is drawn faded, and its tooltip says why
  (“Linux only”, “summary view”, “warm cache”). The chart then stops implying that every
  green bar should move the line above it.

### Components

- **A suite manifest** at `explorations/benchmarks/index-suite.json`. It holds
  components, trees, per-build-era commands, weights, the reference build, and a version
  number. A change to it is a new index version, never an edit.
- **A history driver in the repository.** The 2026-10-05 cells ran from scratch scripts
  (`drive_history.py`, which registers two command shapes with `compare_tools`). They
  move into `explorations/benchmarks/realtree/history.py`, reading the manifest, with
  tests.
- **Projection.** `timeline.py` computes component ratios, the index, and its bootstrap
  interval per build per platform from the history cells, and records the manifest
  version.
- **The page.** `report_html.py` adds the chooser and the bar mapping.
  A job-to-component table lives beside the manifest.
- **Records.** Each experiment’s projection gains `index_components`, derived from its
  primary job and platform through that table.

## Implementation Plan

### Phase 1: Manifest, Driver, and macOS Index

- [ ] Write `index-suite.json` with the components, commands per build era, weights, and
  reference build agreed in review.
- [ ] Move the history driver into the harness with tests: command shapes per era,
  answer-check groups, anchor, alternation, and internal-storage checks.
- [ ] Run the macOS cells for the 13 milestone builds: K, with the default, summary,
  repeat and content components; G, with scale.
- [ ] Project the index and its interval; add the chooser, the headline, and the faded
  bars to the page.
- [ ] Publish it as the macOS index, labeled as such.

### Phase 2: Linux

- [ ] Hand the same builds and manifest to a Linux host.
  Run the Linux cells and project the combined index.

## Testing Strategy

- Unit tests:
  - the index arithmetic (weights, renormalization over missing components, symmetry of
    log ratios);
  - the bootstrap;
  - the job-to-component mapping;
  - the chooser’s default.
- Drift checks: `make perf-report-check` re-derives the index from the committed cells,
  so the page cannot assert a number the cells do not produce.
- A manifest check fails a cell recorded against a different manifest version than the
  one it claims.

## Rollout Plan

Refresh at each release and at the end of a campaign, not per experiment.
A full refresh times every component for every build on both platforms, which takes
hours of machine time.
Release notes quote the unified score with its interval and link the components.

## Open Questions

- **Weights.** Are 30/15/15/15/15/10 right, or should the default command on a
  repository weigh more?
  And should macOS and Linux be equal, or weighted by where fdu runs?
- **Reference build.** Should it be the latest release (moving), or a fixed pin such as
  the pre-work binary, so that scores compare across index versions?
- **More scenarios.** Should watch mode, the opened-root second report, or the Python
  surface be components?
- **The index as a gate.** Should a release require a score no worse than the previous
  release’s, beyond the per-change accept rule?
- **The case against.**
  - A single score invites optimizing the score.
  - Any weighting is a judgment that suits some users and not others.
  - The full suite costs hours per refresh.
  - The mitigations are pre-registration, versioned manifests, always-visible
    components, and refreshing only at checkpoints.
    Whether they suffice is the review’s call.

## References

- [fdu design principles: Performance](../../architecture/fdu-design-principles.md#performance)
- [The performance loop](../../guides/performance-loop.md)
- [The performance evidence report](../../reports/report-2026-08-20-fdu-performance-evidence.md)
- [The loop history](../../reports/report-2026-08-14-performance-campaign-status.md)
- [Campaign-2 plan](plan-2026-08-23-fdu-performance-campaign-2.md)
- Bead `fdu-cpbx`

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
