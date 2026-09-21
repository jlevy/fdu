---
type: is
id: is-01m2zpkwa48ecgw720rkgrs9wb
title: "Correct PR #91/#92 experiment records: exp-116 headline, digest claim, regime labels, exp-124/137 errata"
kind: task
status: closed
priority: 2
version: 4
labels: []
dependencies: []
created_at: 2026-09-20T15:23:06.434Z
updated_at: 2026-09-21T08:20:31.385Z
closed_at: 2026-09-21T08:20:31.385Z
close_reason: "Shipped on #104 (https://github.com/jlevy/fdu/pull/104) at 743d2b9c / 4d78558e vs main a290aedc. CI run 35576573158 green on ubuntu/macos/windows including Performance evidence. exp-116 change_pct is the paired −2.158%; commit quoted as 984e4618; digest/regime/errata corrected; ledger+report regenerated. R3 name negatives, valid-rename load control, and H138 sharing guard restored; apply timer starts at candidates.remove. peak_rss_bytes prints as bytes (exp-117 377.5→339.4 MiB). Local rust-test passed except the known parallel_equivalence flake (not fixed)."
---
From the independent pre-merge verification of PRs #91 and #92 (2026-09-20). Evidence-only; no product behavior, README, CHANGELOG, or golden is affected. Fix once on main after the stack merges, then `make perf-ledger` and `make perf-report` in the same commit.

- V91-1: exp-116 `verdict.change_pct: -99.939` with `primary_job: default-tree`, `primary_metric: wall_ns`. The record's own paired effect for that job/metric is -2.158% [−3.88, +2.753], `passes_acceptance: false`. −99.9% is a cross-job ratio (1.6 ms second report vs 2,612 ms one-shot), but the schema defines the field as the headline paired median change, and the ledger index row and timeline.json carry it as such. Set it to the paired value (or null if the schema allows) and keep the 1,630x figure in `reason`.
- V91-6: exp-116 `commit: 2c6535c8` is a docs-only H119 commit; the probe mode landed in `984e4618`.
- V91-2: the runbook and exp-117 claim the tree engine digest is unchanged across exp-106/108–115. Frontmatter shows exp-106 `41a1e845…`, exp-108 `aaf1e17d…`, exp-109+ `3fbfed48…` (exp-109 itself discloses the move). Say "same shape"; the content digest is what is actually constant.
- V91-5: "uncontrolled" regime is labelled on H113/H116/H118/H120 registry rows but not H107/H108/H112/H115/H117, and "Confirmed" is applied to H108/H112/H117 from the same kind of cells that hold H113 "Open; needs quiet host". Label consistently; consider a host-pressure field in the experiment schema so the regime reaches the ledger.
- V92-3: exp-124 describes the completeness mechanism as HashMap length with `new_failure_modes: []`; the measured candidate had the R3 fail-open and shipped code counts visited files. exp-137's verdict reason says "report identity unchanged" while the benchmark digested no reports, and its "What was changed" predates the R1 fix. Add dated errata (precedent: 7ed0fd15). No timing claim changes.

## Notes

Context posted for Linux handoff on PR #94: https://github.com/jlevy/fdu/pull/94#issuecomment-5757234985
