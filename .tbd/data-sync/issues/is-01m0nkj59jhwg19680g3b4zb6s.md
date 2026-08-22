---
type: is
id: is-01m0nkj59jhwg19680g3b4zb6s
title: "Write experiment-loop reference files: contract+schemas, statistics+verdicts, traps, worked examples"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncz1cxrczgbs2hdn1t5gab
created_at: 2026-08-22T20:45:56.401Z
updated_at: 2026-08-22T20:47:11.332Z
---
Four files under ~/.claude/skills/experiment-loop/references/.

contract.md (~250 lines): the artifact spine (id/title/date/hypotheses/tier/subject/method/results/complexity/verdict) with field meanings; the softschema self-description block; the FOUR result shapes with copyable YAML (paired, conditions, record, determination); decisions table — accepted/rejected/unresolved/blocked/abandoned/superseded/baseline/in-progress, with abandoned requiring budget_spent + best_reached + reopen_when; metric roles outcome/cost/guard/mechanism with four-domain examples; STARTER SCHEMAS: complete hand-written experiment.schema.yaml (JSON Schema draft 2020-12, defs for the four shapes, additionalProperties false, guidance to delete unused shapes and replace subject fields) and hypothesis.schema.yaml (id, claim, criterion with shape/metric/direction, instrument, regime, registered incl. retroactive, notes); artifact skeleton with body headings (What was measured / What was tried / Result / What the prediction got wrong / Limits); method.operator for cross-agent replication.

statistics.md (~200 lines): evidence tiers (exploratory/confirmatory, mapping fdu campaign_stage and metabrowser's whole loop); the overlap test (median + min-max range, n>=3, overlapping means no detectable effect, NEVER a small win); the paired bootstrap — procedure plus a copyable dependency-free function adapted from benchmarks/realtree/measure.py paired_comparison and _bootstrap_median_interval: pairs at equal ordinals, deltas and ratios, deterministic seed, 2000 resamples, percentile 2.5/97.5; evidence flags derived from the interval, never stored opinions; accept-rule construction template (arithmetic clauses + exactly one written judgment clause, authored BEFORE measuring); guards as independent gates; validity guards refuse rather than annotate; run both orderings on surprise; a surprising result is noise until it survives.

traps.md (~180 lines): the named-failure catalog, each entry trap then guard, grouped: Before measuring / While measuring / Instrumentation / The record. Sources: fdu playbook and loop guide, metabrowser README and PR 66 write-ups, plus the record-level defects (single significant boolean, id collisions, implicit variant order, hand-maintained registry, median without range, ratio-of-medians vs paired change, back-dating registrations).

worked-examples.md (~150 lines): one mapping per domain — fdu performance, metabrowser webapp, geometric packing search (record shape: best score vs standing best + beat_record determination, code refs per attempt, abandoned-with-budget), proof/algorithm strategy portfolio (determination outcomes, budgets); each maps subject / hypotheses / instrument / metrics+roles / shape / tier / decisions / what the body holds; pointers to exemplar artifacts (fdu exp-051, metabrowser exp-003) and both generated ledgers.
