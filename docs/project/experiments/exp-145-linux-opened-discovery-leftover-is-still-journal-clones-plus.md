---
title: Linux opened-discovery leftover is still journal clones plus live roll-ups
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-145
  title: Linux opened-discovery leftover is still journal clones plus live roll-ups
  date: "2026-09-20"
  hypotheses:
    - H145
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
    run_artifact: /tmp/fdu-realtree/results/run-exp-145-h145-linux-opened-discovery-leftover-uncontrolled.json
  results:
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1496980649.0
          candidate_median: 1508023276.5
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.025
          change_pct: -0.231
          ci95_low_pct: -0.579
          ci95_high_pct: 1.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 1174009001.5
          candidate_median: 1184382420.5
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.017
          change_pct: 0.374
          ci95_low_pct: -0.521
          ci95_high_pct: 1.1
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1553772000.0
          candidate_median: 1562332000.0
          control_p95_over_median: 1.03
          candidate_p95_over_median: 1.028
          change_pct: -0.187
          ci95_low_pct: -0.482
          ci95_high_pct: 0.894
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 1392624500.0
          candidate_median: 1401034000.0
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.017
          change_pct: 0.35
          ci95_low_pct: -1.251
          ci95_high_pct: 1.651
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 172154000.0
          candidate_median: 170345000.0
          control_p95_over_median: 1.184
          candidate_p95_over_median: 1.216
          change_pct: -2.762
          ci95_low_pct: -9.374
          ci95_high_pct: 12.521
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 112134144.0
          candidate_median: 112205824.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.001
          change_pct: 0.035
          ci95_low_pct: -0.062
          ci95_high_pct: 0.185
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
        reasons:
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 86316399.5
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
    primary_job: opened-discovery
    primary_metric: wall_ns
    change_pct: -0.231
    reason: "same leftover identity as Darwin H127: 5772 journal clones, 438k live roll-up merges, 2.75x first-pass; no smallest cut; do not port macos_bulk"
    commit: eec14927
---
## What was predicted

H145 is the Linux leftover after the current engine on opened-root discovery, the H127
analog.

Named before measuring:

- Job: same-binary 12-pair `opened-discovery` on reconstructible `linux-v6.12`.
- Cross-job counters against `scan-index` on the same tree.
- Determination: leftover is still journal clones / `read_dir`+`fstatat` / live roll-up
  merges (expected), or a userspace stage at least 3% Darwin did not name.
- Do not port `macos_bulk`. Do not apply H115 rebuild to progressive commits unless the
  leftover names that.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Quiet start passed (load/core 0.151). The cell did not hold: 3 of 30 samples invalidated
at the end (load/core 0.251). That incomplete quiet run is not a verdict and was not
topped up. Saved as `run-exp-145-h145-linux-opened-discovery-leftover.json`.

The claim-grade pair is **uncontrolled**. Same HEAD probe both arms (sha256
`2bc59b68…`). 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.
Initial load/core 0.165; final 0.227. The 0.25 bar was not lowered.
0 invalid samples. No RAM disk.

Subject: the same frozen `linux-v6.12` clone as exp-144. Fingerprint unchanged.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1497.0 ms | 1174.0 ms | 106.9 MiB |
| candidate | 1508.0 ms | 1184.4 ms | 107.0 MiB |

Self-comparison wall −0.23% [−0.58%, +1.10%]. Attachment only.

Counters-on attribution (same binary):

| Job | Component | Opens | Enum | Stats | Walk | Finish | Journal ret/clone | Roll-up merges |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `scan-index` hit 1 | 428.6 ms | 5,769 | 0 | 92,474 | 426.3 ms | 1.8 ms | 0 / 0 | 98,241 |
| `scan-index` hit 2 | 434.4 ms | 5,769 | 0 | 92,474 | 432.6 ms | 1.3 ms | 0 / 0 | 98,241 |
| `scan-index` hit 3 | 458.9 ms | 5,769 | 0 | 92,474 | 456.6 ms | 1.8 ms | 0 / 0 | 98,241 |
| `opened-discovery` hit 1 | 1196.9 ms | 5,769 | 0 | 92,473 | — | — | 5,772 / 5,772 | 438,021 |
| `opened-discovery` hit 2 | 1182.7 ms | 5,769 | 0 | 92,473 | — | — | 5,772 / 5,772 | 438,021 |
| `opened-discovery` hit 3 | 1178.9 ms | 5,769 | 0 | 92,473 | — | — | 5,772 / 5,772 | 438,021 |

Same directory opens.
`dir_enumeration_calls` 0 on both jobs (portable path).
Opened-discovery component is about **2.75×** first-pass (1,180 / 430). Darwin H127 was
8.8× because first-pass used `getattrlistbulk` while opened used `read_dir`+`fstatat`.
On Linux both jobs pay per-entry `statx`, so the ratio shrinks.
The extra opened cost is still one journal clone per directory plus 4.5× live roll-up
merges (438,021 vs 98,241). Control reads 358 on both.

## What the determination said

**Same** leftover identity as Darwin H127, different ratio.
Leftover is journal clones, live progressive roll-ups, and portable `read_dir`+`fstatat`
/ `statx`. No smallest skippable userspace cut ≥3%.

No engine patch. Do not port `macos_bulk`. Do not apply H115 rebuild to progressive
commits. Do not retry H104–H106.
