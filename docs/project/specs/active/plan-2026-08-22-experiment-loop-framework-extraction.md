# Feature: Extracting the Experiment Loop as a Reusable Framework

**Date:** 2026-08-22

**Author:** fdu project

**Status:** Draft

## Overview

Two projects have now run the same research method against different subjects.
fdu built the heavy form: 64 soft-schema experiment artifacts, paired interleaved trials
with bootstrap intervals, a compiled contract, and generated views whose drift fails
`make check`. Metabrowser ([PR #66](https://github.com/jlevy/metabrowser/pull/66))
rebuilt the method from scratch for a full-stack web app in one 772-line script:
hand-written schema, median-and-range statistics, a browser a person drives by hand —
and independently converged on the same artifact shape, the same accept-rule structure,
and several of the same hard-won rules, word for word.

That convergence is the extraction evidence.
What both loops kept is the framework; where they differ is a knob, because each choice
was right for its subject.
This plan extracts the shared core as a skill plus a small set of individually usable
pieces, deliberately under-built: metabrowser proves the method transfers with almost no
shared code, so every piece must earn its place by replacing something an adopter would
otherwise hand-roll.

The target domains are wider than performance.
The same loop fits searching for optimal geometric packings (hypotheses are algorithms;
the artifact is the research report on trying one, with the best score found and a
pass/fail on beating the standing record), strategy portfolios for a proof or an
algorithm (explored until resolved or a budget runs out), and any campaign where
systematic hypotheses meet an iteration loop, a record, and generated reporting.

## Goals

- A skill carrying the invariant core: the loop, the record discipline, and the named
  failure modes, so an agent starting a campaign in any domain inherits the method
- A small package of separable pieces — contract base, two statistics tests, generated
  views, identity and reference checks — each adoptable alone and skippable alone
- Support for all four comparison shapes the known domains need: paired change,
  condition ranges, score against a standing record, and categorical determination
- A hypothesis registry made of artifacts, with the predicted criterion and the
  instrument declared up front, so pre-registration is checkable rather than honoured
- fdu re-hosted on the extraction with byte-identical generated views, and metabrowser’s
  loop reproducible on it at a fraction of its hand-rolled line count

## Non-Goals

- **Not a benchmark runner or a measurement harness.** The framework never executes the
  workload. fdu’s `measure.py` and metabrowser’s `serve`/`probe` stay where they are; how
  to produce a valid sample is exactly the domain knowledge.
- **Not a statistics library.** Two tests — paired bootstrap interval, and
  median-with-range overlap — chosen by declared evidence tier.
  Nothing else.
- **Not a replacement for softschema.** Validation, compilation, and the self-describing
  envelope stay there; this adds the experiment contract and the views.
- **Not a monolith.** A loop that wants only the artifact contract and a ledger must not
  inherit the harness, the statistics, or the report renderer.
- **Not a rewrite of either project’s committed record.** If the generic contract cannot
  validate fdu’s 64 artifacts unchanged, the contract is wrong.
- **Not a CI gate.** “An exploration answers a question once; a benchmark defends an
  answer forever” (metabrowser’s README) — only the second belongs in a release gate,
  and this framework is for the first.

## Background

### What fdu built

The full analysis of fdu’s mechanisms is in
[the performance loop](../../guides/performance-loop.md) and
[the instrumentation playbook](../../guides/performance-instrumentation-playbook.md);
what matters here is which mechanisms carried weight:

1. **One artifact per round, split soft-schema.** Frontmatter carries what a tool reads;
   prose carries the reasoning.
   31 accepted, 28 rejected — the record leads with its failures, and the refutations
   are what stop the next agent re-running a dead end.
2. **The contract compiled from a model** (`make perf-schema`), with drift checked.
3. **The measured half never retyped.** `perf-record` lifts numbers from the run JSON;
   the operator supplies only hypothesis, complexity, decision, and one sentence.
4. **Views generated, generation checked.** The ledger and charted report rebuild from
   artifacts, and `perf-report-check` fails `make check` on drift.
   The record cannot be edited into a better story, because the story is generated.
5. **Paired and interleaved measurement**, decided on paired differences — on 21% of
   recorded entries the paired change and the ratio of medians disagree by more than two
   points.
6. **Four separated evidence fields** (`passes_acceptance`, `ci_excludes_zero`,
   `direction`, `noninferiority`) after one `significant` boolean proved unable to
   distinguish a regression from a null result.
7. **Regime recorded with every number**, and claims never extrapolated across regimes.
8. **The failure modes written down as failures**, each with the run that produced it.

### What metabrowser rebuilt, and what it added

Metabrowser’s loop (`explorations/` on the PR branch) is the method at minimum viable
weight: 3–6 runs per condition, median with range, overlap instead of bootstrap, a
schema written by hand because eight result fields do not need a compiler, and a browser
driven by a person because committing browser automation is a dependency decision, not a
harness detail. Four rounds in, it has confirmed three hypotheses, left one honestly
unresolved, and caught two silent methodology defects.

What it added that fdu lacked:

- **Instrument validity as a refusal, not a caveat.** Six runs were taken in a browser
  pane that was 0×0; every layout-dependent number was measured against nothing while
  the timings looked reasonable.
  `record` now refuses a viewport under 900×600. The general rule: the instrument must
  prove it was measuring something before a run may enter the record.
- **The `unresolved` decision.** H3 was instrumented, never reproduced, and recorded as
  unresolved with the reason — worth more than an answer, and a distinct state from
  fdu’s `blocked` (cannot test yet).
- **Each hypothesis names its instrument** in the registry, and “a hypothesis whose
  instrument does not exist yet is marked blocked rather than measured badly.”
- **Provenance filled automatically at record time** — commit, dirty flag, port, corpus,
  and the walk duration read back out of the server’s own log — because “a number nobody
  can trace is a number nobody can defend.”
- **Correction without rewriting.** exp-001’s absolute numbers were invalidated by the
  viewport defect; the artifact was annotated rather than replaced, because both its
  conditions met the same defect and its *comparison* stands.
- **A costed floor for the extraction.** Of its 772 harness lines, roughly half —
  compare, report generation, record-time provenance, the schema — are generic method,
  not web-app knowledge.
  That half is what the package must replace, and what adopting should stop costing.

### What the two loops agree on

Everything both kept, having built independently, is the invariant core:

| Invariant | fdu | metabrowser |
| --- | --- | --- |
| One soft-schema artifact per round; frontmatter only what a view reads | `experiment.py` docstring | schema header comment says it verbatim |
| Failures recorded like successes | 28 of 64 rejected | rejected variants written up inside accepted rounds |
| Criterion named before measurement; post-hoc metric switch is never an accept | pre-registration rule, exp-051 | “The metric is named before the measurement” |
| A number without its spread is not a result | 95% bootstrap interval | “A median without its range is not a result” |
| Verdict = arithmetic plus one written judgment | accept rule, fourth clause | accept rule, fourth clause |
| Views generated from artifacts, never edited | `perf-ledger`, `perf-report` | `run.py report` |
| Regime recorded with the number | `os_cache`, virtualization | `cold` flag; “a fresh server is not a cold scan” |
| Shared hypothesis numbering; “no id ever means two things” | loop guide | plan registry, same sentence |
| Provenance captured at the source, never retyped | binary sha256, run JSON | commit, dirty, port, walk log |
| Nothing in CI | “a timing gate on a shared runner measures the runner” | “an exploration answers a question once” |
| The record is corrected, not rewritten | append-only policy | exp-001 annotated |
| An independent check that the result is real, before it may be good | per-trial oracle digest | viewport floor, visibility state |

### Where they differ — knobs, not defects

| Choice | fdu | metabrowser | Why both are right |
| --- | --- | --- | --- |
| Statistics | paired bootstrap CI | median + range overlap | 12+ interleaved trials vs 3–6 hand-driven runs |
| Interleaving | per-trial | sequential conditions | automated harness vs person driving a browser |
| Schema origin | compiled from Pydantic | hand-written YAML | 40+ fields vs 8 |
| Drift gate | in `make check` | none | 64 artifacts and two generated pages vs 4 and one |
| Measurement | fully automated | probe pasted by hand | dependency policy differs |
| Harness | 9,600 lines | 772 lines | subject complexity differs |

The one gap both share: **the hypothesis registry is hand-maintained in both** — a
Markdown table, free-text ids on artifacts, no check that a referenced id exists, status
updated by hand. Both loops applied their discipline to everything except the registry.

## Design

### Principle: a small core, and pieces that compose

The design rule, stated once: every piece must be adoptable alone and skippable alone,
and each must replace something an adopter would otherwise hand-roll.
Metabrowser is the calibration — the method worked with no shared code at all, so the
package justifies itself only by making rung one of the ladder below cost tens of lines
instead of hundreds.

### The adoption ladder

| Rung | What you take | What you get | Who lives here |
| --- | --- | --- | --- |
| 0 | The skill + softschema | The method, the artifact split, validation | any one-question loop |
| 1 | + contract base and views | Ledger and report generated, id/reference checks | metabrowser-scale loops |
| 2 | + run capture | Measured half lifted from a run document, never retyped | loops with a machine-readable harness |
| 3 | + drift gate and full statistics | Paired bootstrap, `check`-enforced publishing | fdu-scale campaigns |

Each rung is optional and none implies the next.

### The contract: invariant spine, open flesh

`Experiment` keeps the shape both projects converged on — `id`, `title`, `date`,
`hypotheses`, `subject`, `method`, `results`, `complexity`, `verdict` — with three
openings:

- **`subject` is a project-supplied payload.** fdu’s tree digest and host, metabrowser’s
  corpus and viewport, a packing problem’s instance definition.
  The framework requires only that it exist and validate against the campaign’s model.
- **`method` carries how to re-run this**, at whatever precision the domain affords:
  binary hashes for fdu, commit-plus-port for metabrowser, a code reference (repo,
  commit, entry point) and compute budget for a packing or proof search.
- **`results` is a list of typed result shapes** — see below — rather than one shape.

### Four comparison shapes

The known domains need exactly four, each small, each usable beside the others in one
artifact:

| Shape | Fields | Decided by | Domain that forced it |
| --- | --- | --- | --- |
| `paired` | control/candidate medians, `change_pct`, CI, evidence flags | interval vs threshold | fdu |
| `conditions` | per-condition median + range, `overlapping` | ranges at n≥3 | metabrowser |
| `record` | score, direction, `standing_best`, `beat_record` | comparison to a standing best | packing search |
| `determination` | outcome from a declared enum, e.g. proved / refuted / no-progress | the outcome itself | proof search |

A packing round typically carries a `record` result (best density found, against the
best ever recorded for that instance) and a `determination` (did it beat the record:
pass/fail). A proof round carries a `determination` plus cost metrics.
The campaign declares which shapes its verdicts may rest on, and — following the rule
both projects already enforce — the shape and criterion are named in the hypothesis
before anything runs.

### Metric roles

Declared once per campaign; each metric carries id, unit, direction, and role:

| Role | Meaning | fdu | metabrowser | packing | proof |
| --- | --- | --- | --- | --- | --- |
| `outcome` | the accept rule scores it | `wall_ns` | the hypothesis’s named metric | best score | the determination |
| `cost` | qualifies the win | `cpu_ns` | `transferred_kb` | CPU-hours | budget spent |
| `guard` | independent limit; breach rejects | `peak_rss_bytes` | “nothing else moved the wrong way” | validity checker passes | kernel accepts the proof |
| `mechanism` | explains, never decides | `component_ns`, faults | `render_spans`, `long_tasks` | iterations, restarts | subgoals closed |

The guard role generalizes both projects’ correctness checks: fdu’s oracle digest,
metabrowser’s viewport floor, a packing solution’s overlap check, and a proof checker
are the same slot — **the independent proof that the result is real, before it may be
counted good.** A run failing a guard is invalid, not merely rejected, and the
framework’s record step refuses it the way metabrowser’s `record` refuses a collapsed
viewport.

### Evidence tiers

fdu’s `campaign_stage` (exploratory → discovery → held-out) and metabrowser’s whole loop
(which lives at exploratory) unify into a declared tier per experiment.
The tier decides which statistics are required and what the artifact may claim:
`conditions` overlap suffices at exploratory; a confirmatory claim needs the paired
bootstrap or a pre-registered equivalent.
A campaign that never leaves exploratory is legitimate — metabrowser is one — and its
artifacts say so, which is the point: the tier is recorded, so a reader knows what kind
of evidence they are holding.

### Decisions

The base vocabulary merges both projects’ and adds one state the search domains need:

- `accepted` / `rejected` — measured, and the claim resolved
- `unresolved` — measured, could not be resolved; the reason is the finding
  (metabrowser’s H3)
- `blocked` — cannot be tested yet; names the missing instrument or regime
- `abandoned` — a search explored under a budget and stopped: records budget spent, the
  best result reached, and what would justify reopening.
  This is the packing and proof-search verdict, distinct from `rejected` because the
  claim was not refuted — the search ran out of promise.
- `superseded`, `baseline`, `in-progress` — bookkeeping, as in fdu

### The registry becomes artifacts

One small soft-schema artifact per hypothesis: stable never-reused id, the claim stated
so it can be wrong, the **predicted criterion** (which metric or determination, which
direction), the **instrument** that would show it (metabrowser’s addition), the regime
it applies to, and the registration date.
Status is generated from the experiments that reference the id, exactly as the ledger is
generated from artifacts; a referenced id that does not exist fails the build.

This closes the one gap both projects share, and it makes the pre-registration rule
enforceable: accepting on an alternative criterion is legitimate only when the registry
artifact declared it, with a commit date behind the declaration.

### Components

| Piece | Extracted from | Standalone use |
| --- | --- | --- |
| Contract base + result shapes | fdu `experiment.py`, metabrowser `experiment.schema.yaml` | validate artifacts, nothing else |
| Statistics: paired bootstrap; range overlap | fdu `ledger.py`; metabrowser `_summarize` | score a comparison from raw numbers |
| Artifact writer | fdu `record.py`, metabrowser `cmd_record` provenance | write one artifact from a run + operator judgment |
| Ledger + report views | fdu `summary.py`/`timeline.py`/`report_html.py`, metabrowser `cmd_report` | regenerate views from an artifact directory |
| Identity + reference checks | fdu `check_identifiers`, new registry check | pre-commit or build-time check |
| Registry contract + generated status | new; gap shared by both | registry alone, even without experiments |

Stays behind: fdu’s `measure.py`, `compare_tools.py`, `provenance.py` (2,154 + 1,496 +
689 lines) and metabrowser’s `serve`/`probe`/`probe-server` — the harnesses are the
domain knowledge.

### API Changes

None to fdu’s engine, CLI, or Python binding.
The `perf-*` targets keep their names and behaviour and become thin wrappers over the
framework with fdu’s adapter and config.

## Implementation Plan

### Phase 1: The core pieces and the skill

- [ ] Contract base with open `subject`, re-run provenance, the four result shapes, the
  merged decision vocabulary, and metric roles; fixtures from all four domains (an fdu
  artifact, a metabrowser artifact, a packing round, a proof round)
- [ ] The two statistics tests, parameterized by threshold and tier; evidence flags
  derived from intervals or ranges, never stored opinions
- [ ] Artifact writer: measured half from a run document or a pasted payload, judged
  half from the operator; validity guards refuse rather than annotate
- [ ] Ledger and report views over an artifact directory, chart set and columns as
  config; identity check across the set
- [ ] Registry artifact contract, generated status view, and the reference check
- [ ] The skill: the invariant core, the ladder, the knobs and how to choose them, and
  every named trap from both projects’ records

### Phase 2: Re-host fdu; prove nothing was lost

- [ ] fdu’s 64 artifacts validate against the generic contract unchanged
- [ ] `perf-record`, `perf-ledger`, `perf-report`, and the checks run through the
  framework; regenerated ledger, projection, and report are byte-identical
- [ ] Convert fdu’s hypothesis table to registry artifacts; the reference check passes
- [ ] Verify the drift gate by mutation: edit an artifact, watch `make check` fail

### Phase 3: Prove the ladder at both ends

- [ ] In metabrowser, rebuild `report`/`compare`/record-provenance on the package and
  count the lines shed; its four artifacts must validate and its report regenerate
  equivalently
- [ ] Run one campaign in a search domain — a packing instance or a proof portfolio —
  using `record` and `determination` shapes, budgets, and `abandoned` verdicts; fold
  back whatever the framework lacked

## Testing Strategy

Unit coverage: the two statistics tests against known inputs; flag derivation from
intervals and ranges; each decision’s required fields (an `abandoned` without a budget
spent is invalid); the identity and reference checks; round-tripping each domain
fixture.

The load-bearing tests are equivalence and refusal: fdu’s committed views regenerate
with an empty diff, and every validity guard is verified by mutation — feed the writer
an invalid run and watch it refuse, following the playbook’s rule that a guard nobody
has watched fail is not yet evidence.

## Open Questions

- **The name.** `expledger` remains the recommendation, with the same case against it:
  ugly compound, “ledger” overloaded by finance, and it names the output view rather
  than the contract and statistics an adopter cannot write themselves.
  `refute` stays the interesting alternative and still reads adversarial.
- **Where the skill lives.** Beside softschema’s own skill in that repository, or as its
  own package; softschema is the one hard dependency either way.
- **One runtime or two.** softschema ships Python and TypeScript; the views and
  statistics doubling that surface is real cost for consumers that are mostly build
  tooling. Python first, and the contract itself stays runtime-neutral YAML.
- **How much of the report renderer travels.** fdu’s 1,431-line SVG page is house style;
  metabrowser’s report is plain Markdown tables.
  The projection is clearly shared; ship the Markdown view, and leave charts as config
  or per-project.
- **Registry back-dating.** Converting existing hypothesis tables wholesale back-dates
  registrations that were never pre-registered.
  Likely answer: converted rows carry `registered: retroactive`, so the enforcement is
  honest from the first real registration onward.

## References

- [The performance loop](../../guides/performance-loop.md) — fdu’s protocol
- [The instrumentation playbook](../../guides/performance-instrumentation-playbook.md) —
  already domain-neutral; the skill’s starting text
- [The experiment ledger](../../reports/report-2026-08-10-fdu-performance-experiments.md)
  and [the evidence report](../../reports/report-2026-08-20-fdu-performance-evidence.md)
  — fdu’s generated views
- [`explorations/benchmarks/realtree/experiment.py`](../../../../explorations/benchmarks/realtree/experiment.py)
  — fdu’s contract
- [metabrowser PR #66](https://github.com/jlevy/metabrowser/pull/66) — the second
  implementation: `explorations/README.md`, `run.py`, and four artifacts
- [The design principles](../../architecture/fdu-design-principles.md) — “Claim Only
  What the Benchmarks Have Shown”; “A Measurement Is Evidence About Its Own Regime”
- [softschema](https://github.com/jlevy/softschema) — the artifact format both loops
  build on

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
