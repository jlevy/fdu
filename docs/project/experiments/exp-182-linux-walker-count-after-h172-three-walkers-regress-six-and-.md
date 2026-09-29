---
title: "Linux walker count after H172: three walkers regress, six and eight do not clear on both trees"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-182
  title: "Linux walker count after H172: three walkers regress, six and eight do not clear on both trees"
  date: "2026-09-29"
  hypotheses:
    - H165
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: 55a09ea65f48d38d88d7e2eacde95fa81d0a9970baabfeeba6f9e9418f478704
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293209
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
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
    trials: 12
    warmups: 3
    interleaved: true
    control: "956659de probe, --threads 4 (shipped PORTABLE)"
    candidate: "956659de probe, --threads 3 (cores - 1)"
    control_binary:
      name: t4
      sha256: 882a639f58a20c22081fca23fc02359ce21809c90d91396a8b350910de555b51
      size_bytes: 3846168
      args:
        - "--threads"
        - "4"
    candidate_binary:
      name: t3
      sha256: 882a639f58a20c22081fca23fc02359ce21809c90d91396a8b350910de555b51
      size_bytes: 3846168
      args:
        - "--threads"
        - "3"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-182/run.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 84690026.0
          candidate_median: 94509218.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.139
          change_pct: 10.253
          ci95_low_pct: 5.944
          ci95_high_pct: 14.082
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        component_ns:
          control_median: 81353172.5
          candidate_median: 91465378.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.148
          change_pct: 11.114
          ci95_low_pct: 6.321
          ci95_high_pct: 14.743
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 297178000.0
          candidate_median: 310356500.0
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.12
          change_pct: 4.147
          ci95_low_pct: 0.566
          ci95_high_pct: 5.596
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 93976500.0
          candidate_median: 105713500.0
          control_p95_over_median: 1.322
          candidate_p95_over_median: 1.158
          change_pct: 11.326
          ci95_low_pct: -2.858
          ci95_high_pct: 26.593
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 202665500.0
          candidate_median: 219401000.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.067
          change_pct: 3.992
          ci95_low_pct: -3.261
          ci95_high_pct: 11.962
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "voluntary_context_switches exceeds its +50% regression limit"
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
          voluntary_context_switches: rejected
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
    change_pct: 10.253
    reason: "quiet 12-pair screen on the H172 head: --threads 3 regresses +10.25% [+5.94%, +14.08%] on linux-v6.12 and +21.95% on node-modules-dense; 6 and 8 walkers do not clear on both real subjects; PORTABLE unchanged"
    commit: null
    kept: control
---
## What was predicted

H165, amended by the overnight plan
([amendment 4](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md)):
once H171, H175 and H172 have taken the consumer off the critical path, does a walker
count other than the shipped four (`PORTABLE` on a 4-vCPU host) do better?
The plan’s first draft proposed `cores − 1` (three walkers), to give the consumer its
own core. The Fable review predicted that this loses once the consumer is light: with
three walkers the walk takes at least a third longer.
It was registered as a screen: arms `--threads 3`, `4`, `6` and `8` on the accepted
head. A `PORTABLE` change needs the accept rule on both real subjects, and a formulation
that does not also change hosts with more cores.

## What was measured

Quiet, 12 pairs, `default-tree`, the H172 head (`956659de` probe); four is the control.

| Walkers | `linux-v6.12` | `node-modules-dense` |
| ---: | --- | --- |
| 4 | 84.7 ms | 72.5 ms |
| 3 | **+10.25% [+5.94%, +14.08%]** | **+21.95% [+14.82%, +26.96%]** |
| 6 | −3.39% [−11.61%, +7.54%] | −2.03% [−6.47%, +0.46%] |
| 8 | +9.65% [−4.67%, +17.46%] | −3.58% [−7.47%, −0.86%] |

The node-modules-dense run is kept beside this record’s evidence.

## Decision

Rejected; the shipped four walkers stay.
Three walkers is a clear regression on both subjects, as the review predicted.
Six and eight walkers do not clear the rule on both real subjects.
Eight moves in opposite directions on the two trees, and a 12-pair screen cannot
separate that from noise.
`PORTABLE` is unchanged.
On this host, after H172, the walk sets the time, and adding walkers to four vCPUs does
not shorten it.
