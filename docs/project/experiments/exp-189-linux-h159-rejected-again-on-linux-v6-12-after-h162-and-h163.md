---
title: Linux H159 rejected again on linux-v6.12 after H162 and H163
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-189
  title: Linux H159 rejected again on linux-v6.12 after H162 and H163
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
    control: "eb00edf8 scratch probe: main with H162 and H163"
    candidate: "a15b20f4 probe: H159 with H162 and H163"
    control_binary:
      name: control
      sha256: 9e46354956d0133cce71db81ce2caac4a92b10a88aaae704f677ec71883668e1
      size_bytes: 3716336
      args: []
    candidate_binary:
      name: candidate
      sha256: 058b9e2ab27ab9c4d85a42bac8d2d20766b827696407f3226eaccec6b2918a96
      size_bytes: 3744176
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-189/run.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 307098914.0
          candidate_median: 322357519.5
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.087
          change_pct: 3.217
          ci95_low_pct: 1.214
          ci95_high_pct: 12.872
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 194410622.0
          candidate_median: 207968347.5
          control_p95_over_median: 1.144
          candidate_p95_over_median: 1.131
          change_pct: 4.228
          ci95_low_pct: 0.234
          ci95_high_pct: 16.664
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 502991500.0
          candidate_median: 515405500.0
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.084
          change_pct: 5.67
          ci95_low_pct: 2.578
          ci95_high_pct: 6.565
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 305423500.0
          candidate_median: 342046000.0
          control_p95_over_median: 1.107
          candidate_p95_over_median: 1.046
          change_pct: 9.263
          ci95_low_pct: 2.422
          ci95_high_pct: 14.284
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 181681000.0
          candidate_median: 185045500.0
          control_p95_over_median: 1.158
          candidate_p95_over_median: 1.153
          change_pct: -1.711
          ci95_low_pct: -10.012
          ci95_high_pct: 7.587
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 35303424.0
          candidate_median: 35655680.0
          control_p95_over_median: 1.027
          candidate_p95_over_median: 1.012
          change_pct: 1.113
          ci95_low_pct: -1.148
          ci95_high_pct: 2.714
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
          control_median: 204540636.0
          candidate_median: 207074727.0
          control_p95_over_median: 1.082
          candidate_p95_over_median: 1.094
          change_pct: 2.259
          ci95_low_pct: -5.326
          ci95_high_pct: 12.957
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 199942628.5
          candidate_median: 202197425.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.094
          change_pct: 1.604
          ci95_low_pct: -4.972
          ci95_high_pct: 13.633
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 399170500.0
          candidate_median: 416958500.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.025
          change_pct: 3.265
          ci95_low_pct: 1.016
          ci95_high_pct: 8.547
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 224942500.0
          candidate_median: 222620500.0
          control_p95_over_median: 1.124
          candidate_p95_over_median: 1.044
          change_pct: 2.421
          ci95_low_pct: -10.354
          ci95_high_pct: 14.495
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 182022500.0
          candidate_median: 193284500.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.121
          change_pct: 9.611
          ci95_low_pct: -2.403
          ci95_high_pct: 19.196
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 36892672.0
          candidate_median: 36632576.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.027
          change_pct: -0.209
          ci95_low_pct: -1.903
          ci95_high_pct: 1.4
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
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
    change_pct: 2.259
    reason: "quiet linux-v6.12 default-tree +2.26% [-5.33%, +12.96%], cold-scan-index +3.22% [+1.21%, +12.87%] regression interval; the recycle helps only the directory-dense generated tree"
    commit: aa58a6b1
    kept: control
---
## What was predicted

exp-188 rejected H159 on `linux-v6.12`, where `.gitignore` classification dominated the
default tree. H162 and H163 then cut that job by about 64%, which should expose the
allocator cost H159 targets if it matters on this tree.
Pre-registered in `fdu-578e` before running: the same hypothesis, jobs, subject and rule
as exp-188 (`default-tree` down at least 3% with the interval below zero, peak RSS upper
bound at most +5%), on the H162 and H163 base.

## What was measured

Control: `main` `0d73ed54` with H162 and H163 cherry-picked (a scratch build at
`eb00edf8`, whose only other difference adapts one call site to `main`’s owned path).
Candidate: `a15b20f4`, which is H159 with H162 and H163. Quiet, 12 interleaved pairs on
`linux-v6.12`, no invalid sample.

- `default-tree`: +2.26% [−5.33%, +12.96%] (204.5 → 207.1 ms).
  Reject.
- `cold-scan-index`: +3.22% [+1.21%, +12.87%], a regression interval; user CPU +9.26%
  [+2.42%, +14.28%].
- Peak RSS: −0.21% and +1.11%, non-inferior.

## Decision

Rejected again on the deciding subject.
The recycle helps on the directory-dense generated tree (eight entries per directory,
exp-188: −10.63%) and does not on the source tree (16 entries per directory), where
`cold-scan-index` now reads slightly worse.
Whether #150 merges on the strength of the generated tree is a maintainer decision;
under the accept rule it does not.
