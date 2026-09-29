---
title: "Linux H159 listing recycle misses on linux-v6.12, -10.6% on the generated tree"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-188
  title: "Linux H159 listing recycle misses on linux-v6.12, -10.6% on the generated tree"
  date: "2026-09-28"
  hypotheses:
    - H159
  subject:
    tree_label: linux-v6.12
    tree_root_id: 14549a49743a72c3c1aadb09f34f8a097211652c22d533110f3c18b91c518d71
    tree_engine_digest: c7a4d447d9bf3ab36d55c385a4bbe3ed367963aca0fc6e8e5671ec3e5ad8b124
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759236097
    tree_allocated_bytes: 1965420544
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16877547520
    host_system: Linux 6.18.44-fc-v37
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: 0d73ed54 probe (main)
    candidate: "aa58a6b1 probe (150): drained listings return to their walker"
    control_binary:
      name: control
      sha256: 2d8684e546f5eb7754e3503b1be74749310880dc96369e6e984c1dbbd157b3d6
      size_bytes: 3707464
      args: []
    candidate_binary:
      name: candidate
      sha256: fadb2e564753663a03663fbd3d8f33d474020d837bbde8f589a9b8e17b5de02b
      size_bytes: 3735352
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-188/run.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 658466531.5
          candidate_median: 669538372.0
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.12
          change_pct: 5.266
          ci95_low_pct: -4.276
          ci95_high_pct: 8.814
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 546640803.0
          candidate_median: 555954954.5
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.096
          change_pct: 4.137
          ci95_low_pct: -4.112
          ci95_high_pct: 6.476
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 846313000.0
          candidate_median: 847383000.0
          control_p95_over_median: 1.073
          candidate_p95_over_median: 1.103
          change_pct: 3.05
          ci95_low_pct: -1.975
          ci95_high_pct: 6.613
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 658263000.0
          candidate_median: 672550500.0
          control_p95_over_median: 1.079
          candidate_p95_over_median: 1.128
          change_pct: 5.193
          ci95_low_pct: -1.773
          ci95_high_pct: 8.333
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 184812500.0
          candidate_median: 184210500.0
          control_p95_over_median: 1.155
          candidate_p95_over_median: 1.049
          change_pct: 1.05
          ci95_low_pct: -14.034
          ci95_high_pct: 9.668
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 37097472.0
          candidate_median: 37648384.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.014
          change_pct: 1.433
          ci95_low_pct: 0.175
          ci95_high_pct: 2.367
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 576404855.0
          candidate_median: 569981821.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.102
          change_pct: -2.191
          ci95_low_pct: -4.505
          ci95_high_pct: 1.078
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 571992169.5
          candidate_median: 564972120.5
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.067
          change_pct: -2.189
          ci95_low_pct: -4.638
          ci95_high_pct: 1.279
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 767698000.0
          candidate_median: 778578500.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.039
          change_pct: 0.634
          ci95_low_pct: -2.165
          ci95_high_pct: 4.751
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 586550500.0
          candidate_median: 577479000.0
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.096
          change_pct: -0.812
          ci95_low_pct: -3.858
          ci95_high_pct: 2.491
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 179087500.0
          candidate_median: 194409000.0
          control_p95_over_median: 1.095
          candidate_p95_over_median: 1.147
          change_pct: 10.947
          ci95_low_pct: -10.234
          ci95_high_pct: 17.734
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 38619136.0
          candidate_median: 38957056.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.003
          change_pct: 0.329
          ci95_low_pct: -0.043
          ci95_high_pct: 1.008
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: noninferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -2.191
    reason: "quiet linux-v6.12 default-tree -2.19% [-4.50%, +1.08%] where gitignore classification was about 85% of the job; screening balanced-1m -10.63% [-13.00%, -8.23%], product contract main +10%"
    commit: aa58a6b1
    kept: control
---
## What was predicted

H159, pre-registered in `fdu-578e` and exp-167: the index tier’s remaining Linux gap to
pdu and diskus is glibc arena contention from the index consumer freeing listings that
walker threads allocated.
[#150](https://github.com/jlevy/fdu/pull/150) returns each drained listing to its
walker, which reuses up to one chunk’s worth and frees the rest on its own thread.
The deciding job is `default-tree`, down at least 3% with the interval below zero on
reconstructible `linux-v6.12`; `cold-scan-index` is expected to move;
`linux-balanced-1m` screens; the product `fdu-default-tree` contract is paired in the
tool harness; peak RSS must be non-inferior (upper bound at most +5%).

## What was measured

Control: release probe and CLI at `main` `0d73ed54` (the CLI at `45c7c577`, whose engine
is identical). Candidate: `aa58a6b1`, #150’s head.
Quiet cells, 12 interleaved pairs, no invalid sample.

`linux-v6.12` (deciding):

- `default-tree`: −2.19% [−4.50%, +1.08%] (576.4 → 570.0 ms).
  Reject.
- `cold-scan-index`: +5.27% [−4.28%, +8.81%].
- Peak RSS: +0.33% and +1.43%, non-inferior.

`linux-balanced-1m` (screening,
[evidence/exp-188/balanced-run.json](evidence/exp-188/balanced-run.json)):

- `default-tree`: −10.63% [−13.00%, −8.23%] (1,580.8 → 1,424.1 ms).
- `cold-scan-index`: −7.23% [−8.27%, −5.05%].
- Peak RSS: −4.95% and −5.71%; user CPU −12.92% on `default-tree`.

Product contract, CLI against CLI in the tool harness: `main` took +10% [+8%, +15%] more
wall than the candidate on `linux-balanced-1m`, and +2% [−6%, +6%] on `linux-v6.12`
([balanced](evidence/exp-188/cli-run-balanced.json),
[linux-v6.12](evidence/exp-188/cli-run-linux-v6.12.json)).

A screen under `LD_PRELOAD` jemalloc on `linux-balanced-1m` put the control’s indexed
tree at 1.31 s from 1.67 s and the candidate’s at 1.29 s from 1.48 s: the change
recovers about half of the allocator’s share on that tree.

## Decision

Rejected on its deciding subject.
On `linux-v6.12` the default tree was about 85% `.gitignore` classification on the
consumer thread, a cost that did not exist on the generated tree, so the allocator cost
this change removes had little room to show.
exp-189 repeats the cell on top of H162 and H163, which remove that classification cost.
