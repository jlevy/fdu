---
title: Linux first-run leftover is still the walk; snapshot write not skippable
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-147
  title: Linux first-run leftover is still the walk; snapshot write not skippable
  date: "2026-09-20"
  hypotheses:
    - H146
  subject:
    tree_label: linux-v6.12
    tree_root_id: d7c0dad8d82c8bb394f459b1dd77c9e8af60d0482cca92199519668546a5147e
    tree_engine_digest: a298a9c2c8f8ee910d22b87093a739993cdf153104590f4638eeee740b239bd0
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone .git directory as git left it."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293224
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
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
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args: []
    candidate_binary:
      name: candidate
      sha256: 2bc59b685deda30378ab83d0a9782ea8711451611bd03918f37183c602b978b7
      size_bytes: 3030680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-147-h146-linux-first-run-leftover-quiet.json
  results:
    - job: default-tree-first
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 448576649.0
          candidate_median: 456591794.5
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.029
          change_pct: 1.484
          ci95_low_pct: -2.102
          ci95_high_pct: 3.567
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 446874119.0
          candidate_median: 454787711.5
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.029
          change_pct: 1.486
          ci95_low_pct: -2.088
          ci95_high_pct: 3.586
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 553849500.0
          candidate_median: 561999000.0
          control_p95_over_median: 1.038
          candidate_p95_over_median: 1.018
          change_pct: 0.84
          ci95_low_pct: -1.624
          ci95_high_pct: 2.624
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 445889000.0
          candidate_median: 450516500.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.011
          change_pct: 0.437
          ci95_low_pct: -2.648
          ci95_high_pct: 4.275
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 110995500.0
          candidate_median: 113846500.0
          control_p95_over_median: 1.186
          candidate_p95_over_median: 1.108
          change_pct: 1.775
          ci95_low_pct: -10.833
          ci95_high_pct: 13.178
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 45895680.0
          candidate_median: 45836288.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.006
          change_pct: -0.13
          ci95_low_pct: -0.433
          ci95_high_pct: 0.308
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
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: default-tree-first
    primary_metric: wall_ns
    change_pct: 1.484
    reason: "same leftover identity as Darwin H136: walk 93% of first-run; isolated save ~24ms is >=3% and not skippable; do not retry H100"
    commit: f0126084
---
## What was predicted

H146 is the Linux leftover after H140 on first-run `default-tree-first`, the H136
analog.

Named before measuring:

- Job: same-binary 12-pair `default-tree-first` on reconstructible `linux-v6.12`.
- Attribution: three counters-on first-run hits plus three isolated `snapshot-save` hits
  (scan outside the save timer).
- Determination: snapshot encode/write/render is or is not a skippable ≥3% cut.
  Darwin leftover was still the walk; write ~45 ms and not skippable.
- Do not retry H100. Do not load a snapshot on `fdu PATH`.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Quiet held: initial load/core 0.082, final 0.119, 0 of 30 samples invalidated.
`FDU_COUNTERS` unset on the pair.
3 warmups, 12 timed pairs, interleaved.
Same HEAD probe both arms (sha256 `2bc59b68…`). No RAM disk.

Subject: the same frozen `linux-v6.12` clone as exp-144 through exp-146. Fingerprint
unchanged.

Every timed sample had `snapshot_written` true, `source=scan`, snapshot 6,493,271 bytes.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 448.6 ms | 446.9 ms | 43.8 MiB |
| candidate | 456.6 ms | 454.8 ms | 43.7 MiB |

Self-comparison wall +1.48% [−2.10%, +3.57%]. Attachment only.

Counters-on first-run hits (snapshot deleted before each):

| Hit | Component | Walk | Walk / component | Finish | Written |
| ---: | ---: | ---: | ---: | ---: | --- |
| 1 | 458.8 ms | 426.8 ms | 93.0% | 1.9 ms | true |
| 2 | 470.7 ms | 436.5 ms | 92.7% | 1.6 ms | true |
| 3 | 463.8 ms | 433.8 ms | 93.5% | 1.4 ms | true |

Isolated `snapshot-save` hits (scan is setup; timer is encode + `publish` only): 24.3
ms, 23.0 ms, 23.6 ms.
Median 23.6 ms of a 6.2 MiB image.
That is 5.1–5.3% of the matching first-run component.
Render plus join residue after walk and that save is a few milliseconds.

## What the determination said

**Same** leftover identity as Darwin H136, cheaper write.
First-run leftover is still the walk (93% of component).
Isolated save ~24 ms is ≥3% of first-run and not skippable: there is no existing file
for H100’s identical-rewrite skip; H78/H92 format and `fdu-n75m` fsync stay.
No new userspace cut.

No engine patch. Do not retry H100. Do not load a snapshot on `fdu PATH`.
