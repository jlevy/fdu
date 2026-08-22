---
type: is
id: is-01m0nkj4vderecrkj9ftkhy38c
title: "Draft experiment-loop SKILL.md: method core, campaign setup, round loop, record/resume/merge guidance"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncz1cxrczgbs2hdn1t5gab
created_at: 2026-08-22T20:45:55.948Z
updated_at: 2026-08-22T20:46:42.559Z
---
File: ~/.claude/skills/experiment-loop/SKILL.md (target <=300 lines).

Frontmatter: name experiment-loop; pushy description covering triggers: hypothesis-driven optimization/research campaigns, experiment ledger/research log, performance or load-time loops, strategy/algorithm/packing/proof search, recording negative results, resuming or merging campaigns, iterative improvement methodology.

Sections, in order:
1. What this is — method distilled from two independent implementations (fdu 64-artifact loop; metabrowser explorations). The skill gives a way to think + adaptable pieces, NOT a rigid workflow; the agent builds the lightest loop that answers its problem's question.
2. When to use / when not — campaign vs one-off question; the CI boundary (an exploration answers a question once; a benchmark defends an answer forever).
3. What a finished campaign leaves behind — committed research log (artifacts + registry + raw runs), backing work in the repo, generated final report; revisitable by later agents/models.
4. The invariant core — the 12 rules both projects converged on, condensed, imperative.
5. Set up a campaign — choose subject fields, metric vector + roles, comparison shape(s), tier, decision vocabulary; write the runbook README (resume-from-three-places rule); write experiment.schema.yaml + hypothesis.schema.yaml from references/contract.md starters; directory layout; baseline round first.
6. Run a round — 8-step loop with pre-registration and record-either-way.
7. Record both halves — structured: multiple benchmarks, medians WITH spread, per-metric roles; qualitative: full research report of what was tried and how it was built (code refs: commit, entry point, command; resources; surprises; what the prediction got wrong). Check all of it in.
8. Views are generated — do-not-edit-by-hand, regenerate after every round, optional drift gate.
9. Resume, parallel work, merging — ids are the merge surface; whole-set identity check; renumber the newer campaign on collision; replication is a feature (same H-id, new exp-id, record operator/model); reconciliation = union of artifacts + regenerate.
10. Adapt the weight — knobs table (statistics by tier, schema origin by field count, drift gate by record size, measurement mode by dependency policy, interleaving by automation).
11. When to read each reference file.
12. Exemplars and resources — softschema CLI commands; fdu paths (~/wrk/github/fdu: performance-loop.md, instrumentation playbook, experiment.py, experiments/) + github.com/jlevy/fdu; metabrowser explorations/ + PR jlevy/metabrowser#66; the extraction spec in fdu.; the overlap test (median + min-max range, n>=3, overlapping = no detectable effect NEVER a small win); the paired bootstrap — procedure plus a copyable dependency-free paired_bootstrap_change() adapted from benchmarks/realtree/measure.py paired_comparison + _bootstrap_median_interval (pairs at equal ordinals, deltas AND ratios, deterministic seed 0x5EED, 2000 resamples, percentile 2.5/97.5); evidence flags derived from the interval (passes_acceptance / ci_excludes_zero / direction / noninferiority) never stored opinions; accept-rule construction template (N arithmetic clauses + exactly one written judgment clause, authored BEFORE measuring); guards as independent gates; validity guards refuse rather than annotate; run both orderings on surprise; a surprising result is noise until it survives (~3 measurements to kill one plausible number).

traps.md (~180 lines): the named-failure catalog, each entry trap -> guard, grouped: Before measuring (optimizing what is not slow; two hypotheses competing for one cost; predicting a metric the rule does not score) / While measuring (drift and unpaired arms; harness in its own profile; a fresh server is not a cold scan; hidden-pane idle callbacks; both-orderings; identical-output check before timing) / Instrumentation (counter reading zero — equality guard verified by deletion; shared atomics measuring contention; /proc/self/io counting only read/write; flat profile vs caller tree; 0x0 viewport — instrument must prove it measured something) / The record (single significant boolean; ratio-of-medians vs paired change; id collisions across parallel campaigns; implicit variant order; hand-maintained registry; back-dating registrations — retroactive marker; median without range).

worked-examples.md (~150 lines): one mapping table per domain — fdu (performance), metabrowser (full-stack webapp), geometric packing search (record shape: best score vs standing best + beat_record determination, code refs per attempt, abandoned-with-budget), proof/algorithm strategy portfolio (determination outcomes, budgets); each maps subject / hypotheses / instrument / metrics+roles / shape / tier / decisions used / what the body holds; pointers to gold-standard exemplar artifacts (fdu exp-051, metabrowser exp-003) and both generated ledgers.
EOF
)
