---
title: "Linux H180 summary walker trims cut the default summary 9% on node-modules-dense"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-183
  title: "Linux H180 summary walker trims cut the default summary 9% on node-modules-dense"
  date: "2026-09-29"
  hypotheses:
    - H180
  subject:
    tree_label: node-modules-dense
    tree_root_id: db6d49b68990d6b3a741e6289f360187d4317abfdda8be3f70915f799eb7acd4
    tree_engine_digest: 11c4122bec53693180e2e240899b75cdccc707ca9759b9acfadb80a0b19bafac
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.2, npm 10.9.7); rebuilt 2026-09-29 on this host: 79,957 entries (exp-190 recorded 79,953), 9,439 directories, no .gitignore."
    tree_reconstructible: true
    tree_entries: 79957
    tree_directories: 9439
    tree_files: 70411
    tree_symlinks: 107
    tree_apparent_bytes: 538992241
    tree_allocated_bytes: 757870592
    tree_max_depth: 12
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v49
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "956659de probe: H172 head"
    candidate: "70c2725c probe: H180 summary walker trims"
    control_binary:
      name: control
      sha256: 882a639f58a20c22081fca23fc02359ce21809c90d91396a8b350910de555b51
      size_bytes: 3846168
      args: []
    candidate_binary:
      name: h180
      sha256: 9430e7379bb91acefc51c26a4ee3b1491a976beeb3a4dd9a0320e6223b3cc670
      size_bytes: 3851832
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-183/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 73486768.5
          candidate_median: 67528712.5
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.091
          change_pct: -8.678
          ci95_low_pct: -11.488
          ci95_high_pct: -4.661
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 70708585.5
          candidate_median: 64693938.5
          control_p95_over_median: 1.109
          candidate_p95_over_median: 1.099
          change_pct: -8.693
          ci95_low_pct: -12.099
          ci95_high_pct: -4.385
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 262773000.0
          candidate_median: 243126000.0
          control_p95_over_median: 1.127
          candidate_p95_over_median: 1.115
          change_pct: -7.044
          ci95_low_pct: -10.836
          ci95_high_pct: -4.497
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 77053000.0
          candidate_median: 61318500.0
          control_p95_over_median: 1.139
          candidate_p95_over_median: 1.278
          change_pct: -15.88
          ci95_low_pct: -22.452
          ci95_high_pct: -11.642
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 192960500.0
          candidate_median: 184940000.0
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.094
          change_pct: -3.71
          ci95_low_pct: -8.817
          ci95_high_pct: 2.076
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "minor_faults straddles its +10% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: inconclusive
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 75646599.5
          candidate_median: 74557658.5
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.187
          change_pct: -2.795
          ci95_low_pct: -4.61
          ci95_high_pct: 4.524
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 72722171.5
          candidate_median: 71155634.0
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.191
          change_pct: -3.209
          ci95_low_pct: -4.787
          ci95_high_pct: 4.57
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 250140000.0
          candidate_median: 251589500.0
          control_p95_over_median: 1.097
          candidate_p95_over_median: 1.123
          change_pct: 0.3
          ci95_low_pct: -2.662
          ci95_high_pct: 4.807
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 70082500.0
          candidate_median: 58770000.0
          control_p95_over_median: 1.375
          candidate_p95_over_median: 1.404
          change_pct: -11.928
          ci95_low_pct: -31.598
          ci95_high_pct: -4.18
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 179564500.0
          candidate_median: 191365000.0
          control_p95_over_median: 1.191
          candidate_p95_over_median: 1.155
          change_pct: 13.764
          ci95_low_pct: -0.97
          ci95_high_pct: 15.523
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 125
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -8.678
    reason: "quiet 20-pair node-modules-dense aggregate-summary -8.68% [-11.49%, -4.66%]; --no-controls -1.12% n.s.; default-tree placebos include zero; walker instructions -33% (callgrind); answers identical"
    commit: 70c2725c
    kept: candidate
---
## What was predicted

H180, from the side-by-side profile in
[the overnight research brief](../research/research-2026-09-29-linux-peers-matchers-and-hot-path.md):
the transient summary route’s walkers cost about twice the default route’s per entry.
Every entry parsed `Path::file_name()` to decide whether it was a `.gitignore`
(`path_control_spelling`, about 270 instructions per entry even with no ignore file),
and every entry’s joined path was cloned into its `Op::Upsert` even when the original
was then dropped. The change decides the spelling from the listed name bytes the walker
already holds (`control::control_spelling`, the check the detached route uses).
It also joins each path once at its final capacity and moves it into the operation
unless the entry descends.
The path-move piece overlaps H51, refuted on macOS. The Linux mechanism differs: the
summary route is CPU-bound, and glibc frees cross threads.
It was pre-registered in the registry (`e897031b`):
- **Deciding:** `aggregate-summary` and `aggregate-summary --no-controls` on
  `node-modules-dense` and `linux-v6.12`, 20 pairs.
- **Placebo:** `default-tree`, whose detached route this does not touch.
- **Prediction:** walker instructions −15% to −25% on the summary route.

## What was measured

Callgrind of the release command line, `--view summary`, `node-modules-dense`: walker
instructions per entry fell from 3,071–3,081 to 2,065–2,068 (−32.8%) with `.gitignore`
on and from 2,811 to 2,031–2,032 (−27.7%) with it off.
That is more than predicted.

Quiet cell, 20 pairs, no invalid samples.
Control: the accepted H172 head (`956659de` probe; `0228ea42` has the same crates).
Candidate: `70c2725c`.

| Job | Control | H180 | Change |
| --- | ---: | ---: | --- |
| `aggregate-summary` | 73.5 ms | 67.5 ms | **−8.68% [−11.49%, −4.66%]** |
| `aggregate-summary --no-controls` | 71.6 ms | 69.1 ms | −1.12% [−8.59%, +1.63%] |
| `default-tree` (placebo) | 75.6 ms | 74.6 ms | −2.79% [−4.61%, +4.52%] |
| `default-tree --no-controls` (placebo) | 73.2 ms | 72.8 ms | +0.21% [−4.32%, +2.39%] |

On this tree, with no `.gitignore` to read, the wall effect with controls on is mostly
the spelling check, which runs only when ignore handling is on.
The path-move piece alone does not clear the rule here.

Answers: the product command line’s output over the three subjects was byte-identical to
the base (54 comparisons).
An Opus adversarial review passed the change.
It checked that the spelling decision is unchanged for every name a listing can produce,
and that the new join is byte-identical to `Path::join` on 240 directory and name pairs.

## Decision

Accepted with exp-184 on the default summary (controls on), the job users run.
The `--no-controls` arm clears on `linux-v6.12` (exp-184) but not here.
About 110 lines.
