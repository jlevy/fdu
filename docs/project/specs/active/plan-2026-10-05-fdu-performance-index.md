# Feature: The fdu Performance Index

**Date:** 2026-10-05 (last updated 2026-10-08)

**Author:** fdu project, with Claude Code assistance

**Status:** Phase 1 merged; Phase 2 open.
The maintainer has set the suite and weights below.
Phase 1 merged with [#176](https://github.com/jlevy/fdu/pull/176), which publishes an
exploratory macOS score labeled with the components it covers, and froze version 1 of
the manifest. Phase 2, the Linux cells, is `fdu-iwmq`; a quotable macOS score waits on
`fdu-bkj2`.

## Overview

One pre-registered number that says how fast fdu is across everything it is for, with
every component it is made of published beside it.
The **fdu performance index** combines a fixed suite of components by a weighted sum of
log cost ratios (runtime, and peak memory for the memory component).
Each component is one user-facing scenario on one platform, or peak memory, and each
ratio is measured within one interleaved session against a fixed reference build.
The charted page shows the index as its headline line, and a chooser switches the same
chart to any single component.

## Goals

- **One score that reflects what matters.** It covers a first run with empty caches, the
  default tree view, the summary view, code and document analysis with the content cache
  cold and warm, revalidating a saved index, an opened root, scale, and memory, on macOS
  and on Linux.
- **Every optimization target is exercised.** Every job the loop measures maps to a
  component whose measurement runs that job’s code path, so every kept change can move
  the score. Today the 1M-tree line cannot show Linux, summary, cache, or `.gitignore`
  work, which is why the green bars and the line disagree.
- **No chaining across sessions.** Every ratio comes from one interleaved session.
  Compounding per-experiment effects claims about 7,700× over the 47 kept wall-time
  changes, and far more if remeasurements are counted again, so the index never does it.
- **Fixed before measuring.** The suite, the commands, the weights, and the reference
  build are recorded before a component’s cell runs, and change only by a versioned
  revision.
- **Every component stays visible.** The page’s chooser shows each component on the same
  axis, so no regression hides inside a good average.

## Non-Goals

- **Peer comparison.** The index measures fdu against itself.
  Rankings against dust, dumac, pdu, diskus, and the rest stay in the tool comparisons.
- **The accept rule.** Single changes are still decided by their own paired cell and the
  3% rule. Using the index as a gate on changes is an open question, not this spec.
- **A cold OS cache and bare metal.** The index is warm-steady and uses the hosts
  available; components for a cold OS cache (a first read from disk) and for bare metal
  wait for hosts that can measure them.
  This is distinct from the cold-cache component, which empties fdu’s own caches.
- **Tail latency.** `p95_over_median` at 12 to 20 rounds is an order statistic, not a
  stable population figure.
  It stays in the records, outside the index.

## Background

- **The loop judges each change on its own job, tree, and platform.** That is right for
  a verdict and wrong for a summary.
  Of the 74 accepted changes at least 3% better on their primary metric:
  - 43 were judged on jobs other than `cold-scan-index` and `default-tree`: the summary
    view, the content cache, revalidation, content queries, snapshots, and the opened
    root;
  - 30 were measured on Linux;
  - 4 were judged on peak memory;
  - 17 record no changed lines: checkpoints, validations, and determinations that
    re-measure earlier work.
- **The history cells of 2026-10-05** time 13 milestone builds in one session on two
  trees on the internal SSD. They and their description arrive on `main` with
  [#169](https://github.com/jlevy/fdu/pull/169), under
  `docs/project/reports/performance-evidence/history/` and in the evidence report’s
  “Reading the Charted Page”.
  - The generated 1M-entry tree: 10.8×, all of it from H1 and bulk metadata.
    Every later build is level, because macOS metadata calls set the time on a tree four
    times the vnode limit.
  - The Linux v6.12 source tree: 3.3×, with a rise at 0.1.0 when `.gitignore` reading
    arrives and steps down after it.
  - Neither cell sees the summary view, the cache, content, or Linux.
  - Both cells were measured before this spec existed.
    They are adopted as the cold-cache and scale components because their commands and
    cache state match those components exactly.
    The other 11 cells were timed under the driver branch’s earlier text of this spec,
    and are admitted on the same ground: what each ran equals what version 1 asks, and
    each records the text it was timed under (see Measurement).
- **The project’s own rules name the dimensions:**
  - every output surface is a benchmark job
    ([design principles](../../architecture/fdu-design-principles.md#every-output-surface-is-a-benchmark-job));
  - a warm path that loses to a cold scan is a defect, and a measurement is evidence
    about its own regime
    ([design principles: Performance](../../architecture/fdu-design-principles.md#performance));
  - a speedup bought with memory is not free
    ([the performance loop: What we measure](../../guides/performance-loop.md#what-we-measure)).
- **The index’s form is not new.** The 2026-08-14 performance explorer (branch
  `codex/performance-research-white-paper`) computed a weighted latency index over
  Linux/macOS × cold/warm cells as `1 − exp(Σ w · ln(candidate ÷ control))`. That is the
  same form, over fewer cells.

## Design

### The Formula

For build *b*, component *c* with weight *w_c*, and reference build *R*:

- **job ratio**: *b*’s runtime relative to *R* on one job of a component, taken as the
  paired figure from that job’s interleaved session, the median over adjacent pairs of
  *b*’s time divided by *R*’s. *R*’s own ratio is 1. Memory is the exception, a ratio of
  medians, described under Memory below;
- **component ratio** *r_b,c*: the job ratio when the component has one job, and the
  geometric mean of its jobs’ ratios, equally weighted, when it has several;
- **index** *I_b* = exp(Σ_c *w_c* · ln *r_b,c* ÷ Σ_c *w_c*), summed over the components
  *b* has. It is a relative cost: runtime for every component except memory, which is
  peak memory. 1.0 is the reference build, and 0.5 means half its cost on the weighted
  suite;
- **score** = 1 ÷ *I_b*. “3.4× better than the first build” is the ratio of two builds’
  scores. That ratio is valid because each build’s ratio to *R* comes from a session in
  which both were measured side by side, so *R*’s own speed cancels.

The exponent is a weighted sum of log cost ratios.

**Why not a plain weighted sum?**

- A sum of raw times lets the slowest benchmark set the score: a 70 s scan would
  outweigh every 160 ms job combined.
- A weighted arithmetic mean of ratios treats a 2× slowdown (+100%) and a 2× speedup
  (−50%) unequally.
- Summing logs is scale-free and symmetric: on two components of equal weight, doubling
  one’s speed and halving the other’s cancel exactly.

**Interval.** Each job’s 95% paired interval is converted to a standard error in log
space, and the errors are combined as independent, which they are, each job being its
own session. The memory component has no interval of its own, and is not independent of
the cells it is read from; it contributes its point value only.
The interval of a ratio between two builds’ scores combines their two log-space errors
the same way.
Treating the two as independent is conservative: they share their reference
sessions, so the true interval is narrower.

**Memory.** Peak RSS enters as a ratio of medians to the reference build, not a paired
figure, since the harness records peak RSS per arm.
Each component counts once (a component with two jobs averages them first), and each
score line averages its own components: the full score’s memory reads every measured
component, and the partial score’s reads only the partial score’s components, so every
build on a line is averaged over the same mix.

### The Suite

Each component is a command on a fixed tree, run as each build shipped it, with its
cache state stated exactly.
`.gitignore` is read by default where a build supports it, and not equalized where it
does not. Trees:

- **K**, the Linux v6.12 source tree from `git clone --depth 1 --branch v6.12`: about
  92k entries and 358 `.gitignore` files, below macOS’s vnode limit;
- **G**, the generated 1M-entry tree from the `balanced` recipe.

Weights per platform, set by the maintainer:

| Component | Command, tree, and cache state | Why it matters | Weight |
| --- | --- | --- | ---: |
| Cold cache | `fdu --cache off PATH` on K | Every user’s first run of a tree | 15% |
| Warm-cache content: code | `fdu --view code --analyze code PATH` on K, content cache filled by an untimed run just before | Content analysis is where fdu’s cache pays | 10% |
| Warm-cache content: documents | `fdu --view documents --analyze words PATH` on K, filled the same way | The same, for prose and documents | 10% |
| Default tree view | `fdu PATH` on K, default cache policy, steady state | What most users run, repeatedly | 15% |
| Summary view | `fdu --view summary PATH` on K, default cache policy, steady state | The du-replacement total | 7.5% |
| Code view | `fdu --view code --analyze code PATH` on K, caches empty | Code metrics by language | 7.5% |
| Documents view | `fdu --view documents --analyze words PATH` on K, caches empty | Prose and document metrics | 7.5% |
| Multi-view content report | `fdu --view code,documents,languages --analyze code,words PATH` on K, caches empty | One scan, many views | 5% |
| Warm metadata cache | two jobs on K: `cold-open-save`, a timed first run that scans and writes the snapshot; and `warm-revalidate`, the probe’s `revalidate` mode loading that snapshot and reconciling it, after an untimed run wrote it | Saving, loading, and revalidating an index | 7.5% |
| Opened root | two jobs on K: the probe’s `opened-second-report` and `delta-apply-large` | Opening a root, discovering it, reporting again, and applying a change | 5% |
| Scale | `fdu --cache off PATH` on G | Very large trees, where peers compete hardest | 5% |
| Memory | Peak RSS of each score line’s own components, each counted once | A speedup bought with memory is not free | 5% |

Every job is timed whole-process, from launch to exit, so a probe job’s timed region
includes the discovery, snapshot load, and snapshot save around its own component timer.
It also includes the probe’s verification walk, its oracle, in every build: builds
before 0.1.0 cannot turn it off, so the driver leaves it on for all of them and builds
compare like for like.
That cost is harness instrumentation and varies by build, across the 12.5% of the weight
the probe jobs carry; separating it is tracked as `fdu-92bg`, and until then every
score, quoted or not, includes it.
The commands are written as v0.3.0 spells them; earlier builds use their own spelling of
the same request (`--no-cache` before 0.1.0, for example), and a build that cannot make
the request at all has no ratio for that job.
The driver records the exact argv per build.

The split is the maintainer’s: 15% cold cache, 20% warm-cache content, and the rest
across the views and the other optimization targets, with scale and memory kept in.
Content analysis carries 40% in all: 20% warm, 15% for the two cold views, and 5% for
the multi-view report.

“Cold cache” means fdu’s own caches are empty, not a cold OS disk cache: the OS keeps
the tree’s metadata cached, as in every record so far.
A view run with the default cache policy is timed in its steady state, after the
warm-ups. For the default tree view on 0.2.0 or later that is a full scan, the same
measurement as cold cache, because `--cache auto` writes no snapshot for a one-shot
metadata report; a build that reuses a snapshot there is credited for it.
The two components stay separate on purpose: they are different user situations, a first
run and a repeat run, that the current design happens to serve the same way, and a
design that served repeat runs differently should show in the score.

### Every Optimization Target Is Exercised

Every benchmark job the loop measures maps to a component, and the component’s timed
region runs that job’s code path on a scored tree, through the command line or the probe
mode that exercises it.
A recorded job with no component fails `make check`, which forces a new target into the
suite, with a weight, before its first verdict is published.
Labels alone are not coverage, so the table below names, for each job, what in the
component exercises it.

| Job in the record | Component | What exercises it |
| --- | --- | --- |
| `cold-scan-index`, `cold-scan-producer`, `adaptive-scan-index`, `default-tree-first`, `content-disabled` | Cold cache | A first-run scan of K |
| `default-tree`, `cli-default-tree` | Default tree view | `fdu PATH`, repeated |
| `aggregate-summary`, `rich-summary-report`, `rich-summary-open-pipeline`, `rich-summary-shared-openers`, `selected-allocated-total` | Summary view | `--view summary` |
| `content-basic`, `code-sloc` | Code view | `--view code` with analysis, cold |
| `markdown-prose`, `text-prose` | Documents view | `--view documents` with analysis, cold |
| `content-cache-hit`, `code-sloc-cache-hit` | Warm-cache content: code | `--view code` with the content cache filled |
| `document-cache-hit` | Warm-cache content: documents | `--view documents` with the content cache filled |
| `content-query` | Multi-view content report | Several content views from one scan |
| `cold-snapshot-save`, `cold-open-save` | Warm metadata cache | The timed `cold-open-save` job, which scans and writes the snapshot |
| `warm-revalidate`, `warm-snapshot-load` | Warm metadata cache | The timed `warm-revalidate` job, which loads the snapshot and reconciles it |
| `opened-discovery`, `opened-second-report`, `index-second-report` | Opened root | The timed `opened-second-report` job, which opens, discovers, and reports again |
| `delta-apply-large`, `delta-apply-batched` | Opened root | The timed `delta-apply-large` job |
| Peak RSS on any job | Memory | Peak RSS of every measured component |

The job table is a separate file from the weighted suite, so mapping a new job to an
existing component is not a new index version; adding or reweighting a component is.

A change measured only on Linux moves the Linux half of the score, and one measured only
on macOS the macOS half.
Both halves are in the score, so a platform-specific change shows too.

- **Platforms.** macOS and Linux each carry 50% of the total.
- **Missing platform.** A one-platform score is labelled with its platform (“macOS
  score, 12 of 12 components”) and never presented as the full index, which needs both
  platforms.
- **Missing capability.** A build without a component’s capability (the pre-work binary
  has no content views) has no ratio for that component, and its score covers fewer
  components. A score always carries its coverage, the share of the suite’s weight it
  includes.

### Measurement

- **One history cell per job and platform.** A component with two jobs has two cells.
  Each cell records its component, its job, the manifest version, and a digest of its
  component’s definition (tree, command, cache state, jobs); the projection refuses a
  cell whose digest does not match the manifest, so a component redefined in place
  cannot reuse cells measured under the old definition.
  A cell stamped after timing, because the manifest’s text changed while the request did
  not, records the text it was timed under and that it was stamped by argv equivalence.
  All builds in a cell are timed interleaved, anchored on the reference build:
  - order alternated, 3 warm-ups, at least 12 rounds, 20 where the cell stays under an
    hour, and 20 whatever the duration for a cell meant to stand behind a quoted score;
  - answers checked before timing within each capability group;
  - trees and binaries on internal storage (results may go elsewhere);
  - the harness’s quiet gate unchanged, and the regime recorded.
- **Regime.** A score from any cell measured on an uncontrolled host is exploratory, and
  the page and every figure quoting it say so (“exploratory, uncontrolled host”), as the
  [performance loop’s host-pressure regimes](../../guides/performance-loop.md#host-pressure-regimes)
  require. A score quoted in release notes or a README must come from cells timed quiet
  or controlled-interactive, at 20 rounds, and recorded at a confirming stage
  (`--stage discovery` or `held-out`; the driver defaults to `exploratory`); the page
  labels a score with any cell short of 20 rounds exploratory and names the shortfall
  (“exploratory, 12 rounds”), as it does a score with any cell at the exploratory stage.
- **Where.** macOS cells run on the maintainer’s Mac.
  Linux cells run on a Linux host, for example a cloud session handed the same builds
  and manifest.
- **Reference build.** The latest release, v0.3.0, re-pinned only by a versioned
  revision of this spec.
  Each cell’s anchor is the reference build, so every ratio is within-session.

### The Page

- **The chooser.** The runtime-over-time chart gets a metric chooser:
  - **Unified score** (default): the full score, each platform’s score over every
    component, from the first build that supports every component; builds before it are
    drawn as a partial score, visibly distinct, with their coverage;
  - each component on its own, including memory.
- **The headline** states the unified score with its interval and coverage: the full
  score from the first fully covered build to the current release, and, separately, the
  partial score from the pre-work binary.
  The full index is the full scores of both platforms combined.
- **The experiments panel** marks each bar by the component and platform its primary job
  maps to. A bar that does not count toward the chosen metric is drawn faded, and its
  tooltip says what it counts toward.
  The chart then stops implying that every green bar should move the line above it.

### Components

- **The suite manifest** at `explorations/benchmarks/index-suite.json`: components,
  weights, trees, cache states, the reference build, and a version number.
  Version 1 was revised during this spec’s review, before any of its changed components
  was measured, and is frozen when [#176](https://github.com/jlevy/fdu/pull/176) merges;
  from then on a change to a component or a weight is a new version, never an edit, and
  each cell’s definition digest catches an edit in place.
- **The job table** at `explorations/benchmarks/index-jobs.json`, mapping every recorded
  job to a component.
- **The history driver** at `explorations/benchmarks/realtree/history.py`, with tests.
  It reads the manifest, derives each build’s argv for each job by probing the build,
  implements each cache state, times every job whole-process, and writes one history
  cell per job.
- **Projection.** `explorations/benchmarks/realtree/perf_index.py` computes component
  ratios, the index, its interval, and its coverage per build per platform, and
  `timeline.py` refuses to project while any recorded job is unmapped.
- **The page.** `report_html.py` adds the chooser, the score headline, and the bar
  mapping.

## Implementation Plan

### Phase 1: Manifest, Driver, and macOS Index

- [x] The manifest and the job table, as above.
- [x] The history driver in the harness, with tests: argv per build era, unsupported
  capabilities, cache states, probe-job components, the anchor, alternation, answer
  checks, and internal-storage checks.
- [x] The macOS cells for the 13 milestone builds, one per job: cold cache, the default
  tree, summary, code, documents, both warm-cache content components, multi-view, both
  warm-metadata jobs, and both opened-root jobs on K; scale on G; memory read from them.
- [x] The completeness check in the projection, and its test.
- [x] The index, its interval, and its coverage; the chooser, the headline, and the
  faded bars.
- [x] Publish it as the macOS index, labeled as such.

### Phase 2: Linux

- [ ] Hand the same builds, manifest, and driver to a Linux host.
  Run the Linux cells and project the full index.

## Testing Strategy

- Unit tests:
  - the index arithmetic: weights, renormalization over missing components, the symmetry
    of log ratios at equal weights, and the interval;
  - the completeness check, with an unmapped job;
  - the driver’s argv selection, unsupported handling, and cache states;
  - the chooser’s default.
- Drift checks: `make perf-report-check` re-derives the index from the committed cells,
  so the page cannot assert a number the cells do not produce.
- A cell records the digest of its component’s definition, and the projection refuses a
  cell whose digest does not match the manifest’s; a test edits a definition and expects
  the refusal.

## Rollout Plan

Refresh at each release and at the end of a campaign, not per experiment.
A full refresh times every component for every build on both platforms, which takes
hours of machine time.
Release notes quote the unified score with its interval, coverage, platform, and regime,
and link the components; only a score whose every cell was timed quiet or
controlled-interactive, at 20 rounds, at a confirming stage, is quoted there.

## Open Questions

- **Platform weights.** Should macOS and Linux stay equal, or be weighted by where fdu
  runs?
- **More scenarios.** Should watch mode or the Python surface become components?
- **The index as a gate.** Should a release require a score no worse than the previous
  release’s, beyond the per-change accept rule?
- **The case against.**
  - A single score invites optimizing the score.
    Concretely: on builds since 0.2.0 the cold-cache, default-tree, and scale components
    are all full scans, at least 35% of the weight, and the summary view is a fourth at
    7.5%, so the cheapest way to raise the score is the walker, not the cache and
    content work the weights favour.
  - Any weighting is a judgment that suits some users and not others.
  - The full suite costs hours per refresh.
  - The mitigations are pre-registration, versioned manifests, always-visible
    components, coverage beside every partial score, and refreshing only at checkpoints.

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
