---
title: Linux cache-hit leftover after landed stack is already-landed restore work
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-144
  title: Linux cache-hit leftover after landed stack is already-landed restore work
  date: "2026-09-20"
  hypotheses:
    - H144
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
      sha256: 855568de31c7594f1c888883791def61edaace5f41a8ce0c24fed7a77775432b
      size_bytes: 3030680
      args: []
    candidate_binary:
      name: candidate
      sha256: 855568de31c7594f1c888883791def61edaace5f41a8ce0c24fed7a77775432b
      size_bytes: 3030680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-144-h144-linux-cache-hit-leftover.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 605297999.0
          candidate_median: 607317958.5
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.027
          change_pct: -0.085
          ci95_low_pct: -1.788
          ci95_high_pct: 1.256
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 516381082.0
          candidate_median: 518592192.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.027
          change_pct: 0.054
          ci95_low_pct: -1.659
          ci95_high_pct: 1.906
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 604788000.0
          candidate_median: 606771500.0
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.027
          change_pct: -0.09
          ci95_low_pct: -1.775
          ci95_high_pct: 1.293
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 555565000.0
          candidate_median: 559348000.0
          control_p95_over_median: 1.021
          candidate_p95_over_median: 1.028
          change_pct: 0.83
          ci95_low_pct: -0.287
          ci95_high_pct: 2.038
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 51986500.0
          candidate_median: 47870000.0
          control_p95_over_median: 1.156
          candidate_p95_over_median: 1.168
          change_pct: -10.789
          ci95_low_pct: -18.391
          ci95_high_pct: -0.32
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        blocked_ns:
          control_median: 564853.5
          candidate_median: 527145.0
          control_p95_over_median: 1.425
          candidate_p95_over_median: 1.422
          change_pct: -13.71
          ci95_low_pct: -24.353
          ci95_high_pct: 23.826
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 156221440.0
          candidate_median: 156233728.0
          control_p95_over_median: 1.0
          candidate_p95_over_median: 1.001
          change_pct: 0.007
          ci95_low_pct: -0.018
          ci95_high_pct: 0.084
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
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -0.085
    reason: "same leftover identity as Darwin H134: apply ~80ms / parse+candidates ~27ms each; no new userspace cut; do not retry H125/H129/H131/H133"
    commit: 345cd8fc
---
## What was predicted

H144 is the Linux leftover after the landed H125+H129+H131+H133 cache-hit stack, the
H134 analog.

Named before measuring:

- Job: same-binary 12-pair `content-cache-hit` on reconstructible `linux-v6.12`.
- Phase split from counters-on hits
  (`content_sidecar_{read,parse,candidates,apply}_us`).
- Determination: leftover is already-landed restore work and already-rejected stages
  (expected), or a userspace stage at least 3% Darwin H134 did not see.
- Do not retry H125/H129/H131/H133. Do not retry H116.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Quiet held: initial load/core 0.180, final 0.200, 0 of 30 samples invalidated.
`FDU_COUNTERS` unset on the pair.
3 warmups, 12 timed pairs, interleaved.
Same HEAD release probe both arms (sha256 `855568de…`). No RAM disk.

Subject: the same frozen `linux-v6.12` clone as exp-138 (92,474 entries / 86,643 files /
5,769 directories). Fingerprint and engine digest unchanged.
Content digest `06260f6c…`. 86,643 cache hits / 0 applied.
Every timed sample was `source=content-cache`.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 605.3 ms | 516.4 ms | 149.0 MiB |
| candidate | 607.3 ms | 518.6 ms | 149.0 MiB |

Self-comparison wall −0.09% [−1.79%, +1.26%]. Attachment only.

Three `FDU_COUNTERS=1` hits after the pair (apply excludes decode):

| Phase | Hit 1 µs | Hit 2 µs | Hit 3 µs |
| --- | ---: | ---: | ---: |
| read | 9,337 | 8,717 | 9,281 |
| parse | 26,794 | 26,722 | 26,900 |
| candidates | 28,069 | 26,196 | 27,863 |
| apply | 79,992 | 83,724 | 80,113 |

Apply is ~80–84 ms of a ~516 ms counters-off component (~16%). Candidates and parse are
each ~27 ms (~5%). Read is ~9 ms, under 3% of the 605 ms wall.
438,021 roll-up merges.
File opens 0. Walk 0.

`perf` and `/usr/bin/sample` are not on this image, so there is no `content_open` symbol
split. The phase timers are the same instrument Darwin H134 used.

## What the determination said

**Same** leftover identity as Darwin H134. Remaining cost is already-landed restore work
(H115 apply rebuild, H120 streaming parse-into-apply, H129 classify skip, H125
completeness) and already-rejected stages (H116 HashMap, parse-speed).
No new userspace cut ≥3%.

No engine patch. Do not retry H125/H129/H131/H133. Do not retry H116. Do not raise the
README 200K files/s or 4M cached lines/s.
