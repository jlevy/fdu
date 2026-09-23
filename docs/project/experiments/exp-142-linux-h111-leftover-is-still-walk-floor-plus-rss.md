---
title: Linux H111 leftover is still walk floor plus retained-index RSS
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-142
  title: Linux H111 leftover is still walk floor plus retained-index RSS
  date: "2026-09-20"
  hypotheses:
    - H143
  subject:
    tree_label: linux-450k
    tree_root_id: b3459e9451517d4c81d92f78310218b6f158f3fc53e0b5748186122d2f255006
    tree_engine_digest: b77ebb296d346faf853a2d9db41fea20d347e52dfafee01a1d23a357330a7907
    tree_provenance: "Generated balanced recipe, 450,001 entries, manifest f93bffc36eab67a0d2d72909f3552c7bc235e073eff11b5e793e1a5a8407c938, semantic digest 0c5230889cbe6ee25ceb6e64560cb012bccd03126565fd8f8d313e7013715e3d. Reconstructible: python -m benchmarks.generate create --recipe balanced --entries 450000. Same semantic digest as exp-103 and exp-141."
    tree_reconstructible: true
    tree_entries: 450001
    tree_directories: 56251
    tree_files: 393750
    tree_symlinks: 0
    tree_apparent_bytes: 358665192
    tree_allocated_bytes: 1344430080
    tree_max_depth: 7
    tree_mutated_during_run: false
    host_cpu: Intel(R) Xeon(R) Processor
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16791945216
    host_system: Linux 6.12.94+
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: HEAD release probe both arms
    candidate: same probe leftover profile
    control_binary:
      name: control
      sha256: 84060ec54c559a261960c5e7659020da0f045ef9733e54a0daf72b592e9d7e82
      size_bytes: 3030680
      args: []
    candidate_binary:
      name: candidate
      sha256: 84060ec54c559a261960c5e7659020da0f045ef9733e54a0daf72b592e9d7e82
      size_bytes: 3030680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-142-h143-450k-index-leftover.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 857578764.0
          candidate_median: 856732192.5
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.021
          change_pct: 0.131
          ci95_low_pct: -0.957
          ci95_high_pct: 0.756
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 319381209.0
          candidate_median: 318734663.5
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.01
          change_pct: -0.277
          ci95_low_pct: -0.89
          ci95_high_pct: 0.513
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1574917500.0
          candidate_median: 1576612500.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.012
          change_pct: 0.171
          ci95_low_pct: -0.741
          ci95_high_pct: 0.813
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 881806500.0
          candidate_median: 866492000.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.031
          change_pct: -3.515
          ci95_low_pct: -5.672
          ci95_high_pct: 0.372
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 696087500.0
          candidate_median: 721049000.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.049
          change_pct: 2.45
          ci95_low_pct: 0.126
          ci95_high_pct: 6.322
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 157194240.0
          candidate_median: 158410752.0
          control_p95_over_median: 1.012
          candidate_p95_over_median: 1.012
          change_pct: 1.247
          ci95_low_pct: -0.185
          ci95_high_pct: 2.189
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
  reference_tools:
    - name: dust
      wall_ns_median: 564313578.5
      argv:
        - "{binary}"
        - "-s"
        - "-k"
        - "{root}"
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: cold-scan-index
    primary_metric: wall_ns
    change_pct: 0.131
    reason: "walk 94.7-94.8% of 450k cold-scan-index component; leftover is getdents64+statx plus detached finish/retained-index RSS; no new cut; do not restart H86"
    commit: 937f9445
---
## What was predicted

H143 is the leftover after H111 failed on this virtualized host (exp-141).

Named before measuring:

- Job: same-binary 12-pair `cold-scan-index` on reconstructible `linux-450k` (450,001
  entries). That is the H111 index-gate job, not a real-tree accept.
- Walk share of component from counters-on `scan-index`.
- Determination: walk still at least 90% of component and leftover is
  `getdents64`+`statx` plus retained-index RSS, or a userspace stage at least 3% that is
  not H86 or H71.
- Do not compile a walk trim.
  Do not restart H86. Do not retry H71.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Quiet start passed (load/core 0.043). The pair held: final load/core 0.194, 0 of 30
samples invalidated.
`FDU_COUNTERS` unset on the pair.
3 warmups, 12 timed pairs, interleaved.
Same HEAD release probe both arms (sha256 `84060ec5…` / 3,030,680 bytes).
No RAM disk.

Subject: generated `linux-450k` (450,001 entries / 393,750 files / 56,251 directories).
Manifest `f93bffc3…`. Semantic digest `0c523088…` matches exp-103 and exp-141.
Reconstructible:
`python -m benchmarks.generate create --recipe balanced --entries 450000`. Engine digest
`b77ebb29…`. The tree did not mutate.

Host: 4-core KVM Intel Xeon, 16 GiB, Linux 6.12.94+, ext4, virtualized.
`os_cache: warm-steady`. Same class as exp-103.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 857.6 ms | 319.4 ms | 149.9 MiB |
| candidate | 856.7 ms | 318.7 ms | 151.1 MiB |

Self-comparison wall +0.13% [−0.96%, +0.76%]. Attachment only.

Counters-on attribution (same binary, oracle on, matching the harness job):

| Hit | Component | Walk | Finish | Walk / component | Opens | Stats |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 334.8 ms | 317.0 ms | 14.6 ms | 94.7% | 56,251 | 450,001 |
| 2 | 323.2 ms | 306.1 ms | 13.9 ms | 94.7% | 56,251 | 450,001 |
| 3 | 336.9 ms | 319.3 ms | 14.4 ms | 94.8% | 56,251 | 450,001 |

`dir_enumeration_calls` stayed 0, the documented portable-path value.
One directory open per directory including the root.
One metadata stat per fingerprint entry.
File opens 0. Finish is 4.3% of component: the detached consume that builds the retained
index.
That is the time-domain of the H86 composite already named by H111/H143, not a new
cut. Do not restart H86.

Peak RSS on the pair (~150 MiB) is the same class as exp-141’s 158.5 MiB / 5.20×
`arena_spike`. No new RSS owner.

`strace` is not installed on this image, so getdents64 multiplicity is not a counted
ground truth here. The playbook’s Linux floor remains 2.00 `getdents64`/dir plus
per-entry `statx`.

## What the determination said

**Same** leftover identity as minted after exp-141. Walk is still the index job (≥90% of
component). The leftover is the Linux syscall floor (`getdents64`+`statx`+one open per
directory) plus retained-index RSS / detached finish.
No new userspace cut ≥3% that is not H86.

No engine patch. Do not compile a walk trim.
Do not restart H86. Do not retry H71. Do not raise the README 200K files/s or 4M cached
lines/s. A bare-metal remeasure of H111 remains the other way H143 could close; this
host cannot do that.
