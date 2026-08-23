# Feature: Evidence Scope — Make the Loop Say What a Number Is Evidence About

**Date:** 2026-08-23

**Author:** fdu project

**Status:** Draft. Owns the structural fix; the mechanism landed in
[PR #38](https://github.com/jlevy/fdu/pull/38) is the first slice of Phase 1 and this
plan absorbs the rest.

## Overview

The performance loop is rigorous about whether a number is *real* and silent about what
that number is *evidence about*. Paired trials, interleaved ordinals, bootstrap
intervals, an independent oracle, a fixed accept bar, recorded rejections: all of it
establishes that the measured difference happened.
None of it establishes the population the difference generalizes to.

Every serious error this campaign has made has been in the second category, not the
first. Three are on the record, each measured correctly and generalized wrongly, each
caught only because somebody re-measured on a second subject:

| Claim as first stated | On a second subject | Where it is recorded |
| --- | --- | --- |
| fdu walks 22% faster than `ignore`, “held in every sitting” | **+11.8% slower** on `/usr` | [floor report](../../reports/report-2026-08-23-metadata-walk-floor.md) |
| fdu sits 1.17–1.35× the syscall floor | **1.59×** on `/usr` | same |
| The content roll-up fix is worth −13.40% cold | **−2.38%** on dense real source | exp-064 / exp-065 |

The pattern is one mechanism, not three mistakes.
A subject is chosen for convenience, a number is measured on it correctly, and the
number then travels — into a report, a plan, a README, a frontier ranking — carrying no
trace of the subject that produced it.
The floor report states the trap in its own words: “a uniform corpus flatters fdu
specifically, and the person most likely to be fooled by that is the one who just
measured it.”

This plan makes the scope of a claim a recorded, checked property rather than something
each reader has to reconstruct.

## Goals

- Every artifact carries machine-derived subject properties that decide transfer, not
  prose about them.
- Every accepted result states whether it is known to transfer, known to be
  subject-specific, or untested — and the loop distinguishes those three.
- A number quoted out of the generated views carries its qualifier with it, because the
  views render it inline.
- What the guide requires, the build enforces: provenance, id uniqueness, and stale
  controls fail rather than warn.

## Non-Goals

- Rewriting recorded measurements.
  Numbers already in the record are correct about their own subjects; this plan changes
  what is recorded *alongside* them, never the values.
- A general-purpose statistics upgrade.
  The accept rule’s arithmetic is not the problem and is not touched.
- Multi-subject measurement for every experiment.
  A screening run on one subject stays legitimate — it just stops being quotable as a
  tier-level claim.
- Prose-level citation checking across all documentation.
  Phase 2 renders qualifiers at the source; auditing every hand-written sentence is a
  separate question left open below.

## Background

### What the record does and does not hold

66 artifacts across 20 subjects.
The `Subject` model pins identity precisely — a `fdu-index-record-v1` digest over every
entry’s path, kind, size, mtime, ctime, inode and device — so any reader can tell
whether the tree in front of them is the one measured.

That answers “is this the same tree?”
and nothing else. Until PR #38 the record could not answer “how do I get one?”, and of
the 65 artifacts predating it exactly one named how its subject was built — omitting the
argument that determined the size.

The fields that predict transfer were present the whole time and unread.
exp-064 recorded `tree_max_depth: 16` and 595,728,806 apparent bytes against 26,341,376
allocated.
That second pair is a 22.6× sparse ratio, and `gen_tree.py` writing every file
over 256 bytes with `os.truncate` is why: reading a hole costs nothing, so a cold
content job on that tree is mostly per-file bookkeeping — exactly what the change under
test deleted. Both facts sat in the frontmatter, neither was rendered anywhere a reader
would meet them, and the −13.40% travelled into the campaign plan, the architecture
report’s frontier list, and this PR’s own description as a general figure.

### Why the existing checks did not catch it

- **The accept rule is single-subject by construction.** Median at least 3% better,
  interval below zero, no invalidated sample, complexity worth it.
  Every clause is about one comparison on one tree.
  Nothing asks whether the effect survives a different one.
- **`check_identifiers` warns where it should split.** A duplicate experiment id is
  fatal; hypothesis reuse is only a warning, for a good reason recorded in its own
  docstring — a hypothesis is *supposed* to span experiments.
  But that lenience also covered `H86-observability` sitting beside `H86`, which is
  never legitimate, and the warning printed on every ledger build for weeks with nobody
  reading it. Four artifacts ended up claiming ids that meant two things, one of them the
  id of campaign 2’s centerpiece.
- **The generated views print identity, not scope.** The ledger’s “Reproducing this”
  section listed counts, sizes, digest and host — and offered verification while calling
  it reproduction.

### What PR #38 already landed

The first slice, kept deliberately narrow because it shipped inside a performance PR:
`tree_provenance` and `tree_reconstructible` on `Subject`, `--tree-provenance` /
`--tree-reconstructible` on `perf-record`, the apparent-to-allocated ratio rendered past
2×, an explicit “this tree cannot be obtained again” where no recipe exists, the four id
collisions renumbered to H96–H99, and exp-065 recording the cross-subject measurement.

Both new fields are optional with defaults, so nothing yet *requires* them.
That is the gap this plan closes.

## Design

### Approach

Three things become first-class: the subject’s measurable shape, the claim’s scope, and
the enforcement that keeps both honest.

### Components

**1. `SubjectProfile`, derived rather than described.**

Computed from the run, not typed by a person, and attached to every artifact:

| Property | From | Why it decides transfer |
| --- | --- | --- |
| `sparse_ratio` | apparent ÷ allocated | Near-zero read cost per file inflates any per-file bookkeeping win |
| `mean_depth`, `max_depth` | the walk | Per-file ancestor work scales with it |
| `entries_per_directory` | counts | Decides `getdents` batching and width effects |
| `name_realism` | generated vs observed names | Worth ~15 points against the floor; see the floor report |
| `content_density` | bytes read ÷ bytes apparent | Separates a content-tier denominator from a metadata one |

Two subjects are **materially different** when they differ by more than a declared
margin on at least one property.
That predicate is what Phase 2’s transfer rule tests against, so it lives in code with
tests, not in a reviewer’s judgment.

**Denominator is not the only way a result fails to transfer, and the record already
shows the other one.** exp-064 and exp-065 differ 4.36× in per-file wall saving and only
1.31× in per-file user-CPU saving: the mechanism carries almost intact, and what changes
is how much of it the user waits for — 0.95 of each saved CPU microsecond became wall on
the sparse tree against 0.29 on the dense one, because dense source gives the reader
threads real work to hide consumer bookkeeping behind.
So the profile owes a *critical path* property beside its shape ones, derived from the
run’s own `cpu_ns` and `wall_ns`: the fraction of a change’s CPU saving that reached
wall. Rendered beside the headline it answers the question a percentage cannot — whether
this saving is on the path of the regime the change will ship into — and it is the
property that decides how much of H86’s Linux result should be expected on macOS, where
the aggregate tier is kernel-bound.

**2. `verdict.scope`, a required field with three values.**

- `transfers` — measured on at least two materially different subjects, effect holds on
  both.
- `subject-specific` — measured on two or more, and the effect does *not* hold across
  them. exp-064’s cold figure is the worked example, and this is a legitimate,
  publishable outcome, not a failure.
- `untested` — one subject.
  The default, and honest: most screening runs are this.

`untested` is not a lesser accept.
It is an accept whose scope has not been established, and the difference is that it may
not be quoted outside its own artifact.

**3. The generated views carry the qualifier inline.**

Every headline figure in the ledger and evidence report renders with its scope and the
profile properties that constrain it — `−13.40% (untested: sparse 22.6×, depth 16)`.
People quote from the generated views, so the qualifier travels by copy-paste instead of
by discipline. This is the highest-leverage item in the plan and the cheapest.

**4. Enforcement moves from guide to build.**

- `tree_provenance` required for artifacts dated on or after the cutover; the ledger
  fails without it. Pre-cutover artifacts stay valid and render “provenance unrecorded”.
- A hypothesis id that is a *variant spelling* of another (`H86-foo` beside `H86`)
  becomes fatal, while genuine reuse across experiments stays legal.
  The two cases are distinguishable by prefix, so the check can be tightened without
  breaking the record’s normal shape.
- `verdict.scope` required for every new artifact.

**5. A dense generator mode.**

`gen_tree.py` writes holes, which is right for metadata-tier work and actively
misleading for content-tier work.
Add a mode that writes real bytes, and make content-tier experiments use it by default.
The 15,977-file subject stays available for continuity with exp-064.

**6. Control staleness.**

exp-064’s control was 44 commits behind `main` by the time the result was revalidated,
and nothing said so.
Record the control’s commit distance, and have the ledger flag an accepted-but-unlanded
result whose control has drifted past a threshold.

### API Changes

Additive to `fdu.performance:Experiment/v1` — `subject_profile` and `verdict.scope`.
Pre-cutover artifacts continue to validate, as they did when `tree_provenance` was
added.
No engine or CLI surface changes; this is entirely the measurement harness and its
record.

## Implementation Plan

### Phase 1: The record tells the truth about its own subjects

- [ ] `fdu-ew1q` — `SubjectProfile` computed in the harness and written by
  `perf-record`; the materially-different predicate with tests over the real corpus of
  20 subjects.
- [ ] `fdu-i4u4` — `verdict.scope` on the model, required for new artifacts; backfill
  the existing 66 as `untested` except exp-064/065, which are `subject-specific` and
  `transfers` respectively.
- [ ] `fdu-b6lz` — ledger and evidence report render scope and profile inline with every
  headline figure.
- [ ] `fdu-1xlb` — provenance required past the cutover; variant-spelling hypothesis ids
  fatal; control-distance recorded and flagged.
- [ ] `fdu-5hfc` — backfill provenance for the 20 pre-rule subjects, stating plainly
  where no recipe exists rather than inventing one.

### Phase 2: The loop’s rules use it

- [ ] `fdu-jxk5` — the accept rule gains a scope clause: a result may be quoted outside
  its own artifact only at `transfers` or with its `subject-specific` qualifier
  attached. Screening on one subject stays legal and stays unquotable.
- [ ] `fdu-nizv` — dense mode in `gen_tree.py`; content-tier jobs default to it.
- [ ] `fdu-ucbo` — re-screen the tier-level claims currently quoted from single
  subjects, starting with the ones campaign 2 prioritizes against, and record each as
  `transfers` or `subject-specific`.
- [ ] `fdu-jxk5` (same change) — fold the rule into the performance-loop guide and the
  design principles, replacing the prose added in PR #38 with a pointer to the enforced
  version.

## Testing Strategy

The harness has its own suite (159 tests) and this work extends it:

- Profile derivation against all 20 recorded subjects, with the sparse and dense cases
  pinned by their known ratios — exp-064’s subject must come out at 22.6× and exp-065’s
  near 1.
- The materially-different predicate tested on the pair this plan exists because of:
  exp-064’s subject and exp-065’s must classify as materially different, and two runs
  over one tree must not.
- Enforcement tested from both sides: a post-cutover artifact without provenance fails,
  a pre-cutover one passes, `H86-foo` beside `H86` fails, and `H31` across twelve
  experiments still passes.
- The full ledger and evidence report regenerate byte-identically except for the added
  qualifiers, which is what proves no recorded number moved.

## Rollout Plan

Phase 1 lands as one PR against the harness only, so no engine behavior changes and
`make check` is the whole gate.
The cutover date is the merge date, which keeps every existing artifact valid and makes
the new requirement unambiguous for anything recorded after it.

Phase 2 changes what the loop *permits*, so it lands after Phase 1 has been used for at
least one real experiment — most naturally campaign 2’s Phase B, whose accept set
already requires a real tree and which will be the first structural result to need a
scope verdict.

## Open Questions

- What margin makes two subjects materially different?
  A first cut of 2× on sparseness, 1.5× on depth, and generated-versus-observed on names
  is defensible from the three recorded failures, but it is a guess until run against
  the corpus.
- Should `untested` results be renderable in the evidence report at all, or only in
  their own artifacts?
  Rendering them keeps the record complete; hiding them makes misquotation structurally
  impossible.
- Does the prose-level audit belong here later, or as its own effort?
  Phase 2 fixes the source of quotations but does not check sentences already written by
  hand.
- The floor report’s peer comparison against `ignore` is the sharpest live instance of a
  single-subject claim.
  Re-scoping it under this rule may change what the README says, and that is a
  user-facing edit worth deciding deliberately.

## References

- [The performance loop](../../guides/performance-loop.md) — the accept rule and the
  reference-tree section this plan amends
- [Performance campaign 2](plan-2026-08-23-fdu-performance-campaign-2.md) — the consumer
  of these verdicts; its Phase B accept set already anticipates the rule
- [The metadata-walk floor report](../../reports/report-2026-08-23-metadata-walk-floor.md)
  — two of the three recorded generalization failures, and the matched-control method
  that caught them
- [The experiment ledger](../../reports/report-2026-08-10-fdu-performance-experiments.md)
  and [the evidence report](../../reports/report-2026-08-20-fdu-performance-evidence.md)
  — the generated views that must carry the qualifier
- [Experiment-loop framework extraction](plan-2026-08-22-experiment-loop-framework-extraction.md)
  — if the loop is ever extracted as a reusable framework, scope is part of what makes
  it worth reusing
- exp-064, exp-065 — the worked example this plan generalizes from

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
